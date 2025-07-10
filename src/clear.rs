use crate::{
    config::Config,
    history::{ClipboardItem, History},
};
use std::io::{self, ErrorKind, Write};

/// Entfernt alle Einträge aus der History und löscht zugehörige Bilddateien
pub fn clear_history(history: &mut History, cfg: &Config) -> std::io::Result<()> {
    if ask_yes_no("Do you realy want to delete History?") {
        for entry in &history.entries {
            if let ClipboardItem::Image(ref path) = entry.item {
                if path.exists()
                    && path.is_file()
                    && (path.ends_with(".png") || path.ends_with(".jpg"))
                {
                    if let Err(e) = std::fs::remove_file(path) {
                        eprintln!("⚠️  Konnte Bild nicht löschen {}: {e}", path.display());
                    }
                }
            }
        }
        history.clear();
        if let Err(e) = history.save(&cfg.storage_path) {
            eprintln!("⚠️  Fehler beim Speichern der History: {}", e);
        }
        return Ok(());
    } else {
        Err(io::Error::new(ErrorKind::Other, "User aborted"))
    }
}

fn ask_yes_no(question: &str) -> bool {
    loop {
        print!("{} (y/n): ", question);
        // Flush stdout to ensure the question appears before input
        io::stdout().flush().unwrap();

        let mut answer = String::new();
        io::stdin().read_line(&mut answer).unwrap();

        match answer.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => {
                println!("Bitte 'y' oder 'n' eingeben.");
            }
        }
    }
}
