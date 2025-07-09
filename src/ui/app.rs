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

                        // Hole alle neuen Pfade **vor** dem Move
                        let entries: Vec<_> = new_hist
                            .entries
                            .iter()
                            .map(|e| PathBuf::from(&e.content))
                            .collect();

                        // Schreibe new_hist in shared_history (Move)
                        *self.shared_history.lock().unwrap() = new_hist;

                        // Invalide cache für gelöschte/geänderte Pfade
                        self.image_cache.retain(|k, _| entries.contains(k));

                        LAST_MODIFIED = Some(modified);
                    }
                }
            }
        }
    }

    fn handle_key_inputs(&mut self, ctx: &egui::Context, entries_len: usize) {
        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) && self.selected_index + 1 < entries_len {
            self.selected_index += 1;
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && self.selected_index > 0 {
            self.selected_index -= 1;
        }
        if ctx.input(|i| i.key_pressed(Key::Delete)) {
            self.delete_selected();
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ctx.input(|i| i.key_pressed(Key::Enter)) && self.selected_index <= entries_len {
            self.select_entry(self.selected_index);
        }
    }

    fn select_entry(&mut self, index: usize) {
        let mut history = self.shared_history.lock().unwrap();

        if index >= history.entries.len() {
            return;
        }

        let entry = history.entries.remove(index);

        // Entferne andere Duplikate
        history.entries.retain(|e| e.content != entry.content);

        // Setze an den Anfang
        history.entries.insert(0, entry.clone());

        // Bild-Einträge: Hash ggf. nachtragen, Clip setzen ohne erneute Erkennung
        if let crate::history::ClipboardItem::Image(ref path) = entry.item {
            if history.entries[0].hash.is_none() {
                if let Ok(data) = std::fs::read(path) {
                    history.entries[0].hash = Some(crate::util::hash_data(&data));
                }
            }
        }

        let _ = crate::clipboard::set_clipboard_item_no_ignore(&entry.item);

        self.selected_index = 0;

        if let Err(e) = history.save(&self.storage_path) {
            eprintln!("Fehler beim Speichern nach select_entry: {}", e);
        }
    }

    fn fallback_texture(ctx: &egui::Context, path: &PathBuf) -> egui::TextureHandle {
        // Erzeuge ein 1x1 transparentes Bild als Platzhalter
        let fallback_image = egui::ColorImage::from_rgba_unmultiplied([1, 1], &[0, 0, 0, 0]);
        ctx.load_texture(
            format!("fallback:{}", path.to_string_lossy()),
            fallback_image,
            egui::TextureOptions::default(),
        )
    }
}

impl App for HyprclipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.maybe_reload_history();
        let entries = { self.shared_history.lock().unwrap().entries.clone() };

        // 🔑 Eingaben verarbeiten (Up, Down, Delete, Escape)
        self.handle_key_inputs(ctx, entries.len());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📋 Clipboard-Verlauf");
            ui.separator();

            if entries.is_empty() {
                ui.label("Keine Einträge.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("history_grid")
                        .striped(true)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            for (i, entry) in entries.iter().enumerate() {
                                let sel = i == self.selected_index;
                                let path = PathBuf::from(&entry.content);

                                // 👉 Spalte 1: Eintragsname
                                let response = ui.selectable_label(
                                    sel,
                                    path.file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                );

                                // ✅ Spalte 2: Thumbnail (falls Bild)
                                if path.exists()
                                    && path.is_file()
                                    && (entry.content.ends_with(".png") || entry.content.ends_with(".jpg"))
                                {
                                    let texture: &TextureHandle = {
                                        let entry = self.image_cache.entry(path.clone());
                                        entry.or_insert_with(|| {
                                            println!("🔄 Lade Bild: {:?}", path);

                                            match std::fs::read(&path) {
                                                Ok(image_data) if !image_data.is_empty() => {
                                                    match image::load_from_memory(&image_data) {
                                                        Ok(img) => {
                                                            let img = img.to_rgba8();
                                                            let size = [img.width() as _, img.height() as _];
                                                            println!("✅ Bild erfolgreich geladen: {:?}, Größe: {:?}", path, size);

                                                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                                                size,
                                                                img.as_flat_samples().as_slice(),
                                                            );
                                                            ctx.load_texture(
                                                                path.to_string_lossy(),
                                                                color_image,
                                                                egui::TextureOptions::default(),
                                                            )
                                                        }
                                                        Err(e) => {
                                                            eprintln!("❌ Fehler beim Dekodieren: {:?}: {}", path, e);
                                                            Self::fallback_texture(ctx, &path)
                                                        }
                                                    }
                                                }
                                                Ok(_) => {
                                                    eprintln!("❌ Bilddatei ist leer: {:?}", path);
                                                    Self::fallback_texture(ctx, &path)
                                                }
                                                Err(e) => {
                                                    eprintln!("❌ Fehler beim Lesen: {:?}: {}", path, e);
                                                    Self::fallback_texture(ctx, &path)
                                                }
                                            }
                                        })
                                    };

                                    ui.add(egui::Image::new(texture).max_height(150.0).max_width(400.0));
                                } else {
                                    // 👉 Kein Bild: Platzhalter
                                    ui.label("-");
                                }

                                // ✅ Zeilenabschluss
                                ui.end_row();

                                // ✅ scroll_to_me & clicked Handling
                                if sel {
                                    response.scroll_to_me(Some(egui::Align::Center));
                                }
                                if response.clicked() {
                                    self.select_entry(i);
                                }
                            }
                        });
                });
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
