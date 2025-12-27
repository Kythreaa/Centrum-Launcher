use crate::search::AppItem;
use regex::Regex;
use std::collections::{HashMap, HashSet};

pub fn check_web(query: &str, history: &HashMap<String, u32>, _url_re: &Regex, search_engine: &str) -> Vec<AppItem> {
    let mut results = Vec::new();
    let mut seen_names = HashSet::new();

    let (base_url, engine_name) = match search_engine {
        "startpage" => ("https://www.startpage.com/sp/search?q=", "StartPage"),
        "duckduckgo" | "ddg" => ("https://duckduckgo.com/?q=", "DuckDuckGo"),
        "google" => ("https://www.google.com/search?q=", "Google"),
        "bing" => ("https://www.bing.com/search?q=", "Bing"),
        "ecosia" => ("https://www.ecosia.org/search?q=", "Ecosia"),
        "qwant" => ("https://www.qwant.com/?q=", "Qwant"),
        _ => ("https://www.google.com/search?q=", "Google"),
    };

    if query.starts_with('?') {
        let q = query[1..].trim();
        if !q.is_empty() {
            let name = format!("Search {} for '{}'", engine_name, q);
            seen_names.insert(name.clone());
            let url = format!("{}{}", base_url, q.replace(' ', "+"));
            results.push(AppItem {
                name,
                exec: format!("xdg-open {}", url),
                terminal: false,
                icon: "\u{f002}".to_string(),
                desktop_id: "web".to_string(),
                system_icon: None,
            });
        }
        
        let sub = q.to_lowercase();
        let mut h: Vec<_> = history.iter()
            .filter(|(u, _)| u.contains("?q="))
            .filter(|(u, _)| sub.is_empty() || u.to_lowercase().contains(&sub))
            .collect();
        h.sort_by(|a, b| b.1.cmp(a.1).then_with(|| b.0.cmp(a.0)));
        
        for (u, _) in h.into_iter() {
            if results.len() >= 10 { break; }
            if let Some(pos) = u.find("?q=") {
                let q_val = &u[pos+3..];
                let display_q = q_val.split('&').next().unwrap_or(q_val).replace('+', " ");
                let name = format!("Search: {}", display_q);
                
                if seen_names.contains(&name) { continue; }
                seen_names.insert(name.clone());

                results.push(AppItem {
                    name,
                    exec: u.clone(),
                    terminal: false,
                    icon: "\u{f002}".to_string(),
                    desktop_id: "web".to_string(),
                    system_icon: None,
                });
            }
        }
        return results;
    }

    if query.starts_with(':') {
        let sub = query[1..].trim();
        
        if !sub.is_empty() {
            let name = format!("Open '{}'", sub);
            seen_names.insert(name.clone());
            let url = if sub.contains("://") { sub.to_string() } else { format!("https://{}", sub) };
            results.push(AppItem {
                name,
                exec: format!("xdg-open {}", url),
                terminal: false,
                icon: "\u{f059f}".to_string(),
                desktop_id: "web".to_string(),
                system_icon: None,
            });
        }

        let sub_low = sub.to_lowercase();
        let mut h: Vec<_> = history.iter()
            .filter(|(u, _)| u.starts_with("xdg-open http") && !u.contains("?q=")) 
            .filter(|(u, _)| sub_low.is_empty() || u.to_lowercase().contains(&sub_low))
            .collect();
        h.sort_by(|a, b| b.1.cmp(a.1));
        for (u, _) in h {
            let display = u.trim_start_matches("xdg-open ")
                           .trim_start_matches("https://")
                           .trim_start_matches("http://")
                           .to_string();
            
            let name = display.clone();
            if seen_names.contains(&name) { continue; }
            seen_names.insert(name.clone());

            results.push(AppItem {
                name,
                exec: u.clone(),
                terminal: false,
                icon: "\u{f059f}".to_string(),
                desktop_id: "web".to_string(),
                system_icon: None,
            });
        }
        return results;
    }

    if _url_re.is_match(query) || query.starts_with("http") {
        let url = if query.contains("://") { query.to_string() } else { format!("https://{}", query) };
        results.push(AppItem {
            name: "Open Link".to_string(),
            exec: format!("xdg-open {}", url),
            terminal: false,
            icon: "\u{f059f}".to_string(),
            desktop_id: "web".to_string(),
            system_icon: None,
        });
    }
    results
}