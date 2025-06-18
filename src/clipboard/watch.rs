use crate::{config::Config, history::History};
use arboard::{Clipboard, ImageData};
use image::{ImageBuffer, Rgba};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;

/// Überwacht das Clipboard und speichert neue Einträge (Text + PNG) mit Debouncing.
pub async fn watch_clipboard(history: Arc<Mutex<History>>, config: Config) {
    println!("📋 Async Clipboard-Watcher gestartet...");
    let mut clipboard = Clipboard::new().expect("Clipboard konnte nicht initialisiert werden.");

    let mut last_text = String::new();
    let mut last_image_hash: Option<u64> = None;
    let mut last_change = Instant::now();
    let debounce_delay = Duration::from_millis(500);

    // Verzeichnis für gespeicherte Bilder vorbereiten
    let image_dir = PathBuf::from(&config.image_storage_path);
    fs::create_dir_all(&image_dir).expect("Bildverzeichnis konnte nicht erstellt werden.");

    loop {
        let now = Instant::now();

        // TEXT
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

        // BILD
        if let Ok(image) = clipboard.get_image() {
            let hash = calculate_image_hash(&image);
            if Some(hash) != last_image_hash && now.duration_since(last_change) >= debounce_delay {
                println!("🖼️ Neues Bild erkannt (Hash: {:x})", hash);
                last_image_hash = Some(hash);
                last_change = now;

                match save_image_as_png(&image, &image_dir, hash) {
                    Ok(path) => {
                        let msg = format!("🖼️ Bild gespeichert unter {}", path.display());
                        let mut hist = history.lock().unwrap();
                        hist.add(msg);
                        if let Err(err) = hist.save(&config.storage_path) {
                            eprintln!("⚠️ Fehler beim Speichern (Bild): {}", err);
                        }
                    }
                    Err(e) => eprintln!("⚠️ Fehler beim Speichern des Bildes: {}", e),
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

/// Speichert das Bild als PNG im Zielverzeichnis.
fn save_image_as_png(
    image: &ImageData,
    dir: &PathBuf,
    hash: u64,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
        image.width as u32,
        image.height as u32,
        image.bytes.to_vec(),
    )
    .ok_or("Ungültiges Bildformat")?;

    let filename = format!(
        "clip_{:x}_{}.png",
        hash,
        chrono::Local::now().format("%Y%m%d%H%M%S")
    );
    let path = dir.join(filename);
    buffer.save(&path)?;
    Ok(path)
}
