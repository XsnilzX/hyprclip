use crate::error::AnyResult;
use crate::util::hash_data;
use crate::{
    clipboard_state,
    config::Config,
    history::{ClipboardItem, History},
};
use chrono::Local;
use image::{ImageBuffer, Rgba};
use std::{
    fs,
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
    let mut last_item: Option<ClipboardItem> = None;

    let image_dir = PathBuf::from(&config.image_storage_path);
    fs::create_dir_all(&image_dir).expect("📁 Bildverzeichnis konnte nicht erstellt werden.");

    loop {
        // ✅ 1. Ignore prüfen (timestamp-based)
        if clipboard_state::should_ignore_recently(Duration::from_millis(500)) {
            // Änderung stammt von uns selbst → ignorieren
            println!("⚠️ Ignoriere Clipboard-Event wegen kürzlichem self-set.");
            sleep(Duration::from_millis(200)).await;
            continue;
        }

        if last_item.is_some() && clipboard_is_empty() {
            if let Some(item) = &last_item {
                if let Err(e) = crate::clipboard::set_clipboard_item(item) {
                    eprintln!("⚠️ Fehler beim erneuten Setzen des Clipboards: {}", e);
                } else {
                    println!("🔄 Clipboard wiederhergestellt.");
                }
            }
            sleep(Duration::from_millis(200)).await;
            continue;
        }

        let now = Instant::now();

        // ✅ 2. TEXT
        if let Some(text) = get_clipboard_text() {
            let hash = hash_data(&text);

            let mut hist_guard = history.lock().unwrap();
            let is_duplicate = hist_guard.entries.iter().any(|e| e.hash == Some(hash));
            let limit = hist_guard.limit;

            if Some(hash) != last_text_hash
                && !is_duplicate
                && now.duration_since(last_text_change) >= debounce_delay
            {
                println!("📝 Neuer Texteingang: {}", text);
                last_text_hash = Some(hash);
                last_text_change = now;

                let mut hist = History::load(&config.storage_path, limit);
                hist.add_text(text.clone());
                if let Err(err) = hist.save(&config.storage_path) {
                    eprintln!("⚠️ Fehler beim Speichern (Text): {}", err);
                }

                *hist_guard = hist;

                let item = ClipboardItem::Text(text.clone());
                if let Err(e) = crate::clipboard::set_clipboard_item(&item) {
                    eprintln!("⚠️ Fehler beim Setzen des Textes ins Clipboard: {}", e);
                } else {
                    last_item = Some(item);
                }
            }
        }

        // ✅ 3. BILD
        if let Some(image_data) = get_clipboard_image() {
            let hash = hash_data(&image_data);

            // ✅ Skip hash prüfen und konsumieren
            if let Some(skip_hash) = crate::clipboard_state::take_skip_image_hash() {
                if skip_hash == hash {
                    println!("⚠️ Skip Bild (skip_hash match: {:x})", hash);
                    sleep(Duration::from_millis(200)).await;
                    continue;
                }
            }

            // ✅ Skip, wenn exakt gleicher Hash wie zuletzt erkannt
            if Some(hash) == last_image_hash
                || history
                    .lock()
                    .unwrap()
                    .entries
                    .iter()
                    .any(|e| e.hash == Some(hash))
            {
                println!("⚠️ Skip Bild (Hash {:x} bereits bekannt).", hash);
                sleep(Duration::from_millis(200)).await;
                continue;
            }

            if now.duration_since(last_image_change) >= debounce_delay {
                println!("🖼️ Neues Bild erkannt (Hash: {:x})", hash);
                last_image_hash = Some(hash);
                last_image_change = now;

                match save_image_as_png(&image_data, &image_dir, hash) {
                    Ok(path) => {
                        println!("🖼️ Bild gespeichert unter {}", path.display());

                        let mut hist =
                            History::load(&config.storage_path, history.lock().unwrap().limit);
                        hist.add_image(path.clone(), hash);
                        if let Err(err) = hist.save(&config.storage_path) {
                            eprintln!("⚠️ Fehler beim Speichern (Bild): {}", err);
                        }

                        *history.lock().unwrap() = hist;

                        // ✅ Setze skip hash bevor wir Clipboard setzen
                        crate::clipboard_state::set_skip_image_hash(hash);

                        // ✅ Set ignore flag
                        crate::clipboard_state::set_ignore_flag();

                        // ✅ Clipboard erneut setzen
                        let item = ClipboardItem::Image(path.clone());
                        if let Err(e) = crate::clipboard::set_clipboard_item(&item) {
                            eprintln!("⚠️ Fehler beim Setzen des Bildes ins Clipboard: {}", e);
                        } else {
                            last_item = Some(item);
                        }
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

fn clipboard_is_empty() -> bool {
    get_clipboard_text().is_none() && get_clipboard_image().is_none()
}

fn save_image_as_png(data: &[u8], dir: &PathBuf, hash: u64) -> AnyResult<PathBuf> {
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
