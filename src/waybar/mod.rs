use crate::watcher::get_latest_entry;
use serde_json::json;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let latest_clip = crate::watcher::get_latest_entry()?;
    let output = json!({
        "text": latest_clip,
        "tooltip": "Letzter Clipboard-Eintrag",
        "class": "hyprclip"
    });

    println!("{}", output.to_string());
    Ok(())
}
