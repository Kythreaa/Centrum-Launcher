use std::process::Command;
use std::env;
use super::{WindowManager, WindowInfo};
pub struct Generic;
impl WindowManager for Generic {
    fn get_window_list(&self) -> Vec<WindowInfo> {
        Vec::new() 
    }
    fn focus_window(&self, _id: &str) {}
    fn logout(&self) {
        if let Ok(user) = env::var("USER") {
            let _ = Command::new("loginctl").arg("terminate-user").arg(user).spawn();
        }
    }
    fn center_cursor_or_window(&self) {}
}
