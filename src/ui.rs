use crate::config::{load_state, ThemeConfig, PowerOption};
use crate::search::{get_apps, AppItem};
use crate::controller::{setup_key_controller, setup_search_logic, setup_window_events};
use crate::modules::color_picker::{create_color_picker, setup_color_picker_logic};
use crate::modules::app_launcher::{create_app_list, initialize_list_view};
use crate::modules::power_menu::create_power_bar;
use crate::wm::{self, WindowManager}; 
use gtk4::prelude::*;
use gtk4::{
    Align,
    Application,
    ApplicationWindow,
    Box,
    Button,
    CheckButton,
    DrawingArea,
    Grid,
    Label,
    Orientation,
    Overlay,
    ScrolledWindow,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SelectionMode {
    Apps,
    Power,
    Clipboard,
    Color,
}
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum EditingMode {
    None,
    Rename,
    Icon,
}
pub struct LauncherState {
    pub all_apps: Vec<AppItem>,
    pub filtered_apps: Vec<AppItem>,
    pub clipboard_items: Vec<AppItem>,
    pub app_index: usize,
    pub power_index: usize,
    pub clip_index: usize,
    pub power_options: Vec<PowerOption>,
    pub mode: SelectionMode,
    pub editing_mode: EditingMode,
    pub editing_id: Option<String>,
    pub history: HashMap<String, u32>,
    pub show_hidden: bool,
    pub show_hotkeys: bool,
    pub icon_mode: String,
    pub hotkeys: HashMap<String, crate::config::Hotkey>,
    pub theme_config: ThemeConfig,
    pub wm: std::boxed::Box<dyn WindowManager>, 
    pub color_box: Option<Box>,
    pub hex_label: Option<Label>,
    pub rgb_label: Option<Label>,
    pub color_square: Option<DrawingArea>,
    pub color_preview: Option<DrawingArea>,
    pub hue_area: Option<DrawingArea>,
    pub alpha_area: Option<DrawingArea>,
    pub hex_copy_btn: Option<Button>,
    pub rgb_copy_btn: Option<Button>,
    pub current_hsv: (f64, f64, f64),
    pub current_alpha: f64,
    pub is_syncing: bool,
}
pub fn create_hotkeys_window(app: &Application, state: &Rc<RefCell<LauncherState>>) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hotkeys")
        .default_width(450)
        .default_height(550)
        .decorated(false)
        .build();
    let root = Box::builder().orientation(Orientation::Vertical).build();
    root.add_css_class("main-container");
    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(20)
        .margin_top(40)
        .margin_bottom(40)
        .margin_start(40)
        .margin_end(40)
        .halign(Align::Center)
        .valign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .build();
    root.append(&content);
    let hk_label = Label::builder().build();
    hk_label.add_css_class("app-name");
    hk_label.set_markup("<span weight='bold' size='xx-large'>Hotkeys</span>");
    content.append(&hk_label);
    let hk_grid = Grid::builder().column_spacing(30).row_spacing(8).build();
    let keys = vec![
        ("Enter", "Launch / Action"),
        ("Esc", "Close / Cancel"),
        ("Tab", "Switch Apps / Power / Color"),
        ("Up / Down", "Navigate List"),
        ("Delete", "Remove item from search and link history"),
        ("Ctrl + Z", "Clipboard History"),
        ("Ctrl + G", "Color Picker"),
        ("Ctrl + R", "Rename App"),
        ("Ctrl + E", "Change Icon"),
        ("Ctrl + S", "Hide / Show App"),
        ("Ctrl + H", "Toggle Hidden Apps"),
        ("Ctrl + I", "Toggle Icon Mode"),
        (":", "Browser History"),
        ("?", "Web Search"),
        ("/", "File Search"),
    ];
    for (i, (k, d)) in keys.into_iter().enumerate() {
        let k_lbl = Label::builder().label(k).halign(Align::Start).build();
        k_lbl.add_css_class("app-name");
        let d_lbl = Label::builder().label(d).halign(Align::Start).build();
        d_lbl.add_css_class("app-name");
        hk_grid.attach(&k_lbl, 0, i as i32, 1, 1);
        hk_grid.attach(&d_lbl, 1, i as i32, 1, 1);
    }
    content.append(&hk_grid);
    let config_note = Label::builder()
        .label("config.css to set power options")
        .wrap(true)
        .max_width_chars(40)
        .justify(gtk4::Justification::Center)
        .build();
    config_note.add_css_class("app-name");
    content.append(&config_note);
    let dont_show_check = CheckButton::builder()
        .label("Don't show again")
        .active(!state.borrow().show_hotkeys)
        .build();
    dont_show_check.add_css_class("app-name");
    let st_hk = state.clone();
    dont_show_check.connect_toggled(move |btn| {
        st_hk.borrow_mut().show_hotkeys = !btn.is_active();
    });
    content.append(&dont_show_check);
    let close_hk_btn = Button::builder().label("Close").build();
    close_hk_btn.add_css_class("app-name");
    let win_clone = window.clone();
    close_hk_btn.connect_clicked(move |_| win_clone.close());
    content.append(&close_hk_btn);
    window.set_child(Some(&root));
    window.present();
}
pub fn build_ui(app: &Application) {
    crate::config::ensure_config_files();
    let windows = app.windows();
    if !windows.is_empty() {
        for w in &windows {
            w.close();
        }
        return;
    }
    let state_saved = load_state();
    let theme_config = ThemeConfig::load();
    let wm = wm::detect(); 
    let window = ApplicationWindow::builder()
        .application(app)
        .title("centrum-launcher")
        .default_width(state_saved.width)
        .default_height(state_saved.height)
        .decorated(false)
        .build();
    let overlay = Overlay::new();
    overlay.add_css_class("main-container");
    let (scrolled_window, list_container) = create_app_list();
    overlay.set_child(Some(&scrolled_window));
    let header_bg = Box::builder()
        .height_request(100)
        .valign(Align::Start)
        .build();
    header_bg.add_css_class("header-bg");
    overlay.add_overlay(&header_bg);
    let search_entry = gtk4::Entry::builder()
        .halign(Align::Fill)
        .hexpand(true)
        .has_frame(false)
        .valign(Align::Start)
        .build();
    search_entry.add_css_class("search-entry");
    search_entry.set_input_hints(gtk4::InputHints::NO_EMOJI);
    gtk4::prelude::EntryExt::set_alignment(&search_entry, 0.5);
    overlay.add_overlay(&search_entry);
    let (edit_popup, edit_label, edit_entry) = create_edit_popup();
    overlay.add_overlay(&edit_popup);
    let power_options = theme_config.power_options.clone();
    let power_box = create_power_bar(&power_options, &window);
    overlay.add_overlay(&power_box);
    let (color_box, color_preview, color_square, hue_area, alpha_area, hex_label, rgb_label, hex_copy, rgb_copy) = create_color_picker();
    overlay.add_overlay(&color_box);
    window.set_child(Some(&overlay));
    let state = Rc::new(RefCell::new(LauncherState {
        all_apps: get_apps(false),
        filtered_apps: Vec::new(),
        clipboard_items: Vec::new(),
        app_index: 0,
        power_index: 0,
        clip_index: 0,
        power_options,
        mode: SelectionMode::Apps,
        editing_mode: EditingMode::None,
        editing_id: None,
        history: state_saved.history,
        show_hidden: false,
        show_hotkeys: state_saved.show_hotkeys,
        icon_mode: theme_config.icon_mode.clone(),
        hotkeys: theme_config.hotkeys.clone(),
        theme_config,
        wm,
        color_box: Some(color_box),
        hex_label: Some(hex_label),
        rgb_label: Some(rgb_label),
        color_square: Some(color_square),
        color_preview: Some(color_preview),
        hue_area: Some(hue_area),
        alpha_area: Some(alpha_area),
        hex_copy_btn: Some(hex_copy),
        rgb_copy_btn: Some(rgb_copy),
        current_hsv: (0.0, 0.0, 1.0),
        current_alpha: 1.0,
        is_syncing: false,
    }));
    if state_saved.show_hotkeys {
        create_hotkeys_window(app, &state);
    }
    initialize_list_view(&list_container, &window, &state);
    setup_search_logic(&search_entry, &list_container, &scrolled_window, &power_box, &window, &state);
    setup_color_picker_logic(&state);
    setup_key_controller(
        &window, 
        &search_entry, 
        &list_container, 
        &scrolled_window, 
        &power_box, 
        &edit_popup, 
        &edit_label, 
        &edit_entry, 
        &state
    );
    setup_window_events(&window, &state);
    setup_css();
    window.present();
    search_entry.grab_focus();
}
fn create_edit_popup() -> (Box, Label, gtk4::Entry) {
    let popup = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .halign(Align::Center)
        .valign(Align::Center)
        .visible(false)
        .build();
    popup.add_css_class("edit-popup");
    let label = Label::builder().label("Edit").css_classes(["edit-popup-label"]).build();
    let entry = gtk4::Entry::builder().has_frame(false).css_classes(["edit-popup-entry"]).build();
    gtk4::prelude::EntryExt::set_alignment(&entry, 0.5);
    popup.append(&label);
    popup.append(&entry);
    (popup, label, entry)
}
pub fn update_visuals(
    container: &Box,
    scroll_w: &ScrolledWindow,
    p_box: &Box,
    state: &LauncherState,
) {
    let mut i = 0;
    let mut iter = container.first_child();
    let adj = scroll_w.vadjustment();
    let scroll_settings = &state.theme_config.scroll;
    let top_pad = scroll_settings.top_padding;
    let bot_pad = scroll_settings.bottom_padding;
    let target_idx = match state.mode {
        SelectionMode::Apps => state.app_index,
        SelectionMode::Clipboard => state.clip_index,
        SelectionMode::Power => 99999, 
        SelectionMode::Color => 99999,
    };
    if let Some(cb) = &state.color_box {
        cb.set_visible(state.mode == SelectionMode::Color);
    }
    scroll_w.set_visible(state.mode != SelectionMode::Color);
    p_box.set_visible(state.mode != SelectionMode::Color);
    while let Some(child) = iter {
        if (state.mode == SelectionMode::Apps || state.mode == SelectionMode::Clipboard)
            && i == target_idx
        {
            child.add_css_class("selected");
            if let Some((_, y)) = child.translate_coordinates(container, 0.0, 0.0) {
                let (h, val, ps) = (child.height() as f64, adj.value(), adj.page_size());
                let target = if y < val + top_pad {
                    Some(y - top_pad)
                } else if y + h > val + ps - bot_pad {
                    Some(y + h - ps + bot_pad)
                } else {
                    None
                };
                if let Some(t) = target {
                    smooth_scroll(&adj, t, scroll_settings);
                }
            }
        } else {
            child.remove_css_class("selected");
        }
        iter = child.next_sibling();
        i += 1;
    }
    let mut pi = 0;
    let mut p_iter = p_box.first_child();
    while let Some(child) = p_iter {
        if state.mode == SelectionMode::Power && pi == state.power_index {
            child.add_css_class("selected");
        } else {
            child.remove_css_class("selected");
        }
        p_iter = child.next_sibling();
        pi += 1;
    }
}
fn smooth_scroll(adj: &gtk4::Adjustment, target: f64, settings: &crate::config::ScrollSettings) {
    thread_local! {
        static ACTIVE_SCROLL: RefCell<Option<glib::SourceId>> = RefCell::new(None);
    }
    if (adj.value() - target).abs() < 1.0 {
        adj.set_value(target);
        return;
    }
    let duration = settings.duration;
    let interval = settings.interval;
    let easing = settings.easing.clone();
    ACTIVE_SCROLL.with(|active| {
        if let Some(id) = active.borrow_mut().take() {
            id.remove();
        }
        let (start, start_time, adj_clone) = (adj.value(), std::time::Instant::now(), adj.clone());
        let id = glib::timeout_add_local(
            std::time::Duration::from_millis(interval),
            move || {
                let t = (start_time.elapsed().as_millis() as f64 / duration).min(1.0);
                let ease = match easing.as_str() {
                    "linear" => t,
                    "quad" => 1.0 - (1.0 - t) * (1.0 - t),
                    "cubic" => 1.0 - (1.0 - t).powi(3),
                    "quart" => 1.0 - (1.0 - t).powi(4),
                    "expo" => {
                        if t == 1.0 { 1.0 } else { 1.0 - 2.0_f64.powf(-10.0 * t) }
                    }
                    _ => 1.0 - (1.0 - t).powi(3), 
                };
                adj_clone.set_value(start + (target - start) * ease);
                if t >= 1.0 {
                    ACTIVE_SCROLL.with(|a| *a.borrow_mut() = None);
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            },
        );
        *active.borrow_mut() = Some(id);
    });
}
fn load_css_files(provider: &gtk4::CssProvider) {
    let dir = crate::config::get_config_dir();
    let mut final_css = String::new();
    if let Ok(css) = std::fs::read_to_string(dir.join("config.css")) {
        final_css.push_str(&css);
    } else {
        final_css.push_str(crate::config::DEFAULT_CONFIG_CSS);
    }
    let mut is_dark = false;
    let schema_id = "org.gnome.desktop.interface";
    if let Some(source) = gio::SettingsSchemaSource::default() {
        if source.lookup(schema_id, true).is_some() {
            let gsettings = gio::Settings::new(schema_id);
            let (cs, gt) = (
                gsettings.get::<String>("color-scheme"),
                gsettings.get::<String>("gtk-theme").to_lowercase(),
            );
            is_dark = cs == "prefer-dark" || gt.contains("dark");
        }
    }
    let theme_file = if is_dark { "dark.css" } else { "light.css" };
    let default_theme = if is_dark { crate::config::DEFAULT_DARK_CSS } else { crate::config::DEFAULT_LIGHT_CSS };
    final_css.push_str("\n");
    if let Ok(css) = std::fs::read_to_string(dir.join(theme_file)) {
        final_css.push_str(&css);
    } else {
        final_css.push_str(default_theme);
    }
    provider.load_from_data(&final_css);
}
fn setup_css() {
    if let Some(display) = gtk4::gdk::Display::default() {
        let provider = gtk4::CssProvider::new();
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_USER,
        );
        let internal_css = "
            .app-icon, .power-btn, .mono-text, .mono-text text, .mono-text contents, .edit-popup-entry.mono-text, .edit-popup-entry.mono-text text {
                font-family: '0xProto Nerd Font', '0xProto Nerd Font Mono', 'Symbols Nerd Font Mono', 'Symbols Nerd Font', 'Nerd Font', 'monospace' !important;
            }
            .mono-text {
                font-size: 0.85rem !important;
            }
            .effect-shadow {
                text-shadow: 2px 2px 3px rgba(0,0,0,0.6);
                filter: drop-shadow(2px 2px 3px rgba(0,0,0,0.6));
            }
            .effect-outline {
                text-shadow: -1px -1px 0 #000, 1px -1px 0 #000, -1px 1px 0 #000, 1px 1px 0 #000;
                filter: drop-shadow(0 0 1px #000);
            }
            .color-picker-container {
                max-width: 320px;
            }
            .color-picker-container colorchooser .palette {
                display: none;
            }
            .color-picker-container colorchooser stack > box:first-child {
                display: none;
            }
            .color-picker-container colorchooser stack > box:last-child {
                display: flex;
            }
            .color-label-hex, .color-label-rgb {
                font-size: 1rem;
                font-weight: bold;
                color: white;
                margin: 0;
                padding: 0;
            }
            .hue-scale trough {
                background-image: linear-gradient(to right, #ff0000, #ffff00, #00ff00, #00ffff, #0000ff, #ff00ff, #ff0000);
                min-height: 12px;
                border-radius: 5px;
            }
            .hue-scale trough highlight {
                background: none;
            }
            .hue-scale slider {
                min-height: 16px;
                min-width: 16px;
                background: white;
                border: 2px solid black;
                box-shadow: none;
                margin-top: -2px;
                margin-bottom: -2px;
            }
        ";
        let internal_provider = gtk4::CssProvider::new();
        internal_provider.load_from_data(internal_css);
                    gtk4::style_context_add_provider_for_display(
                        &display,
                        &internal_provider,
                        gtk4::STYLE_PROVIDER_PRIORITY_USER + 10,
                    );        load_css_files(&provider);
        let schema_id = "org.gnome.desktop.interface";
        if let Some(source) = gio::SettingsSchemaSource::default() {
            if source.lookup(schema_id, true).is_some() {
                let gsettings = gio::Settings::new(schema_id);
                let p1 = provider.clone();
                gsettings.connect_changed(Some("color-scheme"), move |_, _| load_css_files(&p1));
                let p2 = provider.clone();
                gsettings.connect_changed(Some("gtk-theme"), move |_, _| load_css_files(&p2));
            }
        }
    }
}