use crate::{clipboard_state, history::ClipboardItem, util::hash_data};
use wl_clipboard_rs::copy::{MimeType, Options, Source};

pub fn set_clipboard_item(item: &ClipboardItem) -> Result<(), Box<dyn std::error::Error>> {
    set_clipboard_item_internal(item, true)
}

pub fn set_clipboard_item_no_ignore(
    item: &ClipboardItem,
) -> Result<(), Box<dyn std::error::Error>> {
    set_clipboard_item_internal(item, false)
}

fn set_clipboard_item_internal(
    item: &ClipboardItem,
    set_ignore: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options::new();

    match item {
        ClipboardItem::Text(text) => {
            if set_ignore {
                clipboard_state::set_ignore_flag();
            }
            opts.copy(
                Source::Bytes(text.clone().into_bytes().into()),
                MimeType::Autodetect,
            )?;
        }
        ClipboardItem::Image(path) => {
            let data = std::fs::read(path)?;
            let hash = hash_data(&data);
            if set_ignore {
                clipboard_state::set_skip_image_hash(hash);
                clipboard_state::set_ignore_flag();
            }
            opts.copy(
                Source::Bytes(data.into()),
                MimeType::Specific("image/png".into()),
            )?;
        }
    }

    Ok(())
}
