use crate::{config::Config, history::History};
use arboard::{Clipboard, ImageData};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;

/// Überwacht das Clipboard und speichert neue Einträge (Text und Bild) mit Debouncing.
pub async fn watch_clipboard(history: Arc<Mutex<History>>, config: Config) {
    println!("📋 Async Clipboard-Watcher gestartet...");
    let mut clipboard = Clipboard::new().expect("Clipboard konnte nicht initialisiert werden.");

    let mut last_text = String::new();
    let mut last_image_hash: Option<u64> = None;
    let mut last_change = Instant::now();
    let debounce_delay = Duration::from_millis(500); // Mindestzeit zwischen Änderungen

    loop {
        let now = Instant::now();

        // TEXT-Handling
        if let Ok(current) = clipboard.get_text() {
            if current != last_text && now.duration_since(last_change) >= debounce_delay {
                println!("📝 Neuer Texteingang: {}", current);
                last_text = current.clone();
                last_change = now;

                let mut hist = history.lock().unwrap();
                hist.add(current);
                if let Err(err) = hist.save(&config.storage_path) {
                    eprintln!("⚠️ Fehler beim Speichern (Text): {}", err);
                }
            }
        }

        // BILD-Handling
        if let Ok(image) = clipboard.get_image() {
            let hash = calculate_image_hash(&image);
            if Some(hash) != last_image_hash && now.duration_since(last_change) >= debounce_delay {
                println!("🖼️ Neues Bild erkannt (Hash: {:x})", hash);
                last_image_hash = Some(hash);
                last_change = now;

                let image_entry = format!("<Bild mit Hash {:x}>", hash);

                let mut hist = history.lock().unwrap();
                hist.add(image_entry);
                if let Err(err) = hist.save(&config.storage_path) {
                    eprintln!("⚠️ Fehler beim Speichern (Bild): {}", err);
                }
            }
        }

        sleep(Duration::from_secs(1)).await;
    }
}

/// Berechnet einen einfachen Hash für ein Bild (für Vergleich).
fn calculate_image_hash(image: &ImageData) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    image.bytes.hash(&mut hasher);
    image.width.hash(&mut hasher);
    image.height.hash(&mut hasher);
    hasher.finish()
}
