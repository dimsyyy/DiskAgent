#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod filter;
mod format;
mod i18n;
mod model;
mod platform;
mod scan;
mod settings;
mod theme;
mod ui;
mod web;

fn main() -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/DiskAgent.png"))
        .expect("valid DiskAgent icon");
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 720.0])
            .with_min_inner_size([900.0, 560.0])
            .with_title("DiskAgent")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "DiskAgent",
        options,
        Box::new(|cc| Ok(Box::new(app::DiskAgentApp::new(cc)))),
    )
}
