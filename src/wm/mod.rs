use std::env;
pub mod niri;
pub mod hyprland;
pub mod generic;
pub trait WindowManager {
    fn get_window_list(&self) -> Vec<WindowInfo>;
    fn focus_window(&self, id: &str);
    #[allow(dead_code)]
    fn logout(&self);
    fn center_cursor_or_window(&self);
    fn set_floating(&self);
}
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub app_id: String,
}
pub fn detect() -> Box<dyn WindowManager> {
    let xdg_current = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    let session = env::var("DESKTOP_SESSION").unwrap_or_default().to_lowercase();
    let hypr_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
    if xdg_current.contains("niri") || session.contains("niri") {
        Box::new(niri::Niri)
    } else if xdg_current.contains("hyprland") || hypr_signature {
        Box::new(hyprland::Hyprland)
    } else {
        Box::new(generic::Generic)
    }
}