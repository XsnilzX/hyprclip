use crate::{config::Config, history::History};
use chrono::Local;
use image::{ImageBuffer, Rgba};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;
use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};

pub async fn watch_clipboard(history: Arc<Mutex<History>>, config: Config) {
    println!("📋 Async Clipboard-Watcher (Wayland) gestartet...");

    let mut last_text_hash: Option<u64> = None;
    let mut last_image_hash: Option<u64> = None;
    let mut last_text_change = Instant::now();
    let mut last_image_change = Instant::now();
    let debounce_delay = Duration::from_millis(500);

    let image_dir = PathBuf::from(&config.image_storage_path);
    fs::create_dir_all(&image_dir).expect("📁 Bildverzeichnis konnte nicht erstellt werden.");

    loop {
        let now = Instant::now();

        // TEXT
        if crate::clipboard_state::take_ignore_flag() {
            // Diese Änderung stammt von uns selbst → ignoriere
        } else if let Some(text) = get_clipboard_text() {
            let hash = hash_data(&text);
            if Some(hash) != last_text_hash
                && now.duration_since(last_text_change) >= debounce_delay
            {
                println!("📝 Neuer Texteingang: {}", text);
                last_text_hash = Some(hash);
                last_text_change = now;

                let mut hist = History::load(&config.storage_path, history.lock().unwrap().limit);
                hist.add_text(text);
                if let Err(err) = hist.save(&config.storage_path) {
                    eprintln!("⚠️ Fehler beim Speichern (Text): {}", err);
                }

                // shared_history ersetzen
                *history.lock().unwrap() = hist;
            }
        }

        // BILD
        if crate::clipboard_state::take_ignore_flag() {
            // Ignoriere eigenes Bild
        } else if let Some(image_data) = get_clipboard_image() {
            let hash = hash_data(&image_data);
            if Some(hash) != last_image_hash
                && now.duration_since(last_image_change) >= debounce_delay
            {
                println!("🖼️ Neues Bild erkannt (Hash: {:x})", hash);
                last_image_hash = Some(hash);
                last_image_change = now;

                match save_image_as_png(&image_data, &image_dir, hash) {
                    Ok(path) => {
                        let msg = format!("🖼️ Bild gespeichert unter {}", path.display());
                        println!("{}", msg);

                        let mut hist =
                            History::load(&config.storage_path, history.lock().unwrap().limit);
                        hist.add_image(path.clone());
                        if let Err(err) = hist.save(&config.storage_path) {
                            eprintln!("⚠️ Fehler beim Speichern (Bild): {}", err);
                        }

                        // shared_history ersetzen
                        *history.lock().unwrap() = hist;
                    }
                    Err(e) => eprintln!("⚠️ Fehler beim Speichern des Bildes: {}", e),
                }
            }
        }

        sleep(Duration::from_millis(200)).await;
    }
}

fn get_clipboard_text() -> Option<String> {
    match get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text) {
        Ok((mut pipe, _)) => {
            let mut buf = String::new();
            if let Err(e) = pipe.read_to_string(&mut buf) {
                eprintln!("⚠️ Fehler beim Lesen des Textes aus der Zwischenablage: {e}");
                return None;
            }

            let trimmed = buf.trim();
            if trimmed.is_empty() {
                return None;
            }

            // 🔒 Variante 1: HTML mit <img> Tag ignorieren, um Endlosloop zu verhindern
            if trimmed.starts_with("<meta") && trimmed.contains("<img") {
                println!("⚠️ Ignoriere HTML-Zwischenablage mit <img> Tag, um Loop zu verhindern.");
                return None;
            }

            // 🔒 Variante 2: "0,0" ignorieren
            if trimmed == "0,0" {
                println!("⚠️ Ignoriere Zwischenablage-Eintrag '0,0' (Koordinaten-Placeholder).");
                return None;
            }

            Some(buf.to_string())
        }
        Err(e) => {
            eprintln!("⚠️ Fehler beim Zugriff auf die Zwischenablage (Text): {e}");
            None
        }
    }
}

fn get_clipboard_image() -> Option<Vec<u8>> {
    match get_contents(
        ClipboardType::Regular,
        Seat::Unspecified,
        MimeType::Specific("image/png".into()),
    ) {
        Ok((mut pipe, _)) => {
            let mut data = Vec::new();
            if let Err(e) = pipe.read_to_end(&mut data) {
                eprintln!("⚠️ Fehler beim Lesen des Bildes aus der Zwischenablage: {e}");
                return None;
            }
            if data.is_empty() {
                None
            } else {
                Some(data)
            }
        }
        Err(e) => {
            eprintln!("⚠️ Fehler beim Zugriff auf die Zwischenablage (Bild): {e}");
            None
        }
    }
}

fn hash_data<T: Hash>(data: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

fn save_image_as_png(
    data: &[u8],
    dir: &PathBuf,
    hash: u64,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(data)?.to_rgba8();
    let buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(img.width(), img.height(), img.into_raw())
            .ok_or("Ungültiges Bildformat")?;

    let filename = format!(
        "clip_{:x}_{}.png",
        hash,
        Local::now().format("%Y%m%d%H%M%S")
    );

    let path = dir.join(filename);
    buffer.save(&path)?;
    Ok(path)
}
