use crate::search::AppItem;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
static HOME_PATH: LazyLock<PathBuf> = LazyLock::new(|| std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default());
static HOME_STR: LazyLock<String> = LazyLock::new(|| HOME_PATH.to_string_lossy().to_string());
pub fn check_files(query: &str, _matcher: &SkimMatcherV2, _existing_items: &[AppItem]) -> Vec<AppItem> {
    if !query.starts_with('/') && !query.starts_with('~') { return Vec::new(); }
    
    if query == "/" {
        return list_dir(&*HOME_PATH);
    }

    let full_path = if query.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(query.replacen('~', &home, 1))
    } else if query.starts_with('/') {
        let p = PathBuf::from(query);
        if p.exists() {
            p
        } else {
            let mut hp = HOME_PATH.clone();
            hp.push(&query[1..]);
            hp
        }
    } else {
        PathBuf::from(query)
    };

    if full_path.exists() {
        if full_path.is_dir() && query.ends_with('/') {
            return list_dir(&full_path);
        }

        let path_str = full_path.to_string_lossy().to_string();
        let display_name = if path_str.starts_with(HOME_STR.as_str()) {
            let sub = &path_str[HOME_STR.len()..];
            if sub.starts_with('/') { sub.to_string() } else { format!("/{}", sub) }
        } else { path_str.clone() };
        
        return vec![AppItem {
            icon: if full_path.is_dir() { "\u{f024b}".to_string() } else { "\u{f0214}".to_string() },
            name: display_name,
            exec: format!("OPEN_PATH:{}", path_str),
            terminal: false,
            desktop_id: "file".to_string(),
            system_icon: None,
        }];
    }

    let pattern = &query[1..];
    if pattern.is_empty() { return list_dir(&*HOME_PATH); }
    if pattern.ends_with('/') {
        let mut dir_path = HOME_PATH.clone();
        dir_path.push(pattern);
        if dir_path.is_dir() { return list_dir(&dir_path); }
    }
    let Ok(o) = Command::new("fd")
        .arg("--hidden").arg("--no-ignore").arg("--max-results").arg("200")
        .arg("--absolute-path").arg("--color=never").arg(pattern).arg(&*HOME_PATH)
        .output() else { return Vec::new(); };
    let out_str = String::from_utf8_lossy(&o.stdout);
    let query_lower = pattern.to_lowercase();
    let pattern_has_slash = pattern.contains('/');
    let mut results: Vec<_> = out_str.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let path_str = line.trim().to_string();
            let path = PathBuf::from(&path_str);
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            let (is_exact, starts_with) = if pattern_has_slash {
                let rel = path.strip_prefix(&*HOME_PATH).map(|p| p.to_string_lossy().to_lowercase()).unwrap_or_default();
                (rel == query_lower, rel.starts_with(&query_lower))
            } else {
                (file_name == query_lower, file_name.starts_with(&query_lower))
            };
            let is_hidden = path.components().any(|c| c.as_os_str().to_string_lossy().starts_with('.') && c.as_os_str() != ".");
            let depth = path.components().count();
            let display_name = if path_str.starts_with(HOME_STR.as_str()) {
                let sub = &path_str[HOME_STR.len()..];
                if sub.starts_with('/') { sub.to_string() } else { format!("/{}", sub) }
            } else { path_str.clone() };
            let item = AppItem {
                icon: if path.is_dir() { "\u{f024b}".to_string() } else { "\u{f0214}".to_string() },
                name: display_name,
                exec: format!("OPEN_PATH:{}", path_str),
                terminal: false,
                desktop_id: "file".to_string(),
                system_icon: None,
            };
            (item, is_exact, starts_with, is_hidden, depth, path_str)
        })
        .collect();
    results.sort_by(|a, b| {
        b.1.cmp(&a.1) 
            .then_with(|| b.2.cmp(&a.2)) 
            .then_with(|| a.3.cmp(&b.3)) 
            .then_with(|| a.4.cmp(&b.4)) 
            .then_with(|| a.5.cmp(&b.5)) 
    });
    results.into_iter().take(50).map(|r| r.0).collect()
}
fn list_dir(path: &Path) -> Vec<AppItem> {
    let Ok(entries) = std::fs::read_dir(path) else { return Vec::new(); };
    let mut results: Vec<_> = entries.filter_map(|e| e.ok())
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .map(|entry| {
            let p = entry.path();
            let path_str = p.to_string_lossy().to_string();
            let display_name = if path_str.starts_with(HOME_STR.as_str()) {
                let sub = &path_str[HOME_STR.len()..];
                if sub.starts_with('/') { sub.to_string() } else { format!("/{}", sub) }
            } else { entry.file_name().to_string_lossy().to_string() };
            AppItem {
                icon: if p.is_dir() { "\u{f024b}".to_string() } else { "\u{f0214}".to_string() },
                name: display_name,
                exec: format!("OPEN_PATH:{}", path_str),
                terminal: false,
                desktop_id: "file".to_string(),
                system_icon: None,
            }
        })
        .collect();
    results.sort_by(|a, b| {
        let a_is_dir = a.icon == "\u{f024b}";
        let b_is_dir = b.icon == "\u{f024b}";
        b_is_dir.cmp(&a_is_dir).then_with(|| a.name.cmp(&b.name))
    });
    results
}