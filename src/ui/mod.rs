use crate::history::History;
use eframe::{egui, icon_data::from_png_bytes, NativeOptions};
use include_bytes_plus::include_bytes;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

mod app;
use app::HyprclipApp;

pub fn launch_with_history(
    history: Arc<Mutex<History>>,
    storage_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Icon laden (als Byte-Array – kein image crate nötig!)
    let icon_bytes = include_bytes!("assets/icon.png");
    let icon = from_png_bytes(&icon_bytes)?;

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Hyprclip")
            .with_inner_size([500.0, 300.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_always_on_top()
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Hyprclip",
        options,
        Box::new(move |_cc| Ok(Box::new(HyprclipApp::new(history, storage_path)))),
    )
    .map_err(|e| format!("GUI konnte nicht gestartet werden: {e}").into())
}
