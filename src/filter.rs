use std::time::{Duration, SystemTime};

use crate::model::ScanTree;

#[derive(Clone, Debug)]
pub struct Filter {
    /// Semicolon/comma separated wildcard masks.
    pub include: String,
    /// Semicolon/comma separated wildcard masks.
    pub exclude: String,
    /// Minimum file size in bytes (0 = off).
    pub min_size: u64,
    /// Maximum file age in days (0 = off).
    pub max_age_days: u32,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            include: String::new(),
            exclude: String::new(),
            min_size: 0,
            max_age_days: 0,
        }
    }
}

impl Filter {
    pub fn is_active(&self) -> bool {
        !self.include.trim().is_empty()
            || !self.exclude.trim().is_empty()
            || self.min_size > 0
            || self.max_age_days > 0
    }

    pub fn clear(&mut self) {
        *self = Filter::default();
    }

    pub fn matches(
        &self,
        name: &str,
        size: u64,
        modified: Option<SystemTime>,
        now: SystemTime,
    ) -> bool {
        if self.min_size > 0 && size < self.min_size {
            return false;
        }
        if self.max_age_days > 0 {
            let max_age = Duration::from_secs(self.max_age_days as u64 * 86_400);
            let age = modified
                .and_then(|m| now.duration_since(m).ok())
                .unwrap_or(Duration::ZERO);
            if age > max_age {
                return false;
            }
        }
        if !mask_list_matches(&self.include, name, true) {
            return false;
        }
        if mask_list_matches(&self.exclude, name, false) {
            return false;
        }
        true
    }
}

fn mask_list_matches(list: &str, name: &str, empty_matches: bool) -> bool {
    let masks: Vec<&str> = list
        .split([';', ','])
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .collect();
    if masks.is_empty() {
        return empty_matches;
    }
    masks.iter().any(|m| wildcard_match(m, name))
}

/// Case-insensitive `*` / `?` wildcard matching.
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.to_lowercase().chars().collect();
    let t: Vec<char> = text.to_lowercase().chars().collect();

    let (mut pi, mut ti) = (0usize, 0usize);
    let (mut star, mut star_ti): (Option<usize>, usize) = (None, 0);

    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            star_ti = ti;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Filtered per-node statistics used for display, sorting and percentages.
pub struct ViewStats {
    pub size: Vec<u64>,
    pub alloc: Vec<u64>,
    pub files: Vec<u64>,
    pub folders: Vec<u64>,
    pub visible: Vec<bool>,
    pub excluded_files: u64,
    pub matched_files: u64,
    pub total_files: u64,
}

pub fn compute(tree: &ScanTree, filter: &Filter, hide_empty: bool) -> ViewStats {
    let count = tree.nodes.len();
    let mut stats = ViewStats {
        size: vec![0; count],
        alloc: vec![0; count],
        files: vec![0; count],
        folders: vec![0; count],
        visible: vec![false; count],
        excluded_files: 0,
        matched_files: 0,
        total_files: 0,
    };

    let active = filter.is_active();
    let now = SystemTime::now();

    for i in (0..count).rev() {
        let node = &tree.nodes[i];
        if node.is_dir {
            let has_content = stats.files[i] > 0 || stats.folders[i] > 0;
            let visible = if i == tree.root {
                true
            } else if active || hide_empty {
                has_content
            } else {
                true
            };
            stats.visible[i] = visible;
            if !visible {
                stats.size[i] = 0;
                stats.alloc[i] = 0;
                stats.files[i] = 0;
                stats.folders[i] = 0;
            }
        } else {
            stats.total_files += 1;
            let matched = !active
                || filter.matches(&node.name, node.size, node.modified, now);
            if matched {
                stats.size[i] = node.size;
                stats.alloc[i] = node.alloc;
                stats.files[i] = 1;
                stats.visible[i] = true;
                stats.matched_files += 1;
            } else {
                stats.excluded_files += 1;
            }
        }

        if i == tree.root {
            continue;
        }
        if !stats.visible[i] {
            continue;
        }
        if let Some(parent) = node.parent {
            stats.size[parent] += stats.size[i];
            stats.alloc[parent] += stats.alloc[i];
            stats.files[parent] += stats.files[i];
            if node.is_dir {
                stats.folders[parent] += stats.folders[i] + 1;
            }
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_basics() {
        assert!(wildcard_match("*.rs", "main.rs"));
        assert!(wildcard_match("REPORT?.DOCX", "report1.docx"));
        assert!(!wildcard_match("*.rs", "main.txt"));
        assert!(wildcard_match("*", "anything"));
    }
}
