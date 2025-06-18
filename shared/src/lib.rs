use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::Write;

pub struct ClipStore {
    pub max: usize,
    pub data: VecDeque<String>,
}

impl ClipStore {
    pub fn new(max: usize) -> Self {
        Self {
            max,
            data: VecDeque::new(),
        }
    }

    pub fn push(&mut self, item: String) {
        if self.data.front() == Some(&item) {
            return;
        }
        if self.data.len() >= self.max {
            self.data.pop_back();
        }
        self.data.push_front(item);
        self.save_to_file();
    }

    pub fn save_to_file(&self) {
        let json = serde_json::to_string(&self.data).unwrap();
        let mut file = File::create("/tmp/hyprclip.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    pub fn load_from_file() -> VecDeque<String> {
        if let Ok(data) = fs::read_to_string("/tmp/hyprclip.json") {
            if let Ok(list) = serde_json::from_str(&data) {
                return list;
            }
        }
        VecDeque::new()
    }
}
