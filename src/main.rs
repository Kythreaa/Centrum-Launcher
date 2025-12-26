mod config;
mod controller;
mod search;
mod ui;
mod modules;
mod utils;
mod wm;
use gtk4::prelude::*;
use gtk4::Application;
fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("org.centrum.launcher")
        .build();
    app.connect_activate(ui::build_ui);
    app.run()
}