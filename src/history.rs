use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClipboardItem {
    Text(String),
    Image(PathBuf),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub content: String, // Vorschau (z. B. "🖼 Bild gespeichert...")
    pub timestamp: u64,
    pub item: ClipboardItem, // NEU: Für das tatsächliche Clipboard-Setzen
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub entries: Vec<Entry>,
    pub limit: usize,
}

impl History {
    pub fn new(limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            limit,
        }
    }

    pub fn add_text(&mut self, text: String) {
        if self.entries.first().map(|e| &e.content) == Some(&text) {
            return;
        }

        let entry = Entry {
            content: text.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            item: ClipboardItem::Text(text),
        };
        self.entries.insert(0, entry);
        self.cleanup();
    }

    pub fn add_image(&mut self, image_path: PathBuf) {
        let content = format!("{}", image_path.display());

        if self.entries.first().map(|e| &e.content) == Some(&content) {
            return;
        }

        let entry = Entry {
            content,
            timestamp: chrono::Utc::now().timestamp() as u64,
            item: ClipboardItem::Image(image_path),
        };
        self.entries.insert(0, entry);
        self.cleanup();
    }

    fn cleanup(&mut self) {
        if self.entries.len() > self.limit {
            self.entries.truncate(self.limit);
        }
    }

    pub fn delete_entry(&mut self, index: usize) -> bool {
        if index < self.entries.len() {
            self.entries.remove(index);
            true
        } else {
            false
        }
    }

    /*
    pub fn latest(&self) -> Option<&Entry> {
        self.entries.first()
    }
    */

    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        fs::create_dir_all(path.parent().unwrap())?;
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load(path: &PathBuf, limit: usize) -> Self {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            if let Ok(mut history) = serde_json::from_reader::<_, History>(reader) {
                history.limit = limit;
                history.entries.truncate(limit);
                return history;
            }
        }
        History::new(limit)
    }

    /// Löscht den kompletten Clipboard-Verlauf
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Exportiert den Verlauf als JSON-String
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    /// Gibt alle Einträge zurück, die ein bestimmtes Stichwort enthalten
    pub fn search(&self, keyword: &str) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|e| e.content.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }
}
