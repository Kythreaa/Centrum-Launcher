use super::{WindowManager, WindowInfo};
pub struct Generic;
impl WindowManager for Generic {
    fn get_window_list(&self) -> Vec<WindowInfo> { Vec::new() }
    fn focus_window(&self, _id: &str) {}
    fn logout(&self) {}
    fn center_cursor_or_window(&self) {}
    fn set_floating(&self) {}
}