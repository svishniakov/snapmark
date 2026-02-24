mod action_bar;
mod annotation;
mod app;
mod canvas;
mod clipboard;
mod flatten;
mod history;
mod platform;
mod state;
mod theme;
mod toolbar;
mod ui_controls;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("SnapMark")
        .with_inner_size([1080.0, 760.0])
        .with_min_inner_size([640.0, 480.0])
        .with_icon(load_app_icon());

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "SnapMark",
        options,
        Box::new(|cc| Box::new(app::SnapMarkApp::new(cc))),
    )
}

fn load_app_icon() -> egui::IconData {
    let bytes = include_bytes!("../assets/icon.png");
    eframe::icon_data::from_png_bytes(bytes).unwrap_or_default()
}
