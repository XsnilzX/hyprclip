mod clipboard;
mod config;
mod history;
mod ui;
mod waybar;

use clap::Parser;
use config::Config;
use history::History;
use std::{
    fs::{File, OpenOptions},
    path::Path,
    sync::{Arc, Mutex},
};

/// Hyprclip – Clipboard Manager mit GUI und Waybar-Integration
#[derive(Parser)]
#[command(name = "hyprclip")]
#[command(version = "0.1.0")]
#[command(about = "Clipboard Manager mit GUI, Waybar-Modul und Watcher", long_about = None)]
struct Cli {
    /// Starte im Waybar-Modul-Modus (gibt JSON aus)
    #[arg(long)]
    waybar: bool,

    /// Starte den Hintergrunddienst zur Clipboard-Überwachung
    #[arg(long)]
    watch: bool,

    /// Starte die GUI
    #[arg(long)]
    gui: bool,

    /// Lösche den Verlauf
    #[arg(long)]
    clear: bool,

    /// Exportiert den Verlauf als JSON
    #[arg(long)]
    export: bool,

    /// Sucht im Verlauf nach einem Schlüsselwort
    #[arg(long)]
    search: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cfg = Config::load_or_create();

    let history = Arc::new(Mutex::new(History::load(
        &cfg.storage_path,
        cfg.history_limit,
    )));

    // Wenn keine Flags gesetzt → automatisch GUI + Watch starten
    let nothing_specified =
        !cli.watch && !cli.gui && !cli.clear && !cli.export && cli.search.is_none() && !cli.waybar;
    let launch_gui = cli.gui || nothing_specified;
    let launch_watch = cli.watch || nothing_specified;

    // 🧹 Verlauf löschen
    if cli.clear {
        history.lock().unwrap().clear();
        history.lock().unwrap().save(&cfg.storage_path)?;
        println!("✅ Verlauf gelöscht.");
        return Ok(());
    }

    // 📤 Exportieren
    if cli.export {
        let json = history.lock().unwrap().export_json()?;
        println!("{json}");
        return Ok(());
    }

    // 🔍 Suche
    if let Some(keyword) = cli.search {
        let guard = history.lock().unwrap();
        let results = guard.search(&keyword);
        if results.is_empty() {
            println!("🔍 Keine Treffer für „{}“", keyword);
        } else {
            println!("🔍 Treffer für „{}“:", keyword);
            for entry in results {
                println!("- {}", entry.content);
            }
        }
        return Ok(());
    }

    // 📊 Waybar-Modus
    if cli.waybar {
        waybar::run().await?;
        return Ok(());
    }

    // ▶️ Watcher starten
    let mut _lock_file: Option<File> = None;
    if launch_watch {
        match check_watcher_lock() {
            Some(lock) => {
                _lock_file = Some(lock);
                let h = Arc::clone(&history);
                let c = cfg.clone();
                tokio::spawn(async move {
                    clipboard::watch::watch_clipboard(h, c).await;
                });
            }
            None => {
                eprintln!("⚠️ Watcher läuft bereits.");
                // Wenn explizit nur --watch gesetzt → abbrechen
                if cli.watch && !cli.gui {
                    return Ok(());
                }
            }
        }
    }

    // 🖼️ GUI starten
    if launch_gui {
        ui::launch_with_history(Arc::clone(&history), cfg.storage_path.clone())?;
    } else if cli.watch {
        // Nur Watcher: laufend halten, bis Ctrl+C
        println!("📋 Watcher läuft... (Beenden mit Ctrl+C)");
        tokio::signal::ctrl_c().await?;
        println!("👋 Beendet.");
    }

    // Lock-Datei beim Beenden löschen
    if _lock_file.is_some() {
        std::fs::remove_file("/tmp/hyprclip.lock").ok();
    }

    Ok(())
}

/// Erstellt eine Lock-Datei, um Mehrfach-Start zu verhindern
fn check_watcher_lock() -> Option<File> {
    let lock_path = "/tmp/hyprclip.lock";
    if Path::new(lock_path).exists() {
        return None;
    }

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)
    {
        Ok(file) => Some(file),
        Err(_) => None,
    }
}
