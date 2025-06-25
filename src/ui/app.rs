use crate::history::History;
use eframe::{egui, App, Frame};
use egui_extras::RetainedImage;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct HyprclipApp {
    shared_history: Arc<Mutex<History>>,
    selected_index: usize,
    storage_path: PathBuf,
}

impl HyprclipApp {
    pub fn new(history: Arc<Mutex<History>>, storage_path: PathBuf) -> Self {
        Self {
            shared_history: history,
            selected_index: 0,
            storage_path,
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
        let entries = { self.shared_history.lock().unwrap().entries.clone() };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📋 Clipboard-Verlauf");
            ui.separator();

            if entries.is_empty() {
                ui.label("Keine Einträge.");
            } else {
                for (i, entry) in entries.iter().enumerate() {
                    let sel = i == self.selected_index;
                    let resp = ui.selectable_label(sel, &entry.content);

                    if resp.clicked() {
                        let mut history = self.shared_history.lock().unwrap();

                        let entry = &history.entries[i];
                        if let Err(e) = crate::clipboard::set_clipboard_item(&entry.item) {
                            eprintln!("⚠️ Fehler beim Setzen der Zwischenablage: {}", e);
                        }

                        if i != 0 {
                            let selected = history.entries.remove(i);
                            history.entries.insert(0, selected);
                            self.selected_index = 0;
                            let _ = history.save(&self.storage_path);
                        }
                    }
                }

                if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown))
                    && self.selected_index + 1 < entries.len()
                {
                    self.selected_index += 1;
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) && self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
                    self.delete_selected();
                }
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
