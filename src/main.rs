mod clipboard;
mod clipboard_state;
mod config;
mod history;
mod ui;
mod util;
mod watcher;
mod waybar;

use clap::Parser;
use config::Config;
use history::History;
use std::{
    fs::OpenOptions,
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

    // 🔄 Aktionen mit sofortigem Rückgabewert
    if cli.clear {
        history.lock().unwrap().clear();
        history.lock().unwrap().save(&cfg.storage_path)?;
        println!("✅ Verlauf gelöscht.");
        return Ok(());
    }

    if cli.export {
        let json = history.lock().unwrap().export_json()?;
        println!("{json}");
        return Ok(());
    }

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

    if cli.waybar {
        waybar::run().await?;
        return Ok(());
    }

    if cli.watch {
        run_watcher(history, cfg).await?;
        return Ok(());
    }

    if cli.gui {
        ui::launch_with_history(Arc::clone(&history), cfg.storage_path.clone())?;
        return Ok(());
    }

    // ❓ Fallback wenn kein Flag gesetzt
    eprintln!("❗ Kein Modus gewählt. Starte mit --gui, --watch oder --help");
    Ok(())
}

// 🔐 Watcher-Modus mit Lockfile + Ctrl+C-Abbruch
async fn run_watcher(
    history: Arc<Mutex<History>>,
    cfg: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::signal;

    let lock_path = "/tmp/hyprclip.lock";

    // Lock-Datei exklusiv erstellen
    let _lock = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)
    {
        Ok(file) => file,
        Err(_) => {
            eprintln!("⚠️ Watcher läuft bereits (Lockfile vorhanden).");
            return Ok(());
        }
    };

    println!("📋 Watcher läuft... (Beenden mit Ctrl+C)");

    let watch_task = tokio::spawn({
        let h = Arc::clone(&history);
        let c = cfg.clone();
        async move {
            watcher::watch::watch_clipboard(h, c).await;
        }
    });

    // Auf Ctrl+C warten
    signal::ctrl_c().await?;
    println!("👋 Beenden...");

    // Lock-Datei aktiv löschen
    if let Err(e) = std::fs::remove_file(lock_path) {
        eprintln!("❌ Konnte Lock-Datei nicht löschen: {e}");
    } else {
        println!("🧹 Lock-Datei gelöscht.");
    }

    // Watcher-Task abbrechen (optional)
    watch_task.abort();

    Ok(())
}
