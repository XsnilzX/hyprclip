use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

/// Benutzerkonfiguration für Hyprclip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximal gespeicherte Einträge im Clipboard-Verlauf
    pub history_limit: usize,
    /// Farbmodus für die UI
    pub theme: Theme,
    /// Pfad zur Datei, in der der Verlauf gespeichert wird
    pub storage_path: PathBuf,
    /// Pfad zur Datei, in der Bilder gespeichert werden
    pub image_storage_path: PathBuf,
}

/// Darstellungstypen für die GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            history_limit: 50,
            theme: Theme::System,
            storage_path: Self::default_storage_path(),
            image_storage_path: Self::default_image_storage_path(),
        }
    }
}

impl Config {
    /// Gibt den Pfad zurück, unter dem die Konfigurationsdatei gespeichert ist
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hyprclip")
            .join("config.toml")
    }

    /// Definierter Standardpfad für den Clipboard-Verlauf
    fn default_storage_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hyprclip")
            .join("clipboard.json")
    }

    fn default_image_storage_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hyprclip")
            .join("images")
    }

    /// Lädt die Konfiguration oder erstellt eine neue mit Default-Werten
    pub fn load_or_create() -> Self {
        let path = Self::path();

        if path.exists() {
            match fs::read_to_string(&path)
                .map_err(|e| eprintln!("⚠️  Konnte Konfigurationsdatei nicht lesen: {e}"))
                .and_then(|contents| {
                    toml::from_str(&contents).map_err(|e| eprintln!("⚠️  Fehler beim Parsen: {e}"))
                }) {
                Ok(cfg) => cfg,
                Err(_) => {
                    eprintln!("⚠️  Lade Default-Konfiguration stattdessen");
                    Self::default()
                }
            }
        } else {
            let default = Self::default();
            if let Err(e) = default.save() {
                eprintln!("⚠️  Konnte Default-Konfiguration nicht speichern: {e}");
            }
            default
        }
    }

    /// Speichert die aktuelle Konfiguration in die Datei
    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, toml::to_string_pretty(self).unwrap())?;
        Ok(())
    }
}
