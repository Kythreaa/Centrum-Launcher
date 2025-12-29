use crate::config::{load_custom_overrides, PowerOption};
use gio::prelude::*;
use gio::{AppInfo, DesktopAppInfo};
use glib::prelude::Cast;
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
#[derive(Clone, Debug)]
pub struct AppItem {
    pub name: String,
    pub exec: String,
    pub terminal: bool,
    pub icon: String,
    pub desktop_id: String,
    pub system_icon: Option<gio::Icon>,
}
pub fn get_apps(show_hidden: bool) -> Vec<AppItem> {
    let overrides = load_custom_overrides();
    AppInfo::all().into_iter()
        .filter(|app| {
            let id = app.id().map(|i| i.to_string()).unwrap_or_else(|| app.name().to_string());
            let custom_hidden = overrides.get(&id).and_then(|c| c.hidden);
            
            let is_effectively_hidden = match custom_hidden {
                Some(h) => h,
                None => !app.should_show(),
            };

            if show_hidden { is_effectively_hidden } else { !is_effectively_hidden }
        })
        .map(|app| {
            let id = app.id().map(|i| i.to_string()).unwrap_or_else(|| app.name().to_string());
            let mut name = app.name().to_string();
            let mut icon = None;
            let matched_custom = overrides.get(&id)
                .or_else(|| if id.ends_with(".desktop") { overrides.get(&id[..id.len() - 8]) } else { None })
                .or_else(|| overrides.get(&name));
            let mut sys_icon = app.icon();
            if let Some(custom) = matched_custom {
                if let Some(n) = &custom.name { name = n.clone(); }
                if let Some(i) = &custom.icon { icon = Some(i.clone()); }
                if let Some(si) = &custom.system_icon {
                    sys_icon = match si.as_str() {
                        "nerd" => None,
                        path if si.starts_with('/') => Some(gio::FileIcon::new(&gio::File::for_path(path)).upcast()),
                        themed => Some(gio::ThemedIcon::new(themed).upcast()),
                    };
                }
            }
            AppItem {
                icon: icon.unwrap_or_else(|| get_nerd_icon(&name).to_string()),
                name,
                exec: app.commandline().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                terminal: app.downcast::<DesktopAppInfo>().map(|d| d.boolean("Terminal")).unwrap_or(false),
                desktop_id: id,
                system_icon: sys_icon,
            }
        })
        .collect()
}
pub fn get_nerd_icon(name: &str) -> &'static str {
    static ICON_MAP: &[(&[&str], &str)] = &[
        (&["terminal", "kitty", "alacritty", "foot", "console"], "\u{f489}"),
        (&["firefox", "browser", "zen", "chromium", "chrome", "iron", "vivaldi", "epiphany"], "\u{f269}"),
        (&["code", "visual studio", "vscodium", "sublime", "atom", "jetbrains", "pycharm", "intellij", "clion"], "\u{f0a1e}"),
        (&["files", "nemo", "nautilus", "thunar", "dolphin", "pcmanfm", "index"], "\u{f024b}"),
        (&["discord", "vesktop", "element", "telegram", "whatsapp", "slack", "signal", "messenger"], "\u{f066f}"),
        (&["spotify", "music", "amberol", "rhythmbox", "audacious", "lollypop"], "\u{f1bc}"),
        (&["steam", "lutris", "heroic", "bottles", "prism", "minecraft", "game"], "\u{f04d3}"),
        (&["settings", "control center", "config", "tweak", "preferences"], "\u{eb51}"),
        (&["editor", "text", "gedit", "micro", "nvim", "vim", "leafpad", "mousepad"], "\u{f1782}"),
        (&["image", "pinta", "gimp", "inkscape", "krita", "darktable", "digikam", "photos"], "\u{f02e9}"),
        (&["video", "vlc", "mpv", "celluloid", "totem", "kdenlive", "obs"], "\u{f0fce}"),
        (&["mail", "thunderbird", "geary", "evolution", "mailspring"], "\u{f01ee}"),
        (&["calendar", "clock", "time"], "\u{f00ed}"),
        (&["calc", "calculator", "galculator"], "\u{f0a9a}"),
        (&["camera", "cheese", "guvcview"], "\u{f0100}"),
        (&["font", "character"], "\u{f0b36}"),
        (&["document", "word", "writer", "office", "libreoffice"], "\u{f0219}"),
        (&["pdf", "okular", "evince", "zathura"], "\u{f0226}"),
        (&["system monitor", "btop", "htop", "top", "usage"], "\u{f154d}"),
        (&["password", "bitwarden", "keepass", "auth"], "\u{f07f5}"),
        (&["download", "transmission", "qbittorrent", "deluge"], "\u{f01da}"),
        (&["note", "obsidian", "logseq", "joplin", "keep"], "\u{f178e}"),
    ];
    let n = name.to_lowercase();
    for (keywords, icon) in ICON_MAP {
        if keywords.iter().any(|k| n.contains(k)) {
            return icon;
        }
    }
    "\u{f003b}"
}
pub fn get_clipboard_history() -> Vec<AppItem> {
    let mut items = Vec::with_capacity(50);
    let mut seen_names = std::collections::HashSet::new();
    if let Ok(output) = Command::new("cliphist").arg("list").output() {
        let out = String::from_utf8_lossy(&output.stdout);
        for line in out.lines() {
            if line.is_empty() { continue; }
            if let Some((id_raw, c)) = line.split_once('\t') {
                let id = id_raw.trim();
                let mut d;
                let ic;
                if c.starts_with("[[ binary data ") && c.ends_with(" ]]") {
                    ic = "\u{f0976}".to_string();
                    let t: Vec<&str> = c[15..c.len()-3].split_whitespace().collect();
                    if t.len() >= 3 {
                        d = format!("{} {} ({})", t[t.len()-2].to_uppercase(), t[t.len()-1], t[..t.len()-2].join(" "));
                    } else {
                        d = c.to_string();
                    }
                } else if c.starts_with("<meta http-equiv=\"content-type\" content=\"text/html;") {
                    continue;
                } else if c.starts_with('/') {
                    let p = std::path::Path::new(c);
                    if ["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg"].contains(&p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str()) {
                        ic = "\u{f021f}".to_string();
                        d = format!("[FILE] {}", p.file_name().and_then(|n| n.to_str()).unwrap_or(c));
                    } else {
                        ic = "\u{f014d}".to_string();
                        d = c.chars().filter(|ch| (!ch.is_control() && *ch != '\u{FFFD}') || *ch == '\n').collect();
                    }
                } else {
                    ic = "\u{f014d}".to_string();
                    d = c.chars().filter(|ch| (!ch.is_control() && *ch != '\u{FFFD}') || *ch == '\n').collect();
                }
                d = d.trim().to_string();
                if d.is_empty() || seen_names.contains(&d) { continue; }
                seen_names.insert(d.clone());
                items.push(AppItem {
                    name: d,
                    exec: format!("CLIPBOARD_SET:{}", id),
                    terminal: false,
                    icon: ic,
                    desktop_id: "clipboard".to_string(),
                    system_icon: None,
                });
            }
        }
    }
    items
}
static LAST_CALC: LazyLock<Mutex<(Instant, String, Option<(String, String)>)>> = LazyLock::new(|| {
    Mutex::new((Instant::now() - Duration::from_secs(1), String::new(), None))
});
pub fn check_calc(query: &str) -> Option<AppItem> {
    if query.starts_with('/') || query.starts_with(':') || query.starts_with('?') || query.len() < 2 || !query.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut cache = match LAST_CALC.lock() {
        Ok(guard) => guard,
        Err(p) => {
            let mut guard = p.into_inner();
            *guard = (Instant::now(), String::new(), None);
            guard
        }
    };
    if cache.1 == query {
        return cache.2.as_ref().map(|(n, e)| AppItem {
            name: n.clone(),
            exec: e.clone(),
            terminal: false,
            icon: "\u{f00ec}".to_string(),
            desktop_id: "calc".to_string(),
            system_icon: None,
        });
    }
    let now = Instant::now();
    if now.duration_since(cache.0) < Duration::from_millis(50) {
        return cache.2.as_ref().map(|(n, e)| AppItem {
            name: n.clone(),
            exec: e.clone(),
            terminal: false,
            icon: "\u{f00ec}".to_string(),
            desktop_id: "calc".to_string(),
            system_icon: None,
        });
    }
    cache.0 = now;
    cache.1 = query.to_string();
    let mut ci = query.replace(" of ", " * ");
    if query.contains('%') && !["*", "/", "+", "-"].iter().any(|&op| ci.contains(op)) {
        ci = ci.split_whitespace().collect::<Vec<_>>().join(" * ");
    }
    if let Ok(o) = Command::new("qalc").arg("-t").arg(&ci).output() {
        let r = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !r.is_empty() && r != query && r != "0" && !r.contains("rem(") {
            let rf = r.replace(" + ", "\n");
            let (mut display, mut copy_val) = (rf.clone(), r.clone());
            if r.contains('E') {
                if let Ok(of) = Command::new("qalc").arg("-t").arg("-s").arg("scientific_notation off").arg(&ci).output() {
                    let ff = String::from_utf8_lossy(&of.stdout).trim().to_string();
                    if !ff.is_empty() && ff != r && ff.len() <= 100 {
                        display = format!("{}\n({})", rf, ff);
                        copy_val = ff;
                    }
                }
            }
            let final_exec = format!("COPY:{}", copy_val);
            cache.2 = Some((display.clone(), final_exec.clone()));
            return Some(AppItem {
                name: display,
                exec: final_exec,
                terminal: false,
                icon: "\u{f00ec}".to_string(),
                desktop_id: "calc".to_string(),
                system_icon: None,
            });
        }
    }
    cache.2 = None;
    None
}
pub fn check_system_commands(query: &str, options: &[PowerOption]) -> Vec<AppItem> {
    let mut results = Vec::new();
    let q = query.to_lowercase();
    for opt in options {
        let name = match opt.class.as_str() {
            "shutdown-btn" => "Shutdown",
            "reboot-btn" => "Reboot",
            "logout-btn" => "Log Out",
            "theme-btn" => "Toggle Theme",
            _ => continue,
        };
        if name.to_lowercase() == q {
             results.push(AppItem {
                name: name.to_string(),
                exec: opt.command.clone(),
                terminal: false,
                icon: opt.icon.clone(),
                desktop_id: "system".to_string(),
                system_icon: None,
            });
        }
    }
    results
}