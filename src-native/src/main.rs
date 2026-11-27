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
    // Launch compact, borderless, centered startup splash (460x290).
    // Once backend & models are ready, NotelyApp transitions smoothly to the decorated main window.
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Notely")
        .with_inner_size(Vec2::new(460.0, 290.0))
        .with_decorations(false)
        .with_resizable(false)
        .with_transparent(false);

    if let Some(icon) = load_window_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Notely",
        options,
        Box::new(|cc| Ok(Box::new(app::NotelyApp::new(cc)))),
    )
}
