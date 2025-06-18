use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub content: String,
    pub timestamp: u64,
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

    pub fn add(&mut self, content: String) {
        if self.entries.first().map(|e| &e.content) == Some(&content) {
            return; // kein Duplikat direkt hintereinander
        }

        let entry = Entry {
            content,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        self.entries.insert(0, entry);

        if self.entries.len() > self.limit {
            self.entries.truncate(self.limit);
        }
    }

    pub fn latest(&self) -> Option<&Entry> {
        self.entries.first()
    }

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
