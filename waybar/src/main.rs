use serde::Serialize;
use std::collections::VecDeque;
use std::fs;

#[derive(Serialize)]
struct WaybarOutput {
    text: String,
    tooltip: String,
    class: String,
}

fn main() {
    let content: VecDeque<String> = fs::read_to_string("/tmp/hyprclip.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    if let Some(first) = content.front() {
        let output = WaybarOutput {
            text: format!("📋 {}", first.chars().take(20).collect::<String>()),
            tooltip: first.clone(),
            class: "clipboard".to_string(),
        };
        println!("{}", serde_json::to_string(&output).unwrap());
    }
}
