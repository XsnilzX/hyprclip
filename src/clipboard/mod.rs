use arboard::Clipboard;

pub fn get_latest_entry() -> Result<String, Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    let text = clipboard.get_text()?;
    Ok(text)
}
pub mod watch;
