use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant, SystemTime};

use crate::model::{Node, ScanTree, VolumeInfo};
use crate::platform;

pub struct ScanProgress {
    pub entries: AtomicU64,
    pub files: AtomicU64,
    pub dirs: AtomicU64,
    pub bytes: AtomicU64,
    pub errors: AtomicU64,
    pub current: Mutex<String>,
    pub finished: AtomicBool,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            entries: AtomicU64::new(0),
            files: AtomicU64::new(0),
            dirs: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            current: Mutex::new(String::new()),
            finished: AtomicBool::new(false),
        }
    }
}

pub struct ScanHandle {
    pub rx: Receiver<ScanTree>,
    pub cancel: Arc<AtomicBool>,
    pub progress: Arc<ScanProgress>,
    pub started: Instant,
}

/// Starts a background scan. The UI polls `rx` for the finished tree.
pub fn start_scan(root: std::path::PathBuf) -> ScanHandle {
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let progress = Arc::new(ScanProgress::default());

    let cancel_thread = cancel.clone();
    let progress_thread = progress.clone();

    thread::Builder::new()
        .name("diskagent-scanner".to_string())
        .spawn(move || {
            let tree = scan_tree(root, cancel_thread, progress_thread.clone());
            progress_thread.finished.store(true, Ordering::Relaxed);
            let _ = tx.send(tree);
        })
        .expect("failed to spawn scanner thread");

    ScanHandle {
        rx,
        cancel,
        progress,
        started: Instant::now(),
    }
}

fn name_of(path: &Path) -> String {
    match path.file_name() {
        Some(n) => n.to_string_lossy().into_owned(),
        None => {
            let s = path.to_string_lossy().into_owned();
            if s.is_empty() {
                "\\".to_string()
            } else {
                s
            }
        }
    }
}

fn round_up(value: u64, cluster: u64) -> u64 {
    if cluster == 0 || value == 0 {
        return value;
    }
    value.div_ceil(cluster) * cluster
}

fn scan_tree(
    root_path: std::path::PathBuf,
    cancel: Arc<AtomicBool>,
    progress: Arc<ScanProgress>,
) -> ScanTree {
    let started = Instant::now();
    let volume: VolumeInfo = platform::volume_info(&root_path);
    let cluster = if volume.cluster_size == 0 {
        4096
    } else {
        volume.cluster_size
    };

    let root_name = name_of(&root_path);
    let mut nodes: Vec<Node> = Vec::with_capacity(1024);
    nodes.push(Node::dir(root_path.clone(), root_name, 0, None));

    let meta = std::fs::metadata(&root_path).ok();
    if let Some(m) = &meta {
        if let Ok(t) = m.modified() {
            nodes[0].modified = Some(t);
        }
    }

    let mut error_count: u64 = 0;
    let mut scanned_files: u64 = 0;
    let mut scanned_dirs: u64 = 0;

    // Explicit stack avoids recursion limits on deep trees.
    let mut stack: Vec<usize> = vec![0];

    'walk: while let Some(idx) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let path = nodes[idx].path.clone();
        {
            if let Ok(mut cur) = progress.current.lock() {
                *cur = path.to_string_lossy().into_owned();
            }
        }

        let read = match std::fs::read_dir(&path) {
            Ok(r) => r,
            Err(e) => {
                nodes[idx].read_error = Some(e.to_string());
                error_count += 1;
                progress.errors.store(error_count, Ordering::Relaxed);
                continue;
            }
        };

        for entry in read {
            if cancel.load(Ordering::Relaxed) {
                break 'walk;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(_) => {
                    error_count += 1;
                    progress.errors.store(error_count, Ordering::Relaxed);
                    continue;
                }
            };

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let depth = nodes[idx].depth.saturating_add(1);

            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => {
                    error_count += 1;
                    progress.errors.store(error_count, Ordering::Relaxed);
                    continue;
                }
            };

            if file_type.is_dir() {
                let mut node = Node::dir(path.clone(), name, depth, Some(idx));
                if let Ok(m) = entry.metadata() {
                    node.modified = m.modified().ok();
                    node.hidden = m.file_attributes() & 0x2 != 0;
                    node.compressed = m.file_attributes() & 0x800 != 0;
                    // Reparse points (symlinks/junctions) are never followed.
                    if m.file_attributes() & 0x400 != 0 {
                        node.read_error = Some("reparse point (not followed)".to_string());
                    }
                }
                let new_idx = nodes.len();
                nodes.push(node);
                nodes[idx].children.push(new_idx);
                scanned_dirs += 1;
                progress.dirs.store(scanned_dirs, Ordering::Relaxed);

                let is_reparse = nodes[new_idx]
                    .read_error
                    .as_deref()
                    .map(|s| s.contains("reparse"))
                    .unwrap_or(false);
                if !is_reparse {
                    stack.push(new_idx);
                }
            } else if file_type.is_file() {
                let (size, alloc, modified, compressed, hidden) = match entry.metadata() {
                    Ok(m) => {
                        let len = m.len();
                        let attrs = m.file_attributes();
                        let compressed = attrs & 0x800 != 0;
                        let sparse = attrs & 0x200 != 0;
                        let alloc = if compressed || sparse {
                            platform::compressed_size(&path).unwrap_or_else(|| round_up(len, cluster))
                        } else {
                            round_up(len, cluster)
                        };
                        (
                            len,
                            alloc,
                            m.modified().ok(),
                            compressed,
                            attrs & 0x2 != 0,
                        )
                    }
                    Err(_) => {
                        error_count += 1;
                        progress.errors.store(error_count, Ordering::Relaxed);
                        (0, 0, None, false, false)
                    }
                };

                let node = Node::file(
                    path,
                    name,
                    depth,
                    Some(idx),
                    size,
                    alloc,
                    modified,
                    compressed,
                    hidden,
                );
                let new_idx = nodes.len();
                nodes.push(node);
                nodes[idx].children.push(new_idx);
                scanned_files += 1;
                progress.files.store(scanned_files, Ordering::Relaxed);
                progress.entries.fetch_add(1, Ordering::Relaxed);
                progress.bytes.fetch_add(size, Ordering::Relaxed);
            } else {
                // Sockets, devices and other special entries are ignored.
            }
        }
    }

    // Aggregate child stats into parents. Children always have a larger index
    // than their parent, so a reverse pass is a valid post-order traversal.
    for i in (1..nodes.len()).rev() {
        let (size, alloc, files, folders, is_dir) = {
            let n = &nodes[i];
            (n.size, n.alloc, n.files, n.folders, n.is_dir)
        };
        if let Some(p) = nodes[i].parent {
            let parent = &mut nodes[p];
            parent.size += size;
            parent.alloc += alloc;
            parent.files += files;
            if is_dir {
                parent.folders += folders + 1;
            }
        }
    }

    if let Some(m) = &meta {
        nodes[0].size = nodes[0].size.max(m.len());
    }

    let cancelled = cancel.load(Ordering::Relaxed);
    let duration = started.elapsed();

    ScanTree {
        nodes,
        root: 0,
        root_path,
        volume,
        duration,
        cancelled,
        error_count,
        scanned_files,
        scanned_dirs,
    }
}

/// Convenience for tests / future features: scan synchronously.
#[allow(dead_code)]
pub fn scan_blocking(root: std::path::PathBuf) -> ScanTree {
    scan_tree(
        root,
        Arc::new(AtomicBool::new(false)),
        Arc::new(ScanProgress::default()),
    )
}

#[allow(dead_code)]
pub fn now() -> SystemTime {
    SystemTime::now()
}
