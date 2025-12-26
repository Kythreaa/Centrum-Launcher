use std::process::Command;
use super::{WindowManager, WindowInfo};
pub struct Niri;
impl WindowManager for Niri {
    fn get_window_list(&self) -> Vec<WindowInfo> {
        let mut list = Vec::new();
        if let Ok(output) = Command::new("niri").arg("msg").arg("--json").arg("windows").output() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(windows) = json.as_array() {
                    for win in windows {
                        if let Some(id) = win.get("id").and_then(|i| i.as_u64()) {
                            let title = win.get("title").and_then(|t| t.as_str()).unwrap_or_default();
                            let app_id = win.get("app_id").and_then(|a| a.as_str()).unwrap_or_default();
                            list.push(WindowInfo {
                                id: id.to_string(),
                                title: title.to_string(),
                                app_id: app_id.to_string(),
                            });
                        }
                    }
                }
            }
        }
        list
    }
    fn focus_window(&self, id: &str) {
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("focus-window")
            .arg("--id")
            .arg(id)
            .spawn();
    }
    fn logout(&self) {
        let _ = Command::new("niri").arg("msg").arg("action").arg("quit").spawn();
    }
    fn center_cursor_or_window(&self) {
        let _ = Command::new("niri").arg("msg").arg("action").arg("center-column").spawn();
    }
}
