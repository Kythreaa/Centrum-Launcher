use std::process::Command;
use super::{WindowManager, WindowInfo};
pub struct Hyprland;
impl WindowManager for Hyprland {
    fn get_window_list(&self) -> Vec<WindowInfo> {
        let mut list = Vec::new();
        if let Ok(output) = Command::new("hyprctl").arg("clients").arg("-j").output() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(windows) = json.as_array() {
                    for win in windows {
                        let address = win.get("address").and_then(|a| a.as_str()).unwrap_or_default();
                        let title = win.get("title").and_then(|t| t.as_str()).unwrap_or_default();
                        let class = win.get("class").and_then(|c| c.as_str()).unwrap_or_default();
                        list.push(WindowInfo {
                            id: address.to_string(),
                            title: title.to_string(),
                            app_id: class.to_string(),
                        });
                    }
                }
            }
        }
        list
    }
    fn focus_window(&self, id: &str) {
        let _ = Command::new("hyprctl")
            .arg("dispatch")
            .arg("focuswindow")
            .arg(format!("address:{}", id))
            .spawn();
    }
    fn logout(&self) {
        let _ = Command::new("hyprctl").arg("dispatch").arg("exit").spawn();
    }
    fn center_cursor_or_window(&self) {}
}
