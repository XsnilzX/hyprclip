mod clipboard;
mod config;
mod history;
mod ui;
mod waybar;

use clap::Parser;
use config::Config;
use history::History;
use std::sync::Arc;

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

    /// Starte den Hintergrunddienst zur Clipboard-Überwachung
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
    use std::sync::{Arc, Mutex};

    let cli = Cli::parse();
    let cfg = config::Config::load_or_create();

    // 📋 Verlauf vorbereiten
    let mut history = history::History::load(&cfg.storage_path, cfg.history_limit);

    // 🧹 Verlauf löschen
    if cli.clear {
        history.clear();
        history.save(&cfg.storage_path)?;
        println!("✅ Verlauf gelöscht.");
        return Ok(());
    }

    // 📤 Verlauf exportieren
    if cli.export {
        let json = history.export_json()?;
        println!("{json}");
        return Ok(());
    }

    // 🔍 Verlauf durchsuchen
    if let Some(keyword) = cli.search {
        let results = history.search(&keyword);
        if results.is_empty() {
            println!("🔍 Keine Treffer für „{keyword}“");
        } else {
            println!("🔍 Treffer für „{keyword}“:");
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

    // ▶️ Watcher starten, falls gewünscht
    if cli.watch {
        let history = Arc::new(Mutex::new(history));
        let history_clone = Arc::clone(&history);
        let config_clone = cfg.clone();

        tokio::spawn(async move {
            clipboard::watch::watch_clipboard(history_clone, config_clone).await;
        });

        // Optional: GUI dazu starten
        ui::launch_with_history(Arc::clone(&history), cfg.storage_path.clone()).await?;
    }

    Ok(())
}
