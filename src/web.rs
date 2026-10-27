use std::path::{Component, Path};

use crate::i18n::Lang;

/// Builds the natural-language question that is sent to Google.
///
/// Only the name and a short, privacy-friendly location hint are used. The
/// full local path (and the Windows user name) is intentionally not sent.
pub fn explain_query(name: &str, is_dir: bool, path: &Path, lang: Lang) -> String {
    // For files, the containing folder is more useful than the file path itself.
    let location_path = if is_dir {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    let location = location_hint(location_path);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");

    match lang {
        Lang::Id => {
            let mut query = if is_dir {
                format!("Apa fungsi folder \"{name}\"")
            } else {
                format!("Apa fungsi file \"{name}\"")
            };
            if !location.is_empty() {
                query.push_str(&format!(" di \"{location}\""));
            }
            if !is_dir && !extension.is_empty() {
                query.push_str(&format!(" (ekstensi .{extension})"));
            }
            query.push_str(" dan apakah aman dihapus?");
            query
        }
        Lang::En => {
            let mut query = if is_dir {
                format!("What is the folder \"{name}\"")
            } else {
                format!("What is the file \"{name}\"")
            };
            if !location.is_empty() {
                query.push_str(&format!(" in \"{location}\""));
            }
            if !is_dir && !extension.is_empty() {
                query.push_str(&format!(" (extension .{extension})"));
            }
            query.push_str(" and is it safe to delete?");
            query
        }
    }
}

/// Returns a short location hint for the query.
///
/// Examples:
/// - `C:\Users\Budi\AppData\Local\Temp` -> `AppData\Local\Temp`
/// - `D:\CODE\DiskAgent\src` -> `CODE\DiskAgent\src`
/// - `C:\Users\Budi\Documents` -> `Users\Documents`
fn location_hint(path: &Path) -> String {
    let mut parts: Vec<String> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();

    // Drop the Windows profile name, but keep the meaningful marker around it.
    let mut index = 0;
    while index + 1 < parts.len() {
        if parts[index].eq_ignore_ascii_case("users") {
            let candidate = parts[index + 1].to_ascii_lowercase();
            let shared = matches!(
                candidate.as_str(),
                "public" | "default" | "default user" | "all users"
            );
            if !shared {
                parts.remove(index + 1);
                break;
            }
        }
        index += 1;
    }

    let start = parts.len().saturating_sub(3);
    parts[start..].join("\\")
}

pub fn google_search_url(query: &str, lang: Lang) -> String {
    let hl = match lang {
        Lang::Id => "id",
        Lang::En => "en",
    };
    format!(
        "https://www.google.com/search?q={}&hl={hl}",
        percent_encode(query)
    )
}

fn percent_encode(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                output.push(byte as char);
            }
            _ => output.push_str(&format!("%{byte:02X}")),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_hint_redacts_the_profile_name() {
        let path = Path::new(r"C:\Users\Budi\AppData\Local\Temp");
        assert_eq!(location_hint(path), r"AppData\Local\Temp");

        let documents = Path::new(r"C:\Users\Budi\Documents");
        assert_eq!(location_hint(documents), r"Users\Documents");

        let project = Path::new(r"D:\CODE\DiskAgent\src");
        assert_eq!(location_hint(project), r"CODE\DiskAgent\src");
    }

    #[test]
    fn query_mentions_the_folder_and_safety_in_both_languages() {
        let path = Path::new(r"D:\CODE\DiskAgent\src");
        let id = explain_query("src", true, path, Lang::Id);
        assert!(id.contains("Apa fungsi folder"));
        assert!(id.contains("src"));
        assert!(id.contains("aman dihapus"));

        let en = explain_query("src", true, path, Lang::En);
        assert!(en.contains("What is the folder"));
        assert!(en.contains("safe to delete"));

        let file = explain_query("main.rs", false, &path.join("main.rs"), Lang::Id);
        assert!(file.contains("Apa fungsi file \"main.rs\""));
        assert!(file.contains("CODE\\DiskAgent\\src"));
        assert!(file.contains("ekstensi .rs"));
        assert!(!file.contains("src\\main.rs"));
    }

    #[test]
    fn google_url_escapes_spaces_quotes_and_backslashes() {
        let url = google_search_url(r#"Apa fungsi folder "src" di "CODE\DiskAgent"?"#, Lang::Id);
        assert!(url.starts_with("https://www.google.com/search?q="));
        assert!(url.contains("%20"));
        assert!(url.contains("%22"));
        assert!(url.contains("%5C"));
        assert!(url.ends_with("&hl=id"));
    }
}
