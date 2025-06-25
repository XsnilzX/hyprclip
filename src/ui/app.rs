use crate::history::History;
use eframe::{egui, App, Frame};
use egui::{Key, TextureHandle};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::SystemTime,
};

pub struct HyprclipApp {
    shared_history: Arc<Mutex<History>>,
    selected_index: usize,
    storage_path: PathBuf,
    image_cache: HashMap<PathBuf, TextureHandle>,
}

impl HyprclipApp {
    pub fn new(history: Arc<Mutex<History>>, storage_path: PathBuf) -> Self {
        Self {
            shared_history: history,
            selected_index: 0,
            storage_path,
            image_cache: HashMap::new(),
        }
    }

    fn delete_selected(&mut self) {
        let mut history = self.shared_history.lock().unwrap();
        if history.delete_entry(self.selected_index) {
            if let Err(e) = history.save(&self.storage_path) {
                eprintln!("Fehler beim Speichern: {}", e);
            }

            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
    }

    // Letzter Änderungszeitpunkt der Datei, um unnötiges Neuladen zu vermeiden
    fn maybe_reload_history(&mut self) {
        // Nur laden, wenn sich Datei geändert hat
        static mut LAST_MODIFIED: Option<SystemTime> = None;

        if let Ok(metadata) = std::fs::metadata(&self.storage_path) {
            if let Ok(modified) = metadata.modified() {
                unsafe {
                    if LAST_MODIFIED.map_or(true, |t| t != modified) {
                        // History neu laden
                        let new_hist = History::load(
                            &self.storage_path,
                            self.shared_history.lock().unwrap().limit,
                        );
                        *self.shared_history.lock().unwrap() = new_hist;

                        LAST_MODIFIED = Some(modified);
                    }
                }
            }
        }
    }
}

impl App for HyprclipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.maybe_reload_history();
        let entries = { self.shared_history.lock().unwrap().entries.clone() };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📋 Clipboard-Verlauf");
            ui.separator();

            if entries.is_empty() {
                ui.label("Keine Einträge.");
            } else {
                for (i, entry) in entries.iter().enumerate() {
                    let sel = i == self.selected_index;
                    let path = PathBuf::from(&entry.content);

                    let is_image = path.exists()
                        && path.is_file()
                        && (entry.content.ends_with(".png") || entry.content.ends_with(".jpg"));

                    if is_image {
                        let texture = self.image_cache.entry(path.clone()).or_insert_with(|| {
                            let image_data = std::fs::read(&path).unwrap_or_default();
                            let img = image::load_from_memory(&image_data).unwrap().to_rgba8();
                            let size = [img.width() as usize, img.height() as usize];

                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                size,
                                img.as_flat_samples().as_slice(),
                            );
                            // ctx muss hier verfügbar sein!
                            ctx.load_texture(
                                path.to_string_lossy(),
                                color_image,
                                egui::TextureOptions::default(),
                            )
                        });

                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(sel, path.file_name().unwrap().to_string_lossy())
                                .clicked()
                            {
                                self.selected_index = i;
                                let entry = &self.shared_history.lock().unwrap().entries[i];
                                let _ = crate::clipboard::set_clipboard_item(&entry.item);
                            }

                            ui.add(egui::Image::new(&*texture));
                        });
                    } else {
                        if ui.selectable_label(sel, &entry.content).clicked() {
                            self.selected_index = i;
                            let entry = &self.shared_history.lock().unwrap().entries[i];
                            let _ = crate::clipboard::set_clipboard_item(&entry.item);
                        }
                    }
                }

                if ctx.input(|i| i.key_pressed(Key::ArrowDown))
                    && self.selected_index + 1 < entries.len()
                {
                    self.selected_index += 1;
                }
                if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                if ctx.input(|i| i.key_pressed(Key::Delete)) {
                    self.delete_selected();
                }
            }
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
