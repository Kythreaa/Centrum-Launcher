use crate::ui::{LauncherState, create_hotkeys_window};
use crate::controller::launch_app;
use crate::search::AppItem;
use gtk4::prelude::*;
use gtk4::{Align, ApplicationWindow, Box, Image, Label, Orientation, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;
pub fn create_app_list() -> (ScrolledWindow, Box) {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();
    scrolled.add_css_class("scrolled-window");
    let list = Box::new(Orientation::Vertical, 0);
    list.add_css_class("app-list-container");
    scrolled.set_child(Some(&list));
    (scrolled, list)
}
pub fn initialize_list_view(container: &Box, window: &ApplicationWindow, state: &Rc<RefCell<LauncherState>>) {
    let (apps, config) = {
        let mut s = state.borrow_mut();
        s.filtered_apps = s.all_apps.clone();
        let history = s.history.clone();
        s.filtered_apps.sort_by(|a, b| {
            history.get(&b.exec).unwrap_or(&0).cmp(history.get(&a.exec).unwrap_or(&0))
        });
        (s.filtered_apps.clone(), s.theme_config.clone())
    };
    update_list_view(container, &apps, 0, window, state, &config);
}
pub fn update_list_view(
    container: &Box,
    apps: &[AppItem],
    sel_idx: usize,
    window: &ApplicationWindow,
    st_rc: &Rc<RefCell<LauncherState>>,
    config: &crate::config::ThemeConfig,
) {
    let align_val = config.text_align;
    let icon_pos = &config.icon_position;
    let icon_mode = &config.icon_mode;
    let icon_effect = &config.icon_effect;
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
    for (i, app) in apps.iter().take(100).enumerate() {
        let ib = Box::builder()
            .halign(Align::Fill)
            .valign(Align::Center)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(20)
            .margin_end(20)
            .build();
        ib.add_css_class("app-pill");
        ib.set_cursor_from_name(Some("pointer"));
        let (exec, term, win, st, did, fol) = (
            app.exec.clone(),
            app.terminal,
            window.clone(),
            st_rc.clone(),
            app.desktop_id.clone(),
            config.focus_on_launch,
        );
        let gest = gtk4::GestureClick::new();
        gest.connect_pressed(move |_, _, _, _| {
            if exec == "SHOW_HOTKEYS" {
                let app_ref = win.application().expect("App error");
                create_hotkeys_window(&app_ref, &st);
                return;
            }
            launch_app(&exec, term, &mut st.borrow_mut().history, Some(&did), fol);
            win.close();
        });
        ib.add_controller(gest);
        if i == sel_idx {
            ib.add_css_class("selected");
        }
        let cb = Box::builder().valign(Align::Center).build();
        if icon_pos == "adjacent" {
            cb.set_halign(if align_val < 0.4 { Align::Start } else if align_val > 0.6 { Align::End } else { Align::Center });
            cb.set_hexpand(true);
        } else {
            cb.set_halign(Align::Fill);
            cb.set_hexpand(true);
        }
        if icon_mode == "system" && app.system_icon.is_some() {
            let img = Image::from_gicon(app.system_icon.as_ref().unwrap());
            img.set_pixel_size(32);
            img.set_valign(Align::Center);
            img.set_halign(Align::Start);
            img.add_css_class("app-icon-img");
            if icon_effect == "shadow" { img.add_css_class("effect-shadow"); }
            if icon_effect == "outline" { img.add_css_class("effect-outline"); }
            img.set_margin_end(12);
            cb.append(&img);
        } else {
            let ic_lbl = Label::builder().label(&app.icon).valign(Align::Center).halign(Align::Start).build();
            ic_lbl.add_css_class("app-icon");
            cb.append(&ic_lbl);
        }
        let nm_lbl = Label::builder()
            .label(&app.name)
            .wrap(true)
            .wrap_mode(gtk4::pango::WrapMode::WordChar)
            .xalign(align_val)
            .valign(Align::Center)
            .build();
        nm_lbl.add_css_class("app-name");
        if app.desktop_id == "clipboard" { nm_lbl.add_css_class("clipboard-item"); }
        if icon_pos == "fixed" {
            nm_lbl.set_hexpand(true);
            nm_lbl.set_halign(Align::Fill);
        }
        cb.append(&nm_lbl);
        ib.append(&cb);
        container.append(&ib);
    }
}
