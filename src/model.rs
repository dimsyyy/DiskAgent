use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub type NodeId = usize;

/// One scanned file-system entry.
#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: u16,
    /// Logical size in bytes (sum of descendant file sizes for directories).
    pub size: u64,
    /// Allocated size on disk in bytes.
    pub alloc: u64,
    /// Number of files in this node (recursive for directories, 1 for files).
    pub files: u64,
    /// Number of folders in this node (recursive for directories).
    pub folders: u64,
    pub modified: Option<SystemTime>,
    pub compressed: bool,
    pub hidden: bool,
    pub read_error: Option<String>,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

impl Node {
    pub fn dir(path: PathBuf, name: String, depth: u16, parent: Option<NodeId>) -> Self {
        Self {
            path,
            name,
            is_dir: true,
            depth,
            size: 0,
            alloc: 0,
            files: 0,
            folders: 0,
            modified: None,
            compressed: false,
            hidden: false,
            read_error: None,
            parent,
            children: Vec::new(),
        }
    }

    pub fn file(
        path: PathBuf,
        name: String,
        depth: u16,
        parent: Option<NodeId>,
        size: u64,
        alloc: u64,
        modified: Option<SystemTime>,
        compressed: bool,
        hidden: bool,
    ) -> Self {
        Self {
            path,
            name,
            is_dir: false,
            depth,
            size,
            alloc,
            files: 1,
            folders: 0,
            modified,
            compressed,
            hidden,
            read_error: None,
            parent,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct VolumeInfo {
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub cluster_size: u64,
    pub fs_name: String,
}

#[derive(Debug)]
pub struct ScanTree {
    pub nodes: Vec<Node>,
    pub root: NodeId,
    pub root_path: PathBuf,
    pub volume: VolumeInfo,
    pub duration: Duration,
    pub cancelled: bool,
    pub error_count: u64,
    pub scanned_files: u64,
    pub scanned_dirs: u64,
}

impl ScanTree {
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }

    pub fn parent_id(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id].parent
    }

    pub fn ancestors(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut cur = self.nodes[id].parent;
        while let Some(p) = cur {
            out.push(p);
            cur = self.nodes[p].parent;
        }
        out.reverse();
        out
    }

    pub fn path_of(&self, id: NodeId) -> &std::path::Path {
        &self.nodes[id].path
    }

    /// Total size of the whole scan (unfiltered).
    pub fn total_size(&self) -> u64 {
        self.nodes[self.root].size
    }

    /// Removes `id` and all of its descendants from the in-memory tree.
    ///
    /// Returns the new id of the parent (or root as a fallback) together with
    /// an old-to-new node index map, so callers can remap selection and
    /// expansion state. The scan root itself cannot be removed.
    pub fn remove_subtree(&mut self, id: NodeId) -> Option<(NodeId, Vec<Option<NodeId>>)> {
        let count = self.nodes.len();
        if id >= count || id == self.root {
            return None;
        }

        let mut removed = vec![false; count];
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            if current >= count || removed[current] {
                continue;
            }
            removed[current] = true;
            stack.extend(self.nodes[current].children.iter().copied());
        }

        let mut removed_files = 0u64;
        let mut removed_dirs = 0u64;
        let mut removed_errors = 0u64;
        for (index, node) in self.nodes.iter().enumerate() {
            if !removed[index] {
                continue;
            }
            if node.is_dir {
                removed_dirs += 1;
            } else {
                removed_files += 1;
            }
            if node.read_error.is_some() {
                removed_errors += 1;
            }
        }

        let parent_old = self.nodes[id].parent;
        let old_root = self.root;

        let mut map: Vec<Option<NodeId>> = vec![None; count];
        let mut kept = 0usize;
        for (old, was_removed) in removed.iter().enumerate() {
            if !was_removed {
                map[old] = Some(kept);
                kept += 1;
            }
        }

        let mut nodes = Vec::with_capacity(kept);
        for (old, node) in self.nodes.iter().enumerate() {
            if map[old].is_none() {
                continue;
            }
            let mut kept_node = node.clone();
            kept_node.parent = node.parent.and_then(|parent| map[parent]);
            kept_node.children = node
                .children
                .iter()
                .filter_map(|child| map[*child])
                .collect();
            nodes.push(kept_node);
        }

        self.nodes = nodes;
        self.root = map[old_root].unwrap_or(0);
        let fallback = parent_old
            .and_then(|parent| map.get(parent).copied().flatten())
            .unwrap_or(self.root);

        self.scanned_files = self.scanned_files.saturating_sub(removed_files);
        self.scanned_dirs = self.scanned_dirs.saturating_sub(removed_dirs);
        self.error_count = self.error_count.saturating_sub(removed_errors);

        // Rebuild directory aggregates from the remaining leaves.
        for node in self.nodes.iter_mut() {
            if node.is_dir {
                node.size = 0;
                node.alloc = 0;
                node.files = 0;
                node.folders = 0;
            }
        }
        for index in (1..self.nodes.len()).rev() {
            let (size, alloc, files, folders, is_dir) = {
                let node = &self.nodes[index];
                (node.size, node.alloc, node.files, node.folders, node.is_dir)
            };
            if let Some(parent) = self.nodes[index].parent {
                let parent = &mut self.nodes[parent];
                parent.size += size;
                parent.alloc += alloc;
                parent.files += files;
                if is_dir {
                    parent.folders += folders + 1;
                }
            }
        }

        let modified = std::fs::metadata(&self.nodes[fallback].path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        if let Some(modified) = modified {
            self.nodes[fallback].modified = Some(modified);
        }

        Some((fallback, map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sample_tree() -> ScanTree {
        let root_path = PathBuf::from(r"C:\sample");
        let mut root = Node::dir(root_path.clone(), "sample".to_string(), 0, None);
        let mut dir = Node::dir(root_path.join("sub"), "sub".to_string(), 1, Some(0));
        let inner = Node::file(
            root_path.join("sub").join("inner.bin"),
            "inner.bin".to_string(),
            2,
            Some(1),
            300,
            512,
            None,
            false,
            false,
        );
        let file = Node::file(
            root_path.join("keep.txt"),
            "keep.txt".to_string(),
            1,
            Some(0),
            100,
            512,
            None,
            false,
            false,
        );
        root.children = vec![1, 3];
        root.size = 400;
        root.alloc = 1024;
        root.files = 2;
        root.folders = 1;
        dir.children = vec![2];
        dir.size = 300;
        dir.alloc = 512;
        dir.files = 1;

        ScanTree {
            nodes: vec![root, dir, inner, file],
            root: 0,
            root_path,
            volume: VolumeInfo::default(),
            duration: Duration::ZERO,
            cancelled: false,
            error_count: 0,
            scanned_files: 2,
            scanned_dirs: 1,
        }
    }

    #[test]
    fn remove_subtree_remaps_and_reaggregates() {
        let mut tree = sample_tree();
        let (fallback, map) = tree.remove_subtree(1).expect("subtree removed");

        assert_eq!(fallback, 0);
        assert_eq!(map[1], None);
        assert_eq!(map[3], Some(1));
        assert_eq!(tree.nodes.len(), 2);
        assert_eq!(tree.nodes[0].children, vec![1]);
        assert_eq!(tree.nodes[1].name, "keep.txt");
        assert_eq!(tree.nodes[0].size, 100);
        assert_eq!(tree.nodes[0].alloc, 512);
        assert_eq!(tree.nodes[0].files, 1);
        assert_eq!(tree.nodes[0].folders, 0);
        assert_eq!(tree.scanned_files, 1);
        assert_eq!(tree.scanned_dirs, 0);
    }

    #[test]
    fn scan_root_cannot_be_removed() {
        let mut tree = sample_tree();
        assert!(tree.remove_subtree(0).is_none());
        assert_eq!(tree.nodes.len(), 4);
    }
}
