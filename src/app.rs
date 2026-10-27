use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText, Vec2};
use egui::ViewportCommand;

use crate::filter::{self, Filter, ViewStats};
use crate::format::{self, Unit};
use crate::i18n::{t, Lang};
use crate::model::{Node, NodeId, ScanTree};
use crate::platform;
use crate::scan::{start_scan, ScanHandle};
use crate::settings::Settings;
use crate::theme;
use crate::ui::{dialogs, icons, ribbon, statusbar, treemap};
use crate::web;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Tab {
    Home,
    Scan,
    View,
    Options,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ContentView {
    Details,
    Treemap,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SortKey {
    Name,
    Size,
    Alloc,
    Files,
    Folders,
    Percent,
    Modified,
}

impl SortKey {
    pub fn key(self) -> &'static str {
        match self {
            SortKey::Name => "name",
            SortKey::Size => "size",
            SortKey::Alloc => "alloc",
            SortKey::Files => "files",
            SortKey::Folders => "folders",
            SortKey::Percent => "percent",
            SortKey::Modified => "modified",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "name" => SortKey::Name,
            "alloc" => SortKey::Alloc,
            "files" => SortKey::Files,
            "folders" => SortKey::Folders,
            "percent" => SortKey::Percent,
            "modified" => SortKey::Modified,
            _ => SortKey::Size,
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            SortKey::Name => t(lang, "Name"),
            SortKey::Size => t(lang, "Size"),
            SortKey::Alloc => t(lang, "Allocated"),
            SortKey::Files => t(lang, "Files"),
            SortKey::Folders => t(lang, "Folders"),
            SortKey::Percent => t(lang, "Percent"),
            SortKey::Modified => t(lang, "Last Modified"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Columns {
    pub size: bool,
    pub alloc: bool,
    pub files: bool,
    pub folders: bool,
    pub percent: bool,
    pub modified: bool,
}

impl Default for Columns {
    fn default() -> Self {
        Self {
            size: true,
            alloc: true,
            files: true,
            folders: true,
            percent: true,
            modified: true,
        }
    }
}

pub struct DiskAgentApp {
    pub lang: Lang,
    pub tab: Tab,
    pub view: ContentView,

    pub scan: Option<ScanTree>,
    pub job: Option<ScanHandle>,
    pub root_path: Option<PathBuf>,
    pub history: Vec<PathBuf>,
    pub history_pos: usize,
    pub scan_started: Option<Instant>,

    pub selected: Option<NodeId>,
    pub expanded: HashSet<NodeId>,
    pub treemap_root: Option<NodeId>,
    pub rows: Vec<NodeId>,
    pub rows_dirty: bool,
    pub stats: Option<ViewStats>,
    pub stats_dirty: bool,

    pub filter: Filter,
    pub filter_draft: Filter,
    pub filter_min_mb: f64,
    pub filter_max_age: u32,
    pub filter_hide_empty: bool,
    pub filter_open: bool,
    pub hide_empty: bool,

    pub columns: Columns,
    pub unit: Unit,
    pub decimals: u8,
    pub sort_key: SortKey,
    pub sort_desc: bool,

    pub search_open: bool,
    pub search_text: String,
    pub search_hits: Vec<NodeId>,
    pub search_pos: usize,
    pub search_focus: bool,

    pub columns_open: bool,
    pub new_folder_open: bool,
    pub new_folder_name: String,
    pub delete_open: bool,
    pub delete_target: Option<NodeId>,

    pub ribbon_collapsed: bool,
    pub expand_menu_open: bool,
    pub expand_menu_pos: egui::Pos2,
    pub expand_menu_just_opened: bool,
    pub status: Option<(String, Instant)>,
    pub row_scroll: Option<usize>,

    pub settings: Settings,
    pub settings_dirty: bool,
    pub last_settings_save: Instant,
    pub elevated: bool,
    pub close_requested: bool,
}

impl DiskAgentApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let settings = Settings::load();
        let lang = settings.lang();
        let unit = settings.unit();
        let columns = Columns {
            size: settings.show_size,
            alloc: settings.show_alloc,
            files: settings.show_files,
            folders: settings.show_folders,
            percent: settings.show_percent,
            modified: settings.show_modified,
        };
        let filter = Filter::default();

        let mut app = Self {
            lang,
            tab: Tab::Home,            view: ContentView::Details,
            scan: None,
            job: None,
            root_path: None,
            history: Vec::new(),
            history_pos: 0,
            scan_started: None,
            selected: None,
            expanded: HashSet::new(),
            treemap_root: None,
            rows: Vec::new(),
            rows_dirty: false,
            stats: None,
            stats_dirty: false,
            filter: filter.clone(),
            filter_draft: filter,
            filter_min_mb: 0.0,
            filter_max_age: 0,
            filter_hide_empty: false,
            filter_open: false,
            hide_empty: settings.hide_empty,
            columns,
            unit,
            decimals: settings.decimals.min(4),
            sort_key: SortKey::from_key(&settings.sort_key),
            sort_desc: settings.sort_desc,
            search_open: false,
            search_text: String::new(),
            search_hits: Vec::new(),
            search_pos: 0,
            search_focus: false,
            columns_open: false,
            new_folder_open: false,
            new_folder_name: String::new(),
            delete_open: false,
            delete_target: None,
            ribbon_collapsed: settings.ribbon_collapsed,
            expand_menu_open: false,
            expand_menu_pos: egui::Pos2::ZERO,
            expand_menu_just_opened: false,
            status: None,
            row_scroll: None,
            settings,
            settings_dirty: false,
            last_settings_save: Instant::now(),
            elevated: platform::is_elevated(),
            close_requested: false,
        };

        // Optional positional argument: scan this folder immediately.
        if let Some(arg) = std::env::args().nth(1) {
            let path = PathBuf::from(arg);
            if path.is_dir() {
                app.start_scan_path(path, true, &cc.egui_ctx);
            }
        }

        app
    }

    pub fn t(&self, en: &'static str) -> &'static str {
        t(self.lang, en)
    }

    pub fn is_scanning(&self) -> bool {
        self.job.is_some()
    }

    pub fn filter_active(&self) -> bool {
        self.filter.is_active() || self.hide_empty
    }

    pub fn set_status(&mut self, text: impl Into<String>) {
        self.status = Some((text.into(), Instant::now()));
    }

    // ------------------------------------------------------------------
    // Scanning
    // ------------------------------------------------------------------

    pub fn select_directory(&mut self, ctx: &egui::Context) {
        let mut dialog = rfd::FileDialog::new();
        if let Some(path) = &self.root_path {
            dialog = dialog.set_directory(path);
        } else if let Some(last) = &self.settings.last_dir {
            dialog = dialog.set_directory(last);
        }
        if let Some(path) = dialog.pick_folder() {
            self.start_scan_path(path, true, ctx);
        }
    }

    pub fn start_scan_path(&mut self, path: PathBuf, record_history: bool, ctx: &egui::Context) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Relaxed);
        }

        if record_history {
            if self.history_pos + 1 < self.history.len() {
                self.history.truncate(self.history_pos + 1);
            }
            if self.history.last() != Some(&path) {
                self.history.push(path.clone());
            }
            self.history_pos = self.history.len().saturating_sub(1);
        }

        self.scan = None;
        self.stats = None;
        self.rows.clear();
        self.expanded.clear();
        self.selected = None;
        self.treemap_root = None;
        self.stats_dirty = false;
        self.rows_dirty = false;
        self.delete_open = false;
        self.delete_target = None;
        self.root_path = Some(path.clone());
        self.job = Some(start_scan(path.clone()));
        self.scan_started = Some(Instant::now());
        self.settings.last_dir = Some(path.to_string_lossy().into_owned());
        self.settings_dirty = true;
        self.status = None;
        ctx.request_repaint();
    }

    pub fn stop_scan(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Relaxed);
            let msg = self.t("Stopping...").to_string();
            self.set_status(msg);
        }
    }

    pub fn refresh(&mut self, ctx: &egui::Context) {
        if let Some(path) = self.root_path.clone() {
            self.start_scan_path(path, false, ctx);
        }
    }

    pub fn navigate_history(&mut self, delta: isize, ctx: &egui::Context) {
        let new_pos = self.history_pos as isize + delta;
        if new_pos < 0 || new_pos as usize >= self.history.len() {
            return;
        }
        self.history_pos = new_pos as usize;
        let path = self.history[self.history_pos].clone();
        self.start_scan_path(path, false, ctx);
    }

    pub fn poll_scan(&mut self) {
        let mut received = None;
        if let Some(job) = &self.job {
            loop {
                match job.rx.try_recv() {
                    Ok(tree) => {
                        received = Some(tree);
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                }
            }
        }

        if let Some(tree) = received {
            self.job = None;
            self.install_tree(tree);
        }
    }

    fn install_tree(&mut self, tree: ScanTree) {
        let cancelled = tree.cancelled;
        let errors = tree.error_count;
        self.settings.last_dir = Some(tree.root_path.to_string_lossy().into_owned());
        self.root_path = Some(tree.root_path.clone());
        self.scan = Some(tree);
        self.stats_dirty = true;
        self.rows_dirty = true;
        self.expanded.clear();
        self.expanded.insert(0);
        self.selected = Some(0);
        self.treemap_root = Some(0);
        self.scan_started = None;

        let msg = if cancelled {
            self.t("Scan stopped - partial results").to_string()
        } else if errors > 0 {
            format!("{} ({} errors)", self.t("Scan complete"), errors)
        } else {
            self.t("Scan complete").to_string()
        };
        self.set_status(msg);
        self.settings_dirty = true;
    }

    // ------------------------------------------------------------------
    // Derived data
    // ------------------------------------------------------------------

    pub fn ensure_rows(&mut self) {
        if self.stats_dirty {
            self.refresh_stats();
        }
        if self.rows_dirty {
            self.rebuild_rows();
        }
    }

    pub fn refresh_stats(&mut self) {
        self.stats = self
            .scan
            .as_ref()
            .map(|tree| filter::compute(tree, &self.filter, self.hide_empty));
        self.stats_dirty = false;
        self.rows_dirty = true;
    }

    fn rebuild_rows(&mut self) {
        self.rows.clear();
        let Some(tree) = self.scan.as_ref() else {
            self.rows_dirty = false;
            return;
        };
        let Some(stats) = self.stats.as_ref() else {
            self.rows_dirty = false;
            return;
        };

        let mut rows = Vec::new();
        let mut stack = vec![tree.root];
        while let Some(id) = stack.pop() {
            if !stats.visible[id] {
                continue;
            }
            rows.push(id);
            if tree.nodes[id].is_dir && self.expanded.contains(&id) {
                let kids = self.sorted_children(tree, stats, id);
                for child in kids.into_iter().rev() {
                    stack.push(child);
                }
            }
        }
        self.rows = rows;
        self.rows_dirty = false;
    }

    pub fn sorted_children(
        &self,
        tree: &ScanTree,
        stats: &ViewStats,
        id: NodeId,
    ) -> Vec<NodeId> {
        let mut kids = tree.nodes[id].children.clone();
        let key = self.sort_key;
        let desc = self.sort_desc;
        kids.sort_by(|a, b| {
            let na = &tree.nodes[*a];
            let nb = &tree.nodes[*b];
            let dir_cmp = nb.is_dir.cmp(&na.is_dir);
            if dir_cmp != std::cmp::Ordering::Equal {
                return dir_cmp;
            }
            let cmp = match key {
                SortKey::Name => na
                    .name
                    .to_lowercase()
                    .cmp(&nb.name.to_lowercase()),
                SortKey::Size => stats.size[*a].cmp(&stats.size[*b]),
                SortKey::Alloc => stats.alloc[*a].cmp(&stats.alloc[*b]),
                SortKey::Files => stats.files[*a].cmp(&stats.files[*b]),
                SortKey::Folders => stats.folders[*a].cmp(&stats.folders[*b]),
                SortKey::Percent => {
                    let pa = percent_value(tree, stats, *a);
                    let pb = percent_value(tree, stats, *b);
                    pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortKey::Modified => na.modified.cmp(&nb.modified),
            };
            if desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
        kids
    }

    pub fn percent_of(&self, id: NodeId) -> f64 {
        match (&self.scan, &self.stats) {
            (Some(tree), Some(stats)) => percent_value(tree, stats, id),
            _ => 0.0,
        }
    }

    pub fn percent_of_total(&self, id: NodeId) -> f64 {
        match (&self.scan, &self.stats) {
            (Some(tree), Some(stats)) => {
                let total = stats.size[tree.root];
                if total == 0 {
                    0.0
                } else {
                    stats.size[id] as f64 / total as f64 * 100.0
                }
            }
            _ => 0.0,
        }
    }

    pub fn set_sort(&mut self, key: SortKey) {
        if self.sort_key == key {
            self.sort_desc = !self.sort_desc;
        } else {
            self.sort_key = key;
            self.sort_desc = !matches!(key, SortKey::Name);
        }
        self.rows_dirty = true;
        self.settings_dirty = true;
    }

    pub fn set_unit(&mut self, unit: Unit) {
        self.unit = unit;
        self.settings_dirty = true;
    }

    // ------------------------------------------------------------------
    // Selection, expansion and search
    // ------------------------------------------------------------------

    pub fn select_node(&mut self, id: NodeId, scroll: bool) {
        self.selected = Some(id);
        self.ensure_visible(id);
        if scroll {
            self.row_scroll = self.rows.iter().position(|&x| x == id);
        }
    }

    pub fn ensure_visible(&mut self, id: NodeId) {
        let ancestors = self
            .scan
            .as_ref()
            .map(|tree| tree.ancestors(id))
            .unwrap_or_default();
        for ancestor in ancestors {
            self.expanded.insert(ancestor);
        }
        self.rows_dirty = true;
        self.ensure_rows();
    }

    pub fn toggle_expanded(&mut self, id: NodeId) {
        let is_dir = self
            .scan
            .as_ref()
            .map(|tree| tree.nodes[id].is_dir)
            .unwrap_or(false);
        if !is_dir {
            return;
        }
        if self.expanded.contains(&id) {
            self.expanded.remove(&id);
        } else {
            self.expanded.insert(id);
        }
        self.rows_dirty = true;
    }

    pub fn expand_levels(&mut self, levels: u16) {
        let ids: Vec<NodeId> = self
            .scan
            .as_ref()
            .map(|tree| {
                tree.nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, node)| node.is_dir && node.depth < levels)
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
        self.expanded.clear();
        self.expanded.extend(ids);
        self.rows_dirty = true;
    }

    pub fn expand_all(&mut self) {
        let ids: Vec<NodeId> = self
            .scan
            .as_ref()
            .map(|tree| {
                tree.nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, node)| node.is_dir)
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
        self.expanded.extend(ids);
        self.rows_dirty = true;
    }

    pub fn collapse_all(&mut self) {
        self.expanded.clear();
        if let Some(tree) = &self.scan {
            self.expanded.insert(tree.root);
        }
        self.rows_dirty = true;
    }

    pub fn move_selection(&mut self, delta: isize) {
        self.ensure_rows();
        if self.rows.is_empty() {
            return;
        }
        let current = self
            .selected
            .and_then(|id| self.rows.iter().position(|&x| x == id))
            .unwrap_or(0);
        let next = (current as isize + delta).clamp(0, self.rows.len() as isize - 1) as usize;
        let id = self.rows[next];
        self.select_node(id, true);
    }

    pub fn select_edge(&mut self, end: bool) {
        self.ensure_rows();
        if self.rows.is_empty() {
            return;
        }
        let idx = if end { self.rows.len() - 1 } else { 0 };
        let id = self.rows[idx];
        self.select_node(id, true);
    }

    pub fn step_expand(&mut self) {
        let Some(id) = self.selected else { return };
        let is_dir = self
            .scan
            .as_ref()
            .map(|tree| tree.nodes[id].is_dir)
            .unwrap_or(false);
        if !is_dir {
            return;
        }
        if !self.expanded.contains(&id) {
            self.expanded.insert(id);
            self.rows_dirty = true;
            return;
        }
        let first = match (self.scan.as_ref(), self.stats.as_ref()) {
            (Some(tree), Some(stats)) => {
                self.sorted_children(tree, stats, id).first().copied()
            }
            _ => None,
        };
        if let Some(first) = first {
            self.select_node(first, true);
        }
    }

    pub fn step_collapse(&mut self) {
        let Some(id) = self.selected else { return };
        if self.expanded.contains(&id) {
            self.expanded.remove(&id);
            self.rows_dirty = true;
            return;
        }
        let parent = self
            .scan
            .as_ref()
            .and_then(|tree| tree.nodes[id].parent);
        if let Some(parent) = parent {
            self.select_node(parent, true);
        }
    }

    pub fn parent_folder(&mut self) {
        let Some(id) = self.selected else { return };
        let parent = self
            .scan
            .as_ref()
            .and_then(|tree| tree.nodes[id].parent);
        if let Some(parent) = parent {
            self.select_node(parent, true);
        }
    }

    pub fn open_search(&mut self) {
        self.search_open = true;
        self.search_focus = true;
    }

    pub fn run_search(&mut self) {
        self.search_hits.clear();
        let query = self.search_text.trim().to_lowercase();
        if query.is_empty() {
            return;
        }
        if let Some(tree) = &self.scan {
            for (i, node) in tree.nodes.iter().enumerate() {
                if node.name.to_lowercase().contains(&query) {
                    self.search_hits.push(i);
                    if self.search_hits.len() >= 5000 {
                        break;
                    }
                }
            }
        }
        self.search_pos = 0;
        if let Some(&id) = self.search_hits.first() {
            self.select_node(id, true);
        }
    }

    pub fn next_search_result(&mut self) {
        if self.search_hits.is_empty() {
            return;
        }
        self.search_pos = (self.search_pos + 1) % self.search_hits.len();
        let id = self.search_hits[self.search_pos];
        self.select_node(id, true);
    }

    // ------------------------------------------------------------------
    // Actions
    // ------------------------------------------------------------------

    pub fn open_selected_location(&mut self) {
        let Some(path) = self
            .selected
            .and_then(|id| self.scan.as_ref().map(|tree| tree.nodes[id].path.clone()))
            .or_else(|| self.root_path.clone())
        else {
            return;
        };
        platform::open_location(&path, path.is_file());
    }

    /// Opens Google with a ready-made question about the selected folder/file.
    pub fn explain_node(&mut self, id: NodeId) {
        let Some((name, is_dir, path)) = self
            .scan
            .as_ref()
            .and_then(|tree| tree.nodes.get(id))
            .map(|node| (node.name.clone(), node.is_dir, node.path.clone()))
        else {
            return;
        };

        let query = web::explain_query(&name, is_dir, &path, self.lang);
        let url = web::google_search_url(&query, self.lang);

        if platform::open_url(&url) {
            let message = format!("{}: {}", self.t("Opening Google search"), name);
            self.set_status(message);
        } else {
            let message = self.t("Could not open browser").to_string();
            self.set_status(message);
        }
    }

    pub fn copy_selected_path(&mut self, ctx: &egui::Context) {
        let text = self
            .selected
            .and_then(|id| self.scan.as_ref().map(|tree| tree.nodes[id].path.to_string_lossy().into_owned()))
            .or_else(|| self.root_path.as_ref().map(|p| p.to_string_lossy().into_owned()));
        if let Some(text) = text {
            ctx.output_mut(|o| o.copied_text = text);
            let msg = self.t("Copy path").to_string();
            self.set_status(msg);
        }
    }

    pub fn start_new_folder(&mut self) {
        self.new_folder_name.clear();
        self.new_folder_open = true;
    }

    pub fn create_folder(&mut self) {
        let name = self.new_folder_name.trim().to_string();
        if name.is_empty() {
            return;
        }
        let Some(tree) = self.scan.as_ref() else { return };
        let parent_id = self
            .selected
            .map(|id| {
                if tree.nodes[id].is_dir {
                    id
                } else {
                    tree.nodes[id].parent.unwrap_or(tree.root)
                }
            })
            .unwrap_or(tree.root);
        let parent_path = tree.nodes[parent_id].path.clone();
        let new_path = parent_path.join(&name);

        match std::fs::create_dir(&new_path) {
            Ok(()) => {
                let new_id = {
                    let Some(tree) = self.scan.as_mut() else { return };
                    let depth = tree.nodes[parent_id].depth.saturating_add(1);
                    let mut node = Node::dir(new_path.clone(), name, depth, Some(parent_id));
                    if let Ok(meta) = std::fs::metadata(&new_path) {
                        node.modified = meta.modified().ok();
                    }
                    let new_id = tree.nodes.len();
                    tree.nodes.push(node);
                    tree.nodes[parent_id].children.push(new_id);
                    new_id
                };
                self.expanded.insert(parent_id);
                self.stats_dirty = true;
                self.rows_dirty = true;
                self.new_folder_open = false;
                let msg = format!("{}: {}", self.t("New folder"), new_path.display());
                self.set_status(msg);
                self.select_node(new_id, true);
            }
            Err(err) => {
                let msg = format!("{}: {}", self.t("Could not create folder"), err);
                self.set_status(msg);
            }
        }
    }

    pub fn request_delete(&mut self, id: NodeId) {
        let allowed = self
            .scan
            .as_ref()
            .map(|tree| id < tree.nodes.len() && id != tree.root)
            .unwrap_or(false);
        if !allowed {
            let msg = self.t("The scan root cannot be deleted").to_string();
            self.set_status(msg);
            return;
        }
        self.selected = Some(id);
        self.delete_target = Some(id);
        self.delete_open = true;
    }

    pub fn cancel_delete(&mut self) {
        self.delete_open = false;
        self.delete_target = None;
    }

    pub fn confirm_delete(&mut self) {
        let Some(id) = self.delete_target.take() else {
            return;
        };
        self.delete_open = false;

        let Some(tree) = self.scan.as_ref() else { return };
        if id >= tree.nodes.len() || id == tree.root {
            return;
        }
        let path = tree.nodes[id].path.clone();

        match platform::delete_permanently(&path) {
            Ok(()) => {
                let removed = self.scan.as_mut().and_then(|tree| {
                    let result = tree.remove_subtree(id);
                    if result.is_some() {
                        tree.volume = platform::volume_info(&tree.root_path);
                    }
                    result
                });

                if let Some((fallback, map)) = removed {
                    self.expanded = self
                        .expanded
                        .iter()
                        .filter_map(|old| map.get(*old).copied().flatten())
                        .collect();
                    self.expanded.insert(fallback);

                    self.selected = self
                        .selected
                        .and_then(|old| map.get(old).copied().flatten())
                        .or(Some(fallback));
                    self.treemap_root = self
                        .treemap_root
                        .and_then(|old| map.get(old).copied().flatten())
                        .or(Some(fallback));

                    self.search_hits.clear();
                    self.search_pos = 0;
                    self.stats_dirty = true;
                    self.rows_dirty = true;
                    self.row_scroll = None;
                    self.delete_target = None;

                    let msg = format!("{}: {}", self.t("Deleted"), path.display());
                    self.set_status(msg);
                }
            }
            Err(err) => {
                self.delete_target = None;
                let msg = format!("{}: {} ({})", self.t("Could not delete"), path.display(), err);
                self.set_status(msg);
            }
        }
    }

    pub fn export_csv(&mut self) {
        let (Some(tree), Some(stats)) = (self.scan.as_ref(), self.stats.as_ref()) else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("DiskAgent-export.csv")
            .add_filter("CSV", &["csv"])
            .save_file()
        else {
            return;
        };

        let mut out = String::from("\u{feff}");
        out.push_str("Path,Name,Folder,Size (bytes),Allocated (bytes),Files,Folders,Percent of parent,Last modified\n");

        for &id in &self.rows {
            let node = &tree.nodes[id];
            let percent = percent_value(tree, stats, id);
            out.push_str(&csv_escape(&node.path.to_string_lossy()));
            out.push(',');
            out.push_str(&csv_escape(&node.name));
            out.push(',');
            out.push_str(if node.is_dir { "yes" } else { "no" });
            out.push(',');
            out.push_str(&stats.size[id].to_string());
            out.push(',');
            out.push_str(&stats.alloc[id].to_string());
            out.push(',');
            out.push_str(&stats.files[id].to_string());
            out.push(',');
            out.push_str(&stats.folders[id].to_string());
            out.push(',');
            out.push_str(&format!("{percent:.2}"));
            out.push(',');
            if let Some(modified) = node.modified {
                out.push_str(&platform::format_time(modified));
            }
            out.push('\n');
        }

        match std::fs::write(&path, out) {
            Ok(()) => {
                let msg = format!("{}: {}", self.t("Export"), path.display());
                self.set_status(msg);
            }
            Err(err) => {
                let msg = format!("{}: {}", self.t("Could not export the file"), err);
                self.set_status(msg);
            }
        }
    }

    pub fn request_admin_restart(&mut self, ctx: &egui::Context) {
        if platform::run_as_admin() {
            self.close_requested = true;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        } else {
            let msg = self.t("Start as Administrator").to_string();
            self.set_status(msg);
        }
    }

    // ------------------------------------------------------------------
    // Keyboard
    // ------------------------------------------------------------------

    fn handle_keys(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        let input = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::ArrowRight),
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Home),
                i.key_pressed(egui::Key::End),
                i.key_pressed(egui::Key::PageDown),
                i.key_pressed(egui::Key::PageUp),
                i.modifiers.command && i.key_pressed(egui::Key::F),
            )
        });
        if input.9 {
            self.open_search();
        }
        let copy_path = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C));
        if copy_path {
            self.copy_selected_path(ctx);
        }
        if input.0 {
            self.move_selection(1);
        }
        if input.1 {
            self.move_selection(-1);
        }
        if input.3 {
            self.step_expand();
        }
        if input.2 {
            self.step_collapse();
        }
        if input.4 {
            if let Some(id) = self.selected {
                if let Some(tree) = &self.scan {
                    if tree.nodes[id].is_dir {
                        self.toggle_expanded(id);
                    }
                }
            }
        }
        if input.5 {
            self.select_edge(false);
        }
        if input.6 {
            self.select_edge(true);
        }
        if input.7 {
            self.move_selection(20);
        }
        if input.8 {
            self.move_selection(-20);
        }
    }

    // ------------------------------------------------------------------
    // Settings
    // ------------------------------------------------------------------

    fn sync_settings(&mut self) {
        self.settings.lang = self.lang.key().to_string();
        self.settings.unit = self.unit.key().to_string();
        self.settings.decimals = self.decimals;
        self.settings.show_size = self.columns.size;
        self.settings.show_alloc = self.columns.alloc;
        self.settings.show_files = self.columns.files;
        self.settings.show_folders = self.columns.folders;
        self.settings.show_percent = self.columns.percent;
        self.settings.show_modified = self.columns.modified;
        self.settings.sort_key = self.sort_key.key().to_string();
        self.settings.sort_desc = self.sort_desc;
        self.settings.hide_empty = self.hide_empty;
        self.settings.ribbon_collapsed = self.ribbon_collapsed;
    }

    fn maybe_save_settings(&mut self, ctx: &egui::Context) {
        if self.settings_dirty && self.last_settings_save.elapsed() > Duration::from_millis(700) {
            self.sync_settings();
            self.settings.save();
            self.settings_dirty = false;
            self.last_settings_save = Instant::now();
            ctx.request_repaint();
        }
    }

    // ------------------------------------------------------------------
    // Content
    // ------------------------------------------------------------------

    fn content_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            let tabs = [
                (ContentView::Details, self.t("Details")),
                (ContentView::Treemap, self.t("Treemap Chart")),
            ];
            for (view, label) in tabs {
                let selected = self.view == view;
                let text = RichText::new(label).size(12.0).color(if selected {
                    theme::ACCENT_DARK
                } else {
                    theme::TEXT_MUTED
                });
                let response = ui.add(
                    egui::Button::new(text)
                        .frame(false)
                        .min_size(Vec2::new(78.0, 24.0)),
                );
                if response.clicked() {
                    self.view = view;
                    if view == ContentView::Treemap {
                        self.ensure_treemap_root();
                    }
                }
                if selected {
                    let rect = response.rect;
                    ui.painter().line_segment(
                        [
                            egui::Pos2::new(rect.left() + 6.0, rect.bottom()),
                            egui::Pos2::new(rect.right() - 6.0, rect.bottom()),
                        ],
                        egui::Stroke::new(2.0_f32, theme::ACCENT),
                    );
                }
            }
            ui.separator();
            if let Some(tree) = &self.scan {
                let path = tree.root_path.to_string_lossy().into_owned();
                ui.label(
                    RichText::new(icons::ellipsis(&path, 90))
                        .size(11.5)
                        .color(theme::TEXT_MUTED),
                );
            }
        });
    }

    fn ensure_treemap_root(&mut self) {
        if self.treemap_root.is_none() {
            self.treemap_root = self.scan.as_ref().map(|tree| tree.root);
        }
    }

    fn welcome(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let width = ui.available_width();
        ui.allocate_ui_with_layout(
            Vec2::new(width, ui.available_height()),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space((ui.available_height() * 0.22).max(24.0));
                let (rect, _) = ui.allocate_exact_size(Vec2::new(96.0, 96.0), egui::Sense::hover());
                icons::draw(ui.painter(), icons::Icon::Drive, rect, theme::ACCENT);
                ui.add_space(14.0);
                ui.label(
                    RichText::new("DiskAgent")
                        .size(28.0)
                        .strong()
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new(self.t("Analyze disk usage of a folder"))
                        .size(13.0)
                        .color(theme::TEXT_MUTED),
                );
                ui.add_space(18.0);
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(self.t("Select Directory"))
                                .size(14.0)
                                .color(Color32::WHITE),
                        )
                        .fill(theme::ACCENT)
                        .min_size(Vec2::new(170.0, 34.0))
                        .rounding(egui::Rounding::same(4.0)),
                    )
                    .clicked()
                {
                    self.select_directory(ctx);
                }
                if let Some(last) = self.settings.last_dir.clone() {
                    ui.add_space(10.0);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("{}: {}", self.t("Rescan"), last))
                                    .size(11.5)
                                    .color(theme::ACCENT_DARK),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        let path = PathBuf::from(last);
                        self.start_scan_path(path, true, ctx);
                    }
                }
            },
        );
    }

    fn scan_progress(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let Some(job) = &self.job else { return };
        let progress = job.progress.clone();
        let files = progress.files.load(Ordering::Relaxed);
        let dirs = progress.dirs.load(Ordering::Relaxed);
        let bytes = progress.bytes.load(Ordering::Relaxed);
        let errors = progress.errors.load(Ordering::Relaxed);
        let current = progress
            .current
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default();
        let elapsed = job.started.elapsed().as_secs_f64();

        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.spinner();
            ui.add_space(8.0);
            ui.label(
                RichText::new(self.t("Scanning..."))
                    .size(16.0)
                    .strong()
                    .color(theme::TEXT),
            );
            ui.label(
                RichText::new(icons::ellipsis(&current, 110))
                    .size(11.5)
                    .color(theme::TEXT_MUTED),
            );
            ui.add_space(12.0);
            let unit = self.unit;
            let decimals = self.decimals;
            let lang = self.lang;
            ui.label(
                RichText::new(format!(
                    "{} {}  ·  {} {}  ·  {} {}  ·  {:.1}s",
                    format::format_count(files, lang),
                    self.t("files"),
                    format::format_count(dirs, lang),
                    self.t("folders"),
                    format::format_bytes(bytes, unit, decimals, lang),
                    "",
                    elapsed
                ))
                .size(12.0)
                .color(theme::TEXT_MUTED),
            );
            if errors > 0 {
                ui.label(
                    RichText::new(format!("{}: {}", self.t("Excluded"), errors))
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
            }
            ui.add_space(14.0);
            if ui
                .add(egui::Button::new(self.t("Stop Scan")).min_size(Vec2::new(120.0, 28.0)))
                .clicked()
            {
                self.stop_scan();
            }
            if progress.finished.load(Ordering::Relaxed) {
                ctx.request_repaint();
            }
        });
    }
}

