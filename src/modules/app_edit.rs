use crate::config::{load_custom_overrides, save_custom_overrides};
use crate::search::{get_apps};
use crate::ui::{EditingMode, LauncherState};
use crate::modules::app_launcher::update_list_view;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
pub fn handle_app_edit(
    state: &Rc<RefCell<LauncherState>>,
    window: &gtk4::ApplicationWindow,
    container: &gtk4::Box,
    edit_entry: &gtk4::Entry,
    popup: &gtk4::Box,
) {
    let mut sh = state.borrow_mut();
    let val = edit_entry.text().to_string();
    if let Some(id) = sh.editing_id.clone() {
        let mut ovr = load_custom_overrides();
        let ent = ovr.entry(id).or_default();
        match sh.editing_mode { 
            EditingMode::Rename => ent.name = if val.is_empty() { None } else { Some(val) }, 
            EditingMode::Icon => {
                let mode = &sh.theme_config.icon_mode;
                let field_val = if val.is_empty() { None } else { Some(val) };
                if mode == "system" {
                    ent.system_icon = field_val;
                } else {
                    ent.icon = field_val;
                }
            },
            _ => {} 
        }
        save_custom_overrides(&ovr);
        sh.all_apps = get_apps(sh.show_hidden); 
        
        let history = sh.history.clone();
        sh.all_apps.sort_by(|a, b| {
            history.get(&b.exec).unwrap_or(&0).cmp(history.get(&a.exec).unwrap_or(&0))
        });
        
        sh.filtered_apps = sh.all_apps.clone();
        sh.editing_mode = EditingMode::None; 
        sh.editing_id = None;
        let config = sh.theme_config.clone();
        update_list_view(container, &sh.filtered_apps, sh.app_index, window, state, &config);
        popup.set_visible(false);
    }
}
