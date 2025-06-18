use crate::history::{Entry, History};
use arboard::Clipboard;
use eframe::{egui, App, Frame};
use std::sync::{Arc, Mutex};

pub async fn launch_with_history(
    history: Arc<Mutex<History>>,
    storage_path: std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Hyprclip",
        options,
        Box::new(move |_cc| Box::new(HyprclipApp::new(history, storage_path))),
    )
    .map_err(|e| format!("GUI konnte nicht gestartet werden: {e}").into())
}

struct HyprclipApp {
    shared_history: Arc<Mutex<History>>,
    selected_index: usize,
    clipboard: Clipboard,
    storage_path: std::path::PathBuf,
}

impl HyprclipApp {
    fn new(history: Arc<Mutex<History>>, storage_path: std::path::PathBuf) -> Self {
        let clipboard = Clipboard::new().expect("Clipboard konnte nicht initialisiert werden.");
        Self {
            shared_history: history,
            selected_index: 0,
            clipboard,
            storage_path,
        }
    }

    fn copy_to_clipboard(&mut self, content: &str) {
        if let Err(e) = self.clipboard.set_text(content.to_string()) {
            eprintln!("⚠️ Fehler beim Setzen des Clipboards: {e}");
        }
    }

    fn delete_selected(&mut self) {
        let mut history = self.shared_history.lock().unwrap();
        if self.selected_index < history.entries.len() {
            history.entries.remove(self.selected_index);
            let _ = history.save(&self.storage_path);
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
    }
}

impl App for HyprclipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let entries = {
            let history = self.shared_history.lock().unwrap();
            history.entries.clone()
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📋 Clipboard-Verlauf");

            ui.separator();

            if entries.is_empty() {
                ui.label("Keine Einträge.");
            } else {
                for (i, entry) in entries.iter().enumerate() {
                    let selected = i == self.selected_index;

                    let response = ui.selectable_label(selected, &entry.content);

                    if response.clicked() {
                        self.copy_to_clipboard(&entry.content);
                    }
                }

                // Tastatursteuerung
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    if self.selected_index + 1 < entries.len() {
                        self.selected_index += 1;
                    }
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(entry) = entries.get(self.selected_index) {
                        self.copy_to_clipboard(&entry.content);
                    }
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
                    self.delete_selected();
                }
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
