use crate::config::PowerOption;
use gtk4::prelude::*;
use gtk4::{Align, ApplicationWindow, Box, Button, Orientation};
use std::process::Command;
pub fn create_power_bar(options: &[PowerOption], window: &ApplicationWindow) -> Box {
    let bar = Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(Align::Center)
        .valign(Align::End)
        .margin_bottom(20)
        .build();
    bar.add_css_class("power-bar");
    for opt in options {
        let btn = Button::builder().label(&opt.icon).has_frame(false).build();
        btn.add_css_class("power-btn");
        btn.add_css_class(&opt.class);
        let win_clone = window.clone();
        let cmd_str = opt.command.clone();
        btn.connect_clicked(move |_| {
            let _ = Command::new("sh").arg("-c").arg(&cmd_str).spawn();
            win_clone.close();
        });
        bar.append(&btn);
    }
    bar
}
