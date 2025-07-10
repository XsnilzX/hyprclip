use crate::{config::Config, error::AnyResult, history::History};
use serde_json::json;

pub async fn run() -> AnyResult<()> {
    let cfg = Config::load_or_create();
    let history = History::load(&cfg.storage_path, cfg.history_limit);
    let count = history.entries.len();
    let output = json!({
        "text": "📋",
        "alt": "hyprclip",
        "tooltip": count,
        "class": "icon_code"
    });

    println!("{}", output.to_string());
    Ok(())
}
