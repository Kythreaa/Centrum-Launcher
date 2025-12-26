use crate::ui::{LauncherState, SelectionMode};
use crate::utils::hsv_to_rgb;
use crate::controller::launch_app;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, DrawingArea, Grid, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_color_picker() -> (Box, DrawingArea, DrawingArea, DrawingArea, DrawingArea, Label, Label, Button, Button) {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .halign(Align::Center)
        .valign(Align::Center)
        .visible(false)
        .width_request(280)
        .hexpand(true)
        .vexpand(true)
        .margin_top(60)
        .build();
    container.add_css_class("color-picker-container");
    let preview = DrawingArea::builder()
        .width_request(40)
        .height_request(40)
        .halign(Align::Center)
        .build();
    container.append(&preview);
    let square = DrawingArea::builder()
        .width_request(260)
        .height_request(180)
        .can_focus(true)
        .focusable(true)
        .build();
    square.add_css_class("color-square");
    container.append(&square);
    let hue_area = DrawingArea::builder()
        .width_request(260)
        .height_request(15)
        .can_focus(true)
        .focusable(true)
        .build();
    hue_area.add_css_class("hue-area");
    container.append(&hue_area);
    let alpha_area = DrawingArea::builder()
        .width_request(260)
        .height_request(15)
        .can_focus(true)
        .focusable(true)
        .build();
    alpha_area.add_css_class("alpha-area");
    container.append(&alpha_area);
    let grid = Grid::builder()
        .column_spacing(10)
        .row_spacing(10)
        .halign(Align::Center)
        .hexpand(true)
        .width_request(260)
        .build();
    let hex_label = Label::builder()
        .label("#FFFFFFFF")
        .css_classes(["app-name", "mono-text"])
        .halign(Align::Start)
        .hexpand(true)
        .width_request(180)
        .build();
    let hex_copy = Button::builder()
        .label("󰆏")
        .css_classes(["power-btn", "color-copy-btn"])
        .halign(Align::End)
        .build();
    let rgb_label = Label::builder()
        .label("255,255,255,1.00")
        .css_classes(["app-name", "mono-text"])
        .halign(Align::Start)
        .hexpand(true)
        .width_request(180)
        .build();
    let rgb_copy = Button::builder()
        .label("󰆏")
        .css_classes(["power-btn", "color-copy-btn"])
        .halign(Align::End)
        .build();
    grid.attach(&hex_label, 0, 0, 1, 1);
    grid.attach(&hex_copy, 1, 0, 1, 1);
    grid.attach(&rgb_label, 0, 1, 1, 1);
    grid.attach(&rgb_copy, 1, 1, 1, 1);
    container.append(&grid);
    container.set_margin_bottom(10);
    (container, preview, square, hue_area, alpha_area, hex_label, rgb_label, hex_copy, rgb_copy)
}