pub(crate) fn percent_value(tree: &ScanTree, stats: &ViewStats, id: NodeId) -> f64 {
    if id == tree.root {
        return 100.0;
    }
    match tree.nodes[id].parent {
        Some(parent) => {
            let parent_size = stats.size[parent];
            if parent_size == 0 {
                0.0
            } else {
                stats.size[id] as f64 / parent_size as f64 * 100.0
            }
        }
        None => 100.0,
    }
}

fn csv_escape(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

impl eframe::App for DiskAgentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_scan();
        if self.close_requested {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        self.ensure_rows();
        self.handle_keys(ctx);

        egui::TopBottomPanel::top("ribbon")
            .frame(egui::Frame::none().fill(theme::RIBBON_BG))
            .show(ctx, |ui| {
                ribbon::show(ui, self);
            });

        if self.job.is_some() {
            ctx.request_repaint_after(Duration::from_millis(120));
        }

        egui::TopBottomPanel::bottom("statusbar")
            .exact_height(24.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(243, 243, 243))
                    .stroke(egui::Stroke::new(1.0_f32, theme::BORDER)),
            )
            .show(ctx, |ui| {
                statusbar::show(ui, self);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::WHITE))
            .show(ctx, |ui| {
                self.content_tabs(ui);
                ui.separator();
                if self.scan.is_some() {
                    match self.view {
                        ContentView::Details => crate::ui::details::show(ui, self),
                        ContentView::Treemap => treemap::show(ui, self),
                    }
                } else if self.job.is_some() {
                    self.scan_progress(ui, ctx);
                } else {
                    // The borrow of self created by scan_progress is gone here.
                }

                if self.scan.is_none() && self.job.is_none() {
                    self.welcome(ui, ctx);
                }
            });

        dialogs::show(ctx, self);
        self.maybe_save_settings(ctx);
    }
}

impl Drop for DiskAgentApp {
    fn drop(&mut self) {
        self.sync_settings();
        self.settings.save();
    }
}
