#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod theme;
mod types;
mod updater;
mod views;

use eframe::egui::{self, IconData, Vec2};

const APP_ICON_PNG: &[u8] = include_bytes!("../../src-tauri/icons/notely-icon.png");

fn load_window_icon() -> Option<IconData> {
    let img = image::load_from_memory(APP_ICON_PNG).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    // Launch directly into the crisp, decorated, High-DPI desktop window.
    // While the local engine starts, an integrated in-app loading screen displays cleanly on the canvas.
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Notely")
        .with_inner_size(Vec2::new(1260.0, 800.0))
        .with_min_inner_size(Vec2::new(900.0, 600.0))
        .with_decorations(true)
        .with_resizable(true)
        .with_maximized(true);

    if let Some(icon) = load_window_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        multisampling: 4,
        vsync: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Notely",
        options,
        Box::new(|cc| Ok(Box::new(app::NotelyApp::new(cc)))),
    )
}
