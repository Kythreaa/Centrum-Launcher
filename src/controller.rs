use crate::search::{check_calc, check_system_commands, get_apps, AppItem};
use crate::ui::{create_hotkeys_window, update_visuals, EditingMode, LauncherState, SelectionMode};
use crate::modules::app_launcher::update_list_view;
use crate::modules::file_search::check_files;
use crate::modules::web_search::check_web;
use crate::modules::app_edit::handle_app_edit;
use crate::utils::{hsv_to_rgb, rgb_to_hsv};
use crate::modules::color_picker::update_color_ui;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box, EventControllerKey, PropagationPhase, ScrolledWindow};
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::Command;
use std::rc::Rc;
use std::sync::LazyLock;
pub fn setup_search_logic(
    entry: &gtk4::Entry,
    container: &Box,
    scroll: &ScrolledWindow,
    p_box: &Box,
    window: &ApplicationWindow,
    state: &Rc<RefCell<LauncherState>>,
) {
    let matcher = SkimMatcherV2::default();
    static URL_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^(https?://)?([\w-]+\.)+[\w-]+(/[-￿\w\-. /?%&=]*)?$").unwrap()
    });
    let (container_c, scroll_c, p_box_c, state_c, window_c) = (
        container.clone(),
        scroll.clone(),
        p_box.clone(),
        state.clone(),
        window.clone(),
    );
    entry.connect_changed(move |e| {
        let text = e.text().to_string();
        {
            let sh_test = match state_c.try_borrow() { Ok(s) => s, Err(_) => return };
            if sh_test.is_syncing || sh_test.editing_mode != EditingMode::None { return; }
        }
        let mut sh = match state_c.try_borrow_mut() { Ok(s) => s, Err(_) => return };
        if sh.mode == SelectionMode::Clipboard {
            let mut results = Vec::new();
            for item in &sh.clipboard_items {
                if item.name.to_lowercase().contains(&text.to_lowercase()) {
                    results.push(item.clone());
                }
            }
            sh.filtered_apps = results;
            sh.clip_index = 0;
            let filtered = sh.filtered_apps.clone();
            let config = sh.theme_config.clone();
            drop(sh);
            update_list_view(&container_c, &filtered, 0, &window_c, &state_c, &config);
            scroll_c.vadjustment().set_value(0.0);
            if let Ok(sh_re) = state_c.try_borrow() {
                update_visuals(&container_c, &scroll_c, &p_box_c, &sh_re);
            }
            return;
        }
        let mut results = Vec::new();
        if text.starts_with('#') || text.starts_with("rgb(") || text.starts_with("rgba(") {
            sh.mode = SelectionMode::Color;
            let mut parsed_rgba = None;
            if text.starts_with('#') {
                if text.len() == 7 || text.len() == 9 {
                    parsed_rgba = gtk4::gdk::RGBA::parse(&text).ok();
                } else if text.contains(',') {
                    let parts: Vec<&str> = text[1..].split(',').map(|s| s.trim()).collect();
                    if parts.len() == 3 || parts.len() == 4 {
                        let r = parts[0].parse::<u8>().ok();
                        let g = parts[1].parse::<u8>().ok();
                        let b = parts[2].parse::<u8>().ok();
                        let a = if parts.len() == 4 { parts[3].parse::<f32>().unwrap_or(1.0) } else { 1.0 };
                        if let (Some(rv), Some(gv), Some(bv)) = (r, g, b) {
                            parsed_rgba = Some(gtk4::gdk::RGBA::new(rv as f32 / 255.0, gv as f32 / 255.0, bv as f32 / 255.0, a));
                        }
                    }
                }
            } else {
                let inner = if text.starts_with("rgba(") {
                    text.trim_start_matches("rgba(").trim_end_matches(')')
                } else {
                    text.trim_start_matches("rgb(").trim_end_matches(')')
                };
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() == 3 || parts.len() == 4 {
                    let r = parts[0].parse::<u8>().ok();
                    let g = parts[1].parse::<u8>().ok();
                    let b = parts[2].parse::<u8>().ok();
                    let a = if parts.len() == 4 { parts[3].parse::<f32>().unwrap_or(1.0) } else { 1.0 };
                    if let (Some(rv), Some(gv), Some(bv)) = (r, g, b) {
                        parsed_rgba = Some(gtk4::gdk::RGBA::new(rv as f32 / 255.0, gv as f32 / 255.0, bv as f32 / 255.0, a));
                    }
                }
            }
            if !sh.is_syncing {
                if let Some(rgba) = parsed_rgba {
                    let (h, s, v) = rgb_to_hsv(rgba.red() as f64, rgba.green() as f64, rgba.blue() as f64);
                    sh.current_hsv = (h, s, v);
                    sh.current_alpha = rgba.alpha() as f64;
                    let st_idle = state_c.clone();
                    glib::idle_add_local(move || {
                        if let Ok(mut sh_idle) = st_idle.try_borrow_mut() {
                            sh_idle.is_syncing = true;
                            update_color_ui(&sh_idle);
                            sh_idle.is_syncing = false;
                        }
                        glib::ControlFlow::Break
                    });
                }
            }
            drop(sh);
            if let Ok(sh_re) = state_c.try_borrow() {
                update_visuals(&container_c, &scroll_c, &p_box_c, &sh_re);
            }
            return;
        }
        if text == "hotkeys?" {
            results.push(AppItem {
                name: "Show Hotkeys Help".to_string(),
                exec: "SHOW_HOTKEYS".to_string(),
                terminal: false,
                icon: "\u{f030c}".to_string(),
                desktop_id: "internal".to_string(),
                system_icon: None,
            });
        }
        if let Some(calc) = check_calc(&text) {
            results.push(calc);
        }
        
        let file_results = check_files(&text, &matcher, &sh.all_apps);
        if text.starts_with('/') || text.starts_with('~') {
            results.extend(file_results);
            results.extend(check_web(&text, &sh.history, &URL_RE, &sh.theme_config.search_engine));
        } else {
            results.extend(check_web(&text, &sh.history, &URL_RE, &sh.theme_config.search_engine));
            results.extend(file_results);
        }
        
        results.extend(check_system_commands(&text, &sh.power_options));
        let mut matches: Vec<(i64, AppItem)> = sh.all_apps
            .iter()
            .filter_map(|app| matcher.fuzzy_match(&app.name, &text).map(|sc| (sc, app.clone())))
            .collect();
        matches.sort_by(|(s1, a), (s2, b)| {
            s2.cmp(s1).then_with(|| {
                let h_a = sh.history.get(&a.exec).unwrap_or(&0);
                let h_b = sh.history.get(&b.exec).unwrap_or(&0);
                h_b.cmp(h_a)
            })
        });
        results.extend(matches.into_iter().map(|(_, a)| a));
        if text.is_empty() {
            results = sh.all_apps.clone();
            results.sort_by(|a, b| {
                let h_a = sh.history.get(&a.exec).unwrap_or(&0);
                let h_b = sh.history.get(&b.exec).unwrap_or(&0);
                h_b.cmp(h_a)
            });
        }
        sh.filtered_apps = results;
        sh.app_index = 0;
        sh.mode = SelectionMode::Apps;
        let filtered = sh.filtered_apps.clone();
        let config = sh.theme_config.clone();
        drop(sh);
        update_list_view(&container_c, &filtered, 0, &window_c, &state_c, &config);
        scroll_c.vadjustment().set_value(0.0);
        if let Ok(sh_re) = state_c.try_borrow() {
            update_visuals(&container_c, &scroll_c, &p_box_c, &sh_re);
        }
    });
}
pub fn setup_key_controller(
    win: &ApplicationWindow,
    entry: &gtk4::Entry,
    container: &Box,
    scroll: &ScrolledWindow,
    p_box: &Box,
    popup: &Box,
    edit_label: &gtk4::Label,
    edit_entry: &gtk4::Entry,
    state: &Rc<RefCell<LauncherState>>,
) {
    let (w, e, c, s, p, pop, el, ee, st, win_captured) = (
        win.clone(),
        entry.clone(),
        container.clone(),
        scroll.clone(),
        p_box.clone(),
        popup.clone(),
        edit_label.clone(),
        edit_entry.clone(),
        state.clone(),
        win.clone(),
    );
    let controller = EventControllerKey::new();
    controller.set_propagation_phase(PropagationPhase::Capture);
    let check_hk = |name: &str, key: gtk4::gdk::Key, mods: gtk4::gdk::ModifierType, hotkeys: &HashMap<String, crate::config::Hotkey>| -> bool {
        if let Some(hk) = hotkeys.get(name) {
            if let Some(hk_key) = gtk4::gdk::Key::from_name(&hk.key) {
                let clean_mods = mods & (gtk4::gdk::ModifierType::CONTROL_MASK | 
                                       gtk4::gdk::ModifierType::ALT_MASK | 
                                       gtk4::gdk::ModifierType::SHIFT_MASK | 
                                       gtk4::gdk::ModifierType::SUPER_MASK |
                                       gtk4::gdk::ModifierType::META_MASK);
                let mut target_mods = hk.mods;
                if target_mods.contains(gtk4::gdk::ModifierType::SUPER_MASK) {
                    if clean_mods.contains(gtk4::gdk::ModifierType::META_MASK) && !clean_mods.contains(gtk4::gdk::ModifierType::SUPER_MASK) {
                        target_mods = (target_mods & !gtk4::gdk::ModifierType::SUPER_MASK) | gtk4::gdk::ModifierType::META_MASK;
                    }
                }
                return key == hk_key && clean_mods == target_mods;
            }
        }
        false
    };
    controller.connect_key_pressed(move |_, key, _, modifier| {
        {
            let sh_test = match st.try_borrow() { Ok(s) => s, Err(_) => return glib::Propagation::Proceed };
            if sh_test.editing_mode != EditingMode::None {
                if key == gtk4::gdk::Key::Escape {
                    drop(sh_test);
                    let mut sh = st.borrow_mut();
                    sh.editing_mode = EditingMode::None; sh.editing_id = None;
                    pop.set_visible(false); e.grab_focus(); return glib::Propagation::Stop;
                }
                if key == gtk4::gdk::Key::Return {
                    drop(sh_test);
                    handle_app_edit(&st, &w, &c, &ee, &pop);
                    e.grab_focus();
                    return glib::Propagation::Stop;
                }
                return glib::Propagation::Proceed;
            }
        }
        let mut sh = st.borrow_mut();
        if key == gtk4::gdk::Key::Escape { drop(sh); w.close(); return glib::Propagation::Stop; }
        if key == gtk4::gdk::Key::Delete {
            let exec_to_remove = match sh.mode {
                SelectionMode::Apps => sh.filtered_apps.get(sh.app_index).map(|a| (a.exec.clone(), a.desktop_id.clone())),
                SelectionMode::Clipboard => sh.filtered_apps.get(sh.clip_index).map(|a| (a.exec.clone(), a.desktop_id.clone())),
                _ => None,
            };
            if let Some((exec, did)) = exec_to_remove {
                if did == "web" || did == "file" || did == "clipboard" {
                    sh.history.remove(&exec);
                    let text = e.text().to_string();
                    drop(sh);
                    e.set_text(&text);
                    return glib::Propagation::Stop;
                }
            }
        }
        if check_hk("hk-toggle-hidden", key, modifier, &sh.hotkeys) || (key == gtk4::gdk::Key::h && modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK)) {
            sh.show_hidden = !sh.show_hidden; sh.all_apps = get_apps(sh.show_hidden);
            sh.filtered_apps = sh.all_apps.clone(); sh.app_index = 0;
            let config = sh.theme_config.clone();
            update_list_view(&c, &sh.filtered_apps, 0, &w, &st, &config);
            update_visuals(&c, &s, &p, &sh); return glib::Propagation::Stop;
        }
        if (check_hk("hk-rename", key, modifier, &sh.hotkeys) || (key == gtk4::gdk::Key::r && modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK))) && sh.mode == SelectionMode::Apps {
            let app_to_edit = sh.filtered_apps.get(sh.app_index).map(|a| (a.name.clone(), a.desktop_id.clone()));
            if let Some((name, id)) = app_to_edit {
                if !["internal", "file", "web", "clipboard"].contains(&id.as_str()) {
                    sh.editing_mode = EditingMode::Rename;
                    sh.editing_id = Some(id);
                    el.set_text("Rename App");
                    pop.set_visible(true);
                    ee.remove_css_class("mono-text");
                    ee.set_text(&name);
                    ee.select_region(0, -1);
                    ee.grab_focus();
                    return glib::Propagation::Stop;
                }
            }
        }
        if (check_hk("hk-icon", key, modifier, &sh.hotkeys) || (key == gtk4::gdk::Key::e && modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK))) && sh.mode == SelectionMode::Apps {
            let app_to_edit = sh.filtered_apps.get(sh.app_index).map(|a| (a.icon.clone(), a.desktop_id.clone()));
            if let Some((icon, id)) = app_to_edit {
                if !["internal", "file", "web", "clipboard"].contains(&id.as_str()) {
                    sh.editing_mode = EditingMode::Icon;
                    sh.editing_id = Some(id);
                    el.set_text("Edit Icon");
                    pop.set_visible(true);
                    ee.add_css_class("mono-text");
                    let current_icon = if sh.theme_config.icon_mode == "system" { "".to_string() } else { icon };
                    ee.set_text(&current_icon);
                    ee.select_region(0, -1);
                    ee.grab_focus();
                    return glib::Propagation::Stop;
                }
            }
        }
        if (check_hk("hk-hide-app", key, modifier, &sh.hotkeys) || (key == gtk4::gdk::Key::s && modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK))) && sh.mode == SelectionMode::Apps {
            let app_id = sh.filtered_apps.get(sh.app_index).map(|a| a.desktop_id.clone());
            if let Some(id) = app_id {
                if !["internal", "file", "web", "clipboard"].contains(&id.as_str()) {
                    let mut overrides = crate::config::load_custom_overrides();
                    let ent = overrides.entry(id).or_default();
                    
                    ent.hidden = Some(!sh.show_hidden);
                    
                    crate::config::save_custom_overrides(&overrides);
                    sh.all_apps = get_apps(sh.show_hidden);
                    sh.filtered_apps = sh.all_apps.clone();
                    sh.app_index = sh.app_index.min(sh.filtered_apps.len().saturating_sub(1));
                    let config = sh.theme_config.clone();
                    update_list_view(&c, &sh.filtered_apps, sh.app_index, &w, &st, &config);
                    update_visuals(&c, &s, &p, &sh);
                    return glib::Propagation::Stop;
                }
            }
        }
        if key == gtk4::gdk::Key::i && modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
            sh.icon_mode = if sh.icon_mode == "system" { "nerd".to_string() } else { "system".to_string() };
            crate::config::save_icon_mode(&sh.icon_mode);
            sh.all_apps = get_apps(sh.show_hidden);
            sh.theme_config.icon_mode = sh.icon_mode.clone();
            let config = sh.theme_config.clone();
            update_list_view(&c, &sh.filtered_apps, sh.app_index, &w, &st, &config);
            update_visuals(&c, &s, &p, &sh);
            return glib::Propagation::Stop;
        }
        if check_hk("hk-clipboard", key, modifier, &sh.hotkeys) {
            if sh.mode == SelectionMode::Clipboard {
                sh.mode = SelectionMode::Apps;
                sh.filtered_apps = sh.all_apps.clone();
                sh.app_index = 0;
                e.set_placeholder_text(None);
            } else {
                sh.mode = SelectionMode::Clipboard;
                sh.clipboard_items = crate::search::get_clipboard_history();
                sh.filtered_apps = sh.clipboard_items.clone();
                sh.clip_index = 0;
                e.set_placeholder_text(Some("Clipboard Search..."));
            }
            e.set_text("");
            let config = sh.theme_config.clone();
            update_list_view(&c, &sh.filtered_apps, 0, &w, &st, &config);
            update_visuals(&c, &s, &p, &sh);
            return glib::Propagation::Stop;
        }
        if check_hk("hk-color-picker", key, modifier, &sh.hotkeys) || (key == gtk4::gdk::Key::g && modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK)) {
            if sh.mode == SelectionMode::Color {
                sh.mode = SelectionMode::Apps;
                sh.filtered_apps = sh.all_apps.clone();
                sh.app_index = 0;
                let config = sh.theme_config.clone();
                update_list_view(&c, &sh.filtered_apps, 0, &w, &st, &config);
                e.set_placeholder_text(None);
            } else {
                sh.mode = SelectionMode::Color;
                e.set_placeholder_text(None);
            }
            e.set_text("");
            update_visuals(&c, &s, &p, &sh);
            return glib::Propagation::Stop;
        }
        if key == gtk4::gdk::Key::Tab {
            if sh.mode == SelectionMode::Color {
                let focusable = vec![
                    sh.color_square.as_ref().map(|w| w.upcast_ref::<gtk4::Widget>()),
                    sh.hue_area.as_ref().map(|w| w.upcast_ref::<gtk4::Widget>()),
                    sh.alpha_area.as_ref().map(|w| w.upcast_ref::<gtk4::Widget>()),
                    sh.hex_copy_btn.as_ref().map(|w| w.upcast_ref::<gtk4::Widget>()),
                    sh.rgb_copy_btn.as_ref().map(|w| w.upcast_ref::<gtk4::Widget>()),
                ];
                let focusable: Vec<_> = focusable.into_iter().flatten().collect();
                let current_focus = gtk4::prelude::GtkWindowExt::focus(win_captured.upcast_ref::<gtk4::Window>());
                let mut next_idx = 0;
                if let Some(cf) = current_focus {
                    if let Some(idx) = focusable.iter().position(|w| **w == cf) {
                        next_idx = (idx + 1) % (focusable.len() + 1);
                    }
                } else {
                    next_idx = 0;
                }
                if next_idx < focusable.len() {
                    focusable[next_idx].grab_focus();
                } else {
                    e.grab_focus();
                }
                return glib::Propagation::Stop;
            }
            sh.mode = match sh.mode {
                SelectionMode::Apps => SelectionMode::Power,
                SelectionMode::Power => SelectionMode::Apps,
                SelectionMode::Clipboard => SelectionMode::Apps,
                SelectionMode::Color => SelectionMode::Apps,
            };
            if sh.mode == SelectionMode::Power { sh.power_index = 0; }
            if sh.mode == SelectionMode::Apps { e.set_placeholder_text(None); }
            update_visuals(&c, &s, &p, &sh); return glib::Propagation::Stop;
        }
        if sh.mode == SelectionMode::Apps {
            if key == gtk4::gdk::Key::Right {
                let mut path_to_set = None;
                if let Some(app) = sh.filtered_apps.get(sh.app_index) {
                    if app.desktop_id == "file" {
                        let raw_path = app.exec.trim_start_matches("OPEN_PATH:").to_string();
                        let home = std::env::var("HOME").unwrap_or_default();
                        let mut display_path = if raw_path.starts_with(&home) {
                            raw_path.replacen(&home, "", 1)
                        } else {
                            raw_path.clone()
                        };
                        if display_path.is_empty() { display_path = "/".to_string(); }
                        if (raw_path.ends_with('/') || std::path::Path::new(&raw_path).is_dir()) && !display_path.ends_with('/') {
                            display_path.push('/');
                        }
                        path_to_set = Some(display_path);
                    }
                }
                if let Some(p) = path_to_set {
                    drop(sh);
                    e.set_text(&p);
                    e.set_position(-1);
                    return glib::Propagation::Stop;
                }
            }
        }
        if sh.mode == SelectionMode::Color {
            let focus = gtk4::prelude::GtkWindowExt::focus(win_captured.upcast_ref::<gtk4::Window>());
            let mut changed = false;
            if let Some(f) = focus {
                let f_widget: gtk4::Widget = f.clone();
                if Some(f_widget.clone()) == sh.color_square.as_ref().map(|w| w.clone().upcast::<gtk4::Widget>()) {
                    let step = 0.02;
                    match key {
                        gtk4::gdk::Key::Left => { sh.current_hsv.1 = (sh.current_hsv.1 - step).clamp(0.0, 1.0); changed = true; }
                        gtk4::gdk::Key::Right => { sh.current_hsv.1 = (sh.current_hsv.1 + step).clamp(0.0, 1.0); changed = true; }
                        gtk4::gdk::Key::Up => { sh.current_hsv.2 = (sh.current_hsv.2 + step).clamp(0.0, 1.0); changed = true; }
                        gtk4::gdk::Key::Down => { sh.current_hsv.2 = (sh.current_hsv.2 - step).clamp(0.0, 1.0); changed = true; }
                        _ => {}
                    }
                } else if Some(f_widget.clone()) == sh.hue_area.as_ref().map(|w| w.clone().upcast::<gtk4::Widget>()) {
                    let step = 5.0;
                    match key {
                        gtk4::gdk::Key::Left | gtk4::gdk::Key::Down => { sh.current_hsv.0 = (sh.current_hsv.0 - step + 360.0) % 360.0; changed = true; }
                        gtk4::gdk::Key::Right | gtk4::gdk::Key::Up => { sh.current_hsv.0 = (sh.current_hsv.0 + step) % 360.0; changed = true; }
                        _ => {}
                    }
                } else if Some(f_widget.clone()) == sh.alpha_area.as_ref().map(|w| w.clone().upcast::<gtk4::Widget>()) {
                    let step = 0.02;
                    match key {
                        gtk4::gdk::Key::Left | gtk4::gdk::Key::Down => { sh.current_alpha = (sh.current_alpha - step).clamp(0.0, 1.0); changed = true; }
                        gtk4::gdk::Key::Right | gtk4::gdk::Key::Up => { sh.current_alpha = (sh.current_alpha + step).clamp(0.0, 1.0); changed = true; }
                        _ => {}
                    }
                }
            }
            if changed {
                update_color_ui(&sh);
                return glib::Propagation::Stop;
            }
        }
        if key == gtk4::gdk::Key::Down {
            if sh.mode == SelectionMode::Apps && sh.app_index + 1 < sh.filtered_apps.len() {
                sh.app_index += 1;
                update_visuals(&c, &s, &p, &sh);
            } else if sh.mode == SelectionMode::Clipboard && sh.clip_index + 1 < sh.filtered_apps.len() {
                sh.clip_index += 1;
                update_visuals(&c, &s, &p, &sh);
            }
            return glib::Propagation::Stop;
        }
        if key == gtk4::gdk::Key::Up {
            if sh.mode == SelectionMode::Apps && sh.app_index > 0 {
                sh.app_index -= 1;
                update_visuals(&c, &s, &p, &sh);
            } else if sh.mode == SelectionMode::Clipboard && sh.clip_index > 0 {
                sh.clip_index -= 1;
                update_visuals(&c, &s, &p, &sh);
            }
            return glib::Propagation::Stop;
        }
        if key == gtk4::gdk::Key::Return {
            if sh.mode == SelectionMode::Color {
                let (h, s, v) = sh.current_hsv;
                let (r, g, b) = hsv_to_rgb(h, s, v);
                let hex = format!("#{:02X}{:02X}{:02X}{:02X}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (sh.current_alpha * 255.0) as u8);
                let fol = sh.theme_config.focus_on_launch;
                let term_cmd = sh.theme_config.terminal.clone();
                launch_app(&format!("COPY:{}", hex), false, &mut sh.history, Some("color"), fol, &term_cmd);
                drop(sh); w.close(); return glib::Propagation::Stop;
            }
            let action = match sh.mode {
                SelectionMode::Apps => sh.filtered_apps.get(sh.app_index).map(|a| (Some(a.exec.clone()), a.terminal, false, Some(a.desktop_id.clone()))),
                SelectionMode::Power => if sh.power_index < sh.power_options.len() { Some((Some(sh.power_options[sh.power_index].command.clone()), false, true, None)) } else { None },
                SelectionMode::Clipboard => sh.filtered_apps.get(sh.clip_index).map(|a| (Some(a.exec.clone()), false, false, None)),
                SelectionMode::Color => None,
            };
            if let Some((ex, trm, _isp, did)) = action {
                if let Some(e_str) = &ex {
                    if e_str == "SHOW_HOTKEYS" {
                        let app_ref = w.application().expect("App error"); drop(sh);
                        create_hotkeys_window(&app_ref, &st); return glib::Propagation::Stop;
                    }
                }
                let fol = sh.theme_config.focus_on_launch;
                let term_cmd = sh.theme_config.terminal.clone();
                drop(sh);
                if let Some(exec) = ex {
                    let mut guard = st.borrow_mut();
                    launch_app(&exec, trm, &mut guard.history, did.as_deref(), fol, &term_cmd);
                }
                w.close();
            }
            return glib::Propagation::Stop;
        }
        if sh.mode == SelectionMode::Power {
            if matches!(key, gtk4::gdk::Key::Left | gtk4::gdk::Key::KP_Left) {
                sh.power_index = if sh.power_index > 0 { sh.power_index - 1 } else { sh.power_options.len().saturating_sub(1) };
                update_visuals(&c, &s, &p, &sh); return glib::Propagation::Stop;
            }
            if matches!(key, gtk4::gdk::Key::Right | gtk4::gdk::Key::KP_Right) {
                sh.power_index = if sh.power_index + 1 < sh.power_options.len() { sh.power_index + 1 } else { 0 };
                update_visuals(&c, &s, &p, &sh); return glib::Propagation::Stop;
            }
        }
        if let Some(c_char) = key.to_unicode() {
            if !c_char.is_control() && !modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK) && !modifier.contains(gtk4::gdk::ModifierType::ALT_MASK) && !modifier.contains(gtk4::gdk::ModifierType::SUPER_MASK) {
                if sh.mode == SelectionMode::Power {
                    sh.mode = SelectionMode::Apps; sh.app_index = 0;
                    update_visuals(&c, &s, &p, &sh);
                    if !e.has_focus() { e.grab_focus(); e.set_position(-1); }
                }
                return glib::Propagation::Proceed;
            }
        }
        glib::Propagation::Proceed
    });
    win.add_controller(controller);
}
pub fn setup_window_events(window: &ApplicationWindow, state: &Rc<RefCell<LauncherState>>) {
    let st = state.clone();
    window.connect_close_request(move |win| {
        let s = st.borrow();
        crate::config::save_state(&crate::config::WindowState {
            width: win.width(), height: win.height(), x: 0, y: 0,
            history: s.history.clone(), show_hidden: s.show_hidden, show_hotkeys: s.show_hotkeys,
        });
        glib::Propagation::Proceed
    });
    let focus_controller = gtk4::EventControllerFocus::new();
    let timer_handle = Rc::new(RefCell::new(None::<glib::SourceId>));
    let t_enter = timer_handle.clone();
    focus_controller.connect_enter(move |_| {
        if let Some(source_id) = t_enter.borrow_mut().take() {
            source_id.remove();
        }
    });
    let t_leave = timer_handle.clone();
    let w_leave = window.clone();
    focus_controller.connect_leave(move |_| {
        let w = w_leave.clone();
        let t = t_leave.clone();
        let id = glib::timeout_add_seconds_local(1, move || {
            if let Some(app) = w.application() {
                if app.windows().len() > 1 {
                    return glib::ControlFlow::Continue;
                }
            }
            w.close();
            *t.borrow_mut() = None;
            glib::ControlFlow::Break
        });
        *t_leave.borrow_mut() = Some(id);
    });
    window.add_controller(focus_controller);
    let st_wm = state.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(400), move || {
        if let Ok(s) = st_wm.try_borrow() {
            s.wm.center_cursor_or_window();
        }
        glib::ControlFlow::Break
    });
}
pub fn launch_app(exec: &str, terminal: bool, history: &mut HashMap<String, u32>, desktop_id: Option<&str>, focus_on_launch: bool, terminal_cmd: &str) {
    let wm = crate::wm::detect();
    if exec.is_empty() { return; }
    if exec.starts_with("CLIPBOARD_SET:") {
        let id = &exec[14..];
        let _ = Command::new("sh").arg("-c")
            .arg(format!("cliphist decode {} | wl-copy", id))
            .spawn();
        return;
    }
    if exec.starts_with("COPY:") {
        let val = &exec[5..];
        let _ = Command::new("wl-copy").arg(val).spawn();
        let title = if let Some(id) = desktop_id {
            if id == "color" { "Color Copied" } else if id == "calc" { "Result Copied" } else { "Copied" }
        } else {
            "Copied"
        };
        let _ = Command::new("notify-send").arg(title).arg(val).spawn();
        return;
    }
    let clean_exec = if exec.starts_with("xdg-open ") { exec.trim_start_matches("xdg-open ").trim_matches('"') }
                    else if exec.starts_with("OPEN_PATH:") { &exec[10..] }
                    else { exec };
    if focus_on_launch {
        if let Some(did) = desktop_id {
            if did != "file" && did != "web" && did != "calc" {
                let windows = wm.get_window_list();
                let (did_lower, did_base) = (did.to_lowercase(), did.to_lowercase().trim_end_matches(".desktop").to_string());
                for win in windows {
                    let app_id = win.app_id.to_lowercase();
                    let title = win.title.to_lowercase();
                    if app_id == did_base || app_id == did_lower || title.contains(&did_base) || (!app_id.is_empty() && did_base.contains(&app_id)) {
                        wm.focus_window(&win.id);
                        *history.entry(clean_exec.to_string()).or_insert(0) += 1;
                        return;
                    }
                }
            }
        }
    }
    if exec.starts_with("xdg-open ") || exec.starts_with("OPEN_PATH:") {
        let _ = Command::new("xdg-open").arg(clean_exec).spawn();
        *history.entry(clean_exec.to_string()).or_insert(0) += 1;
        return;
    }
    *history.entry(clean_exec.to_string()).or_insert(0) += 1;
    let cmd = exec.replace("%f","").replace("%F","").replace("%u","").replace("%U","").replace("%d","").replace("%D","").replace("%n","").replace("%N","").replace("%i","").replace("%c","").replace("%k","");
    let shell_cmd = if terminal { format!("setsid {} -e {} >/dev/null 2>&1 &", terminal_cmd, cmd.trim()) } else { format!("setsid {} >/dev/null 2>&1 &", cmd.trim()) };
    let _ = Command::new("sh").arg("-c").arg(shell_cmd).spawn();
}
