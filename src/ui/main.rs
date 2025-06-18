use eframe::egui;
use std::collections::VecDeque;
use std::fs;

fn load_clip_history() -> VecDeque<String> {
    if let Ok(data) = fs::read_to_string("/tmp/hyprclip.json") {
        if let Ok(list) = serde_json::from_str(&data) {
            return list;
        }
    }
    VecDeque::new()
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_simple_native("Hyprclip GUI", Default::default(), |ctx, _| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Clipboard Historie");
            for entry in load_clip_history() {
                if ui.button(&entry).clicked() {
                    let _ = std::process::Command::new("wl-copy").arg(entry).spawn();
                }
            }
        });
    })
}
