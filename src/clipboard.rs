use crate::clipboard_state;
use crate::history::ClipboardItem;
use wl_clipboard_rs::copy::{MimeType, Options, Source};

pub fn set_clipboard_item(item: &ClipboardItem) -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::new();

    match item {
        ClipboardItem::Text(text) => {
            clipboard_state::set_ignore_flag();
            opts.copy(
                Source::Bytes(text.clone().into_bytes().into()),
                MimeType::Autodetect,
            )?;
        }
        ClipboardItem::Image(path) => {
            let data = std::fs::read(path)?;
            clipboard_state::set_ignore_flag();
            opts.copy(
                Source::Bytes(data.into()),
                MimeType::Specific("image/png".into()),
            )?;
        }
    }

    Ok(())
}
