use crate::history::ClipboardItem;
use wl_clipboard_rs::copy::{MimeType, Options, Source};

pub fn set_clipboard_item(item: &ClipboardItem) -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::new();

    match item {
        ClipboardItem::Text(text) => {
            opts.copy(
                Source::Bytes(text.clone().into_bytes().into()),
                MimeType::Autodetect,
            )?;
        }
        ClipboardItem::Image(path) => {
            let data = std::fs::read(path)?;
            opts.copy(
                Source::Bytes(data.into()),
                MimeType::Specific("image/png".into()),
            )?;
        }
    }

    Ok(())
}
