use crate::{error::AnyResult, watcher::get_latest_entry};
use serde_json::json;

pub async fn run() -> AnyResult<()> {
    let latest_clip = get_latest_entry()?;
    let output = json!({
        "text": "📋",
        "alt": "hyprclip",
        "tooltip": latest_clip,
        "class": "icon_code"
    });

    println!("{}", output.to_string());
    Ok(())
}
