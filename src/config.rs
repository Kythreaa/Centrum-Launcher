use directories::ProjectDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
pub const DEFAULT_CONFIG_CSS: &str = include_str!("resources/config.css");
pub const DEFAULT_DARK_CSS: &str = include_str!("resources/dark.css");
pub const DEFAULT_LIGHT_CSS: &str = include_str!("resources/light.css");
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowState {
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub history: HashMap<String, u32>,
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default = "default_true")]
    pub show_hotkeys: bool,
}
fn default_true() -> bool { true }
impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 600,
            height: 450,
            x: -1,
            y: -1,
            history: HashMap::new(),
            show_hidden: false,
            show_hotkeys: true,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CustomApp {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub system_icon: Option<String>,
    pub hidden: Option<bool>,
}
#[derive(Clone, Debug)]
pub struct PowerOption {
    pub icon: String,
    pub command: String,
    pub class: String,
}
#[derive(Clone, Debug)]
pub struct Hotkey {
    pub key: String,
    pub mods: gtk4::gdk::ModifierType,
}
#[derive(Clone, Debug)]
pub struct ScrollSettings {
    pub duration: f64,
    pub interval: u64,
    pub easing: String,
    pub top_padding: f64,
    pub bottom_padding: f64,
}
#[derive(Clone, Debug)]
pub struct ThemeConfig {
    pub power_options: Vec<PowerOption>,
    pub text_align: f32,
    pub icon_mode: String,
    pub icon_effect: String,
    pub icon_position: String,
    pub search_engine: String,
    pub terminal: String,
    pub focus_on_launch: bool,
    pub scroll: ScrollSettings,
    pub hotkeys: HashMap<String, Hotkey>,
}
impl ThemeConfig {
    pub fn load() -> Self {
        let config_path = get_config_dir().join("config.css");
        let mut css = fs::read_to_string(config_path).unwrap_or_else(|_| DEFAULT_CONFIG_CSS.to_string());
        let comment_re = Regex::new(r"(?s)/\*.*?\*/").unwrap();
        css = comment_re.replace_all(&css, "").to_string();
        static BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\.([\w-]+)\s*\{([^}]*)\}").unwrap());
        static ICON_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-icon:\s*\"([^\"]+)\""#).unwrap());
        static CMD_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-command:\s*\"([^\"]+)\""#).unwrap());
        static ALIGN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"text-align:\s*(\w+);"#).unwrap());
        static MODE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-icon-mode:\s*\\?\"([^\\";]+)\\?\""#).unwrap());
        static EFF_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-icon-effect:\s*(\w+);"#).unwrap());
        static POS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-icon-position:\s*\"([^\"]+)\""#).unwrap());
        static ENGINE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-search-engine:\s*\"([^\"]+)\""#).unwrap());
        static TERM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-(gtk|centrum)-terminal:\s*\"([^\"]+)\""#).unwrap());
        static FOCUS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-focus-on-launch:\s*\"([^\"]+)\""#).unwrap());
        static DUR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-scroll-duration:\s*(\d+)ms"#).unwrap());
        static INT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-scroll-interval:\s*(\d+)ms"#).unwrap());
        static EAS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-scroll-easing:\s*\"([^\"]+)\""#).unwrap());
        static TOP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-scroll-padding-top:\s*(\d+)px"#).unwrap());
        static BOT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-scroll-padding-bottom:\s*(\d+)px"#).unwrap());
        static COMBO_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"-gtk-combo:\s*\"([^\"]+)\""#).unwrap());
        let mut power_options = Vec::new();
        let mut hotkeys = HashMap::new();
        for cap in BLOCK_RE.captures_iter(&css) {
            let class = cap[1].to_string();
            let block = &cap[2];
            if let (Some(icon_cap), Some(cmd_cap)) = (ICON_RE.captures(block), CMD_RE.captures(block)) {
                power_options.push(PowerOption {
                    icon: icon_cap[1].to_string(),
                    command: cmd_cap[1].to_string(),
                    class: class.clone(),
                });
            }
            if class.starts_with("hk-") {
                if let Some(combo_cap) = COMBO_RE.captures(block) {
                    let combo = combo_cap[1].to_string();
                    let parts: Vec<&str> = combo.split('+').collect();
                    let mut mods = gtk4::gdk::ModifierType::empty();
                    let mut key = String::new();
                    for (i, part) in parts.iter().enumerate() {
                        let p = part.to_lowercase().trim().to_string();
                        if i == parts.len() - 1 {
                            key = p;
                        } else {
                            match p.as_str() {
                                "ctrl" => mods |= gtk4::gdk::ModifierType::CONTROL_MASK,
                                "alt" => mods |= gtk4::gdk::ModifierType::ALT_MASK,
                                "shift" => mods |= gtk4::gdk::ModifierType::SHIFT_MASK,
                                "mod" | "super" | "meta" | "win" => mods |= gtk4::gdk::ModifierType::SUPER_MASK,
                                _ => {}
                            }
                        }
                    }
                    if !key.is_empty() {
                        hotkeys.insert(class, Hotkey { key, mods });
                    }
                }
            }
        }
        if power_options.is_empty() {
            power_options = vec![
                PowerOption { icon: "\u{f011}".to_string(), command: "systemctl poweroff".to_string(), class: "shutdown-btn".to_string() },
                PowerOption { icon: "\u{f0e2}".to_string(), command: "systemctl reboot".to_string(), class: "reboot-btn".to_string() },
                PowerOption { icon: "\u{f08b}".to_string(), command: "loginctl terminate-user $USER".to_string(), class: "logout-btn".to_string() },
            ];
        }
        let text_align = ALIGN_RE.captures(&css).map(|c| match &c[1] { "left" => 0.0, "right" => 1.0, _ => 0.5 }).unwrap_or(0.5);
        let icon_mode = MODE_RE.captures(&css).map(|c| c[1].to_string()).unwrap_or_else(|| "nerd".to_string());
        let icon_effect = EFF_RE.captures(&css).map(|c| c[1].to_string()).unwrap_or_else(|| "none".to_string());
        let icon_position = POS_RE.captures(&css).map(|c| c[1].to_string()).unwrap_or_else(|| "fixed".to_string());
        let search_engine = ENGINE_RE.captures(&css).map(|c| c[1].to_lowercase()).unwrap_or_else(|| "google".to_string());
        let terminal = TERM_RE.captures(&css).map(|c| c[2].to_string()).unwrap_or_else(|| crate::utils::detect_terminal());
        let focus_on_launch = FOCUS_RE.captures(&css).map(|c| &c[1] == "true").unwrap_or(true);
        let scroll = ScrollSettings {
            duration: DUR_RE.captures(&css).and_then(|c| c[1].parse().ok()).unwrap_or(120.0),
            interval: INT_RE.captures(&css).and_then(|c| c[1].parse().ok()).unwrap_or(8),
            easing: EAS_RE.captures(&css).map(|c| c[1].to_string()).unwrap_or_else(|| "cubic".to_string()),
            top_padding: TOP_RE.captures(&css).and_then(|c| c[1].parse().ok()).unwrap_or(80.0),
            bottom_padding: BOT_RE.captures(&css).and_then(|c| c[1].parse().ok()).unwrap_or(200.0),
        };
        Self {
            power_options,
            text_align,
            icon_mode,
            icon_effect,
            icon_position,
            search_engine,
            terminal,
            focus_on_launch,
            scroll,
            hotkeys,
        }
    }
}
pub fn get_config_dir() -> PathBuf {
    ProjectDirs::from("org", "centrum", "centrum-launcher")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".config").join("centrum-launcher")
        })
}
pub fn ensure_config_files() {
    let dir = get_config_dir();
    let _ = fs::create_dir_all(&dir);
    
    let config_path = dir.join("config.css");
    if !config_path.exists() {
        let term = crate::utils::detect_terminal();
        let content = DEFAULT_CONFIG_CSS.replace(
            "outline: none;",
            &format!("-centrum-terminal: \"{}\";\n    outline: none;", term)
        );
        let _ = fs::write(config_path, content);
    }

    let other_files = [
        ("dark.css", DEFAULT_DARK_CSS),
        ("light.css", DEFAULT_LIGHT_CSS),
    ];
    for (name, content) in other_files {
        let path = dir.join(name);
        if !path.exists() {
            let _ = fs::write(path, content);
        }
    }
}
pub fn load_custom_overrides() -> HashMap<String, CustomApp> {
    let path = get_config_dir().join("custom_apps.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}
pub fn save_custom_overrides(overrides: &HashMap<String, CustomApp>) {
    let dir = get_config_dir();
    let _ = fs::create_dir_all(&dir);
    if let Ok(content) = serde_json::to_string_pretty(overrides) {
        let _ = fs::write(dir.join("custom_apps.json"), content);
    }
}
pub fn load_state() -> WindowState {
    let path = get_config_dir().join("state.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}
pub fn save_state(state: &WindowState) {
    let dir = get_config_dir();
    let _ = fs::create_dir_all(&dir);
    if let Ok(content) = serde_json::to_string_pretty(state) {
        let _ = fs::write(dir.join("state.json"), content);
    }
}
pub fn save_icon_mode(mode: &str) {
    let config_path = get_config_dir().join("config.css");
    if let Ok(css) = std::fs::read_to_string(&config_path) {
        let re = Regex::new(r#"-gtk-icon-mode:\s*\\?\"[^\";]*\\?\";"#).unwrap();
        let new_line = format!(r#"-gtk-icon-mode: "{}";"#, mode);
        let updated = re.replace(&css, new_line.as_str()).to_string();
        let _ = std::fs::write(config_path, updated);
    }
}