pub fn setup_color_picker_logic(state: &Rc<RefCell<LauncherState>>) {
    if let Ok(sh) = state.try_borrow() {
        update_color_ui(&sh);
    }
    if let Some(cp) = &state.borrow().color_preview {
        let st_c = state.clone();
        cp.set_draw_func(move |_, cr, width, height| {
            if let Ok(sh) = st_c.try_borrow() {
                let (h, s, v) = sh.current_hsv;
                let (r, g, b) = hsv_to_rgb(h, s, v);
                cr.set_source_rgb(r, g, b);
                cr.arc(width as f64 / 2.0, height as f64 / 2.0, (width.min(height) as f64 / 2.0) - 2.0, 0.0, 2.0 * std::f64::consts::PI);
                cr.fill().expect("Fill failed");
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_line_width(2.0);
                cr.arc(width as f64 / 2.0, height as f64 / 2.0, (width.min(height) as f64 / 2.0) - 2.0, 0.0, 2.0 * std::f64::consts::PI);
                cr.stroke().expect("Stroke failed");
            }
        });
    }
    if let Some(sq) = &state.borrow().color_square {
        let st_c = state.clone();
        sq.set_draw_func(move |_, cr, width, height| {
            if let Ok(sh) = st_c.try_borrow() {
                let (h, _, _) = sh.current_hsv;
                for y in 0..height {
                    for x in 0..width {
                        let s = x as f64 / width as f64;
                        let v = 1.0 - (y as f64 / height as f64);
                        let (r, g, b) = hsv_to_rgb(h, s, v);
                        cr.set_source_rgb(r, g, b);
                        cr.rectangle(x as f64, y as f64, 1.0, 1.0);
                        cr.fill().expect("Fill failed");
                    }
                }
                let (_, s, v) = sh.current_hsv;
                let cx = s * width as f64;
                let cy = (1.0 - v) * height as f64;
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_line_width(2.0);
                cr.arc(cx, cy, 5.0, 0.0, 2.0 * std::f64::consts::PI);
                cr.stroke().expect("Stroke failed");
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.arc(cx, cy, 6.0, 0.0, 2.0 * std::f64::consts::PI);
                cr.stroke().expect("Stroke failed");
            }
        });
        let st_drag = state.clone();
        let drag = gtk4::GestureDrag::new();
        let up_color = move |gesture: &gtk4::GestureDrag, x: f64, y: f64| {
            if let Ok(mut sh) = st_drag.try_borrow_mut() {
                if sh.mode == SelectionMode::Color {
                    let widget = gesture.widget().expect("No widget");
                    let (w, h) = (widget.width() as f64, widget.height() as f64);
                    let (start_x, start_y) = gesture.start_point().unwrap_or((0.0, 0.0));
                    let s = ((start_x + x) / w).clamp(0.0, 1.0);
                    let v = (1.0 - ((start_y + y) / h)).clamp(0.0, 1.0);
                    sh.current_hsv.1 = s; sh.current_hsv.2 = v;
                    
                    sh.is_syncing = true;
                    update_color_ui(&sh);
                    sh.is_syncing = false;
                }
            }
        };
        let (up_c, up_d) = (up_color.clone(), up_color);
        drag.connect_drag_begin(move |g, _, _| up_c(g, 0.0, 0.0));
        drag.connect_drag_update(move |g, x, y| up_d(g, x, y));
        sq.add_controller(drag);
    }
    if let Some(ha) = &state.borrow().hue_area {
        let st_c = state.clone();
        ha.set_draw_func(move |_, cr, width, height| {
            if let Ok(sh) = st_c.try_borrow() {
                for x in 0..width {
                    let h = (x as f64 / width as f64) * 360.0;
                    let (r, g, b) = hsv_to_rgb(h, 1.0, 1.0);
                    cr.set_source_rgb(r, g, b);
                    cr.rectangle(x as f64, 0.0, 1.0, height as f64);
                    cr.fill().expect("Fill failed");
                }
                let (h, _, _) = sh.current_hsv;
                let cx = (h / 360.0) * width as f64;
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_line_width(2.0);
                cr.rectangle(cx - 2.0, 0.0, 4.0, height as f64);
                cr.stroke().expect("Stroke failed");
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.rectangle(cx - 3.0, 0.0, 6.0, height as f64);
                cr.stroke().expect("Stroke failed");
            }
        });
        let st_ev = state.clone();
        let drag = gtk4::GestureDrag::new();
        let update_hue = move |gesture: &gtk4::GestureDrag, x: f64, _y: f64| {
            if let Ok(mut sh) = st_ev.try_borrow_mut() {
                if sh.mode == SelectionMode::Color {
                    let widget = gesture.widget().expect("No widget");
                    let w = widget.width() as f64;
                    let (start_x, _) = gesture.start_point().unwrap_or((0.0, 0.0));
                    let h = ((start_x + x) / w).clamp(0.0, 1.0) * 360.0;
                    sh.current_hsv.0 = h;
                    
                    sh.is_syncing = true;
                    update_color_ui(&sh);
                    sh.is_syncing = false;
                }
            }
        };
        let (up_h, up_d) = (update_hue.clone(), update_hue);
        drag.connect_drag_begin(move |g, _, _| up_h(g, 0.0, 0.0));
        drag.connect_drag_update(move |g, x, y| up_d(g, x, y));
        ha.add_controller(drag);
    }
    if let Some(aa) = &state.borrow().alpha_area {
        let st_c = state.clone();
        aa.set_draw_func(move |_, cr, width, height| {
            if let Ok(sh) = st_c.try_borrow() {
                let (h, s, v) = sh.current_hsv;
                let (r, g, b) = hsv_to_rgb(h, s, v);
                let check_size = 5.0;
                for y in (0..height).step_by(check_size as usize) {
                    for x in (0..width).step_by(check_size as usize) {
                        if ((x as f64 / check_size) as i32 + (y as f64 / check_size) as i32) % 2 == 0 {
                            cr.set_source_rgb(0.8, 0.8, 0.8);
                        } else {
                            cr.set_source_rgb(1.0, 1.0, 1.0);
                        }
                        cr.rectangle(x as f64, y as f64, check_size, check_size);
                        cr.fill().expect("Fill failed");
                    }
                }
                let pattern = gtk4::cairo::LinearGradient::new(0.0, 0.0, width as f64, 0.0);
                pattern.add_color_stop_rgba(0.0, r, g, b, 0.0);
                pattern.add_color_stop_rgba(1.0, r, g, b, 1.0);
                cr.set_source(&pattern).expect("Pattern failed");
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                cr.fill().expect("Fill failed");
                let cx = sh.current_alpha * width as f64;
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_line_width(2.0);
                cr.rectangle(cx - 2.0, 0.0, 4.0, height as f64);
                cr.stroke().expect("Stroke failed");
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.rectangle(cx - 3.0, 0.0, 6.0, height as f64);
                cr.stroke().expect("Stroke failed");
            }
        });
        let st_ev = state.clone();
        let drag = gtk4::GestureDrag::new();
        let update_alpha = move |gesture: &gtk4::GestureDrag, x: f64, _y: f64| {
            if let Ok(mut sh) = st_ev.try_borrow_mut() {
                if sh.mode == SelectionMode::Color {
                    let widget = gesture.widget().expect("No widget");
                    let w = widget.width() as f64;
                    let (start_x, _) = gesture.start_point().unwrap_or((0.0, 0.0));
                    let cur_x = (start_x + x).clamp(0.0, w);
                    let a = cur_x / w;
                    sh.current_alpha = a;
                    
                    sh.is_syncing = true;
                    update_color_ui(&sh);
                    sh.is_syncing = false;
                }
            }
        };
        let (up_a, up_d) = (update_alpha.clone(), update_alpha);
        drag.connect_drag_begin(move |g, _, _| up_a(g, 0.0, 0.0));
        drag.connect_drag_update(move |g, x, y| up_d(g, x, y));
        aa.add_controller(drag);
    }
    if let Some(hb) = &state.borrow().hex_copy_btn {
        let st_c = state.clone();
        hb.connect_clicked(move |_| {
            let sh = st_c.borrow();
            let (h, s, v) = sh.current_hsv;
            let (r, g, b) = hsv_to_rgb(h, s, v);
            let hex = format!("#{:02X}{:02X}{:02X}{:02X}", (r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8, (sh.current_alpha*255.0) as u8);
            let mut hist = sh.history.clone();
            launch_app(&format!("COPY:{}", hex), false, &mut hist, Some("color"), sh.theme_config.focus_on_launch);
        });
    }
    if let Some(rb) = &state.borrow().rgb_copy_btn {
        let st_c = state.clone();
        rb.connect_clicked(move |_| {
            let sh = st_c.borrow();
            let (h, s, v) = sh.current_hsv;
            let (r, g, b) = hsv_to_rgb(h, s, v);
            let rgb_str = format!("rgba({},{},{},{:.2})", (r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8, sh.current_alpha);
            let mut hist = sh.history.clone();
            launch_app(&format!("COPY:{}", rgb_str), false, &mut hist, Some("color"), sh.theme_config.focus_on_launch);
        });
    }
}

pub fn update_color_ui(sh: &LauncherState) {
    let (h, s, v) = sh.current_hsv;
    let (r, g, b) = hsv_to_rgb(h, s, v);
    let a = sh.current_alpha;
    
    let hex = format!("#{:02X}{:02X}{:02X}{:02X}", (r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8, (a*255.0) as u8);
    let rgb = format!("{},{},{},{:.2}", (r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8, a);

    if let Some(hl) = &sh.hex_label { hl.set_text(&hex); }
    if let Some(rl) = &sh.rgb_label { rl.set_text(&rgb); }
    
    if let Some(sq) = &sh.color_square { sq.queue_draw(); }
    if let Some(ha) = &sh.hue_area { ha.queue_draw(); }
    if let Some(aa) = &sh.alpha_area { aa.queue_draw(); }
    if let Some(pv) = &sh.color_preview { pv.queue_draw(); }
}