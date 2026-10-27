use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::format::Unit;
use crate::i18n::Lang;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub lang: String,
    pub unit: String,
    pub decimals: u8,
    pub show_size: bool,
    pub show_alloc: bool,
    pub show_files: bool,
    pub show_folders: bool,
    pub show_percent: bool,
    pub show_modified: bool,
    pub sort_key: String,
    pub sort_desc: bool,
    pub hide_empty: bool,
    pub ribbon_collapsed: bool,
    pub last_dir: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lang: "en".to_string(),
            unit: "auto".to_string(),
            decimals: 1,
            show_size: true,
            show_alloc: true,
            show_files: true,
            show_folders: true,
            show_percent: true,
            show_modified: true,
            sort_key: "size".to_string(),
            sort_desc: true,
            hide_empty: false,
            ribbon_collapsed: false,
            last_dir: None,
        }
    }
}

impl Settings {
    pub fn lang(&self) -> Lang {
        Lang::from_key(&self.lang)
    }

    pub fn unit(&self) -> Unit {
        Unit::from_key(&self.unit)
    }

    fn path() -> Option<PathBuf> {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())?;
        Some(base.join("DiskAgent").join("settings.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, text);
        }
    }
}
