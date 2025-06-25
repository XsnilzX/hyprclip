use crate::history::History;
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
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Hyprclip",
        options,
        Box::new(move |_cc| Ok(Box::new(HyprclipApp::new(history, storage_path)))),
    )
    .map_err(|e| format!("GUI konnte nicht gestartet werden: {e}").into())
}
