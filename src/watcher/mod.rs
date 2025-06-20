use std::io::Read;
use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};

/// Gibt den aktuellen Text im Wayland-Clipboard zurück.
pub fn get_latest_entry() -> Result<String, Box<dyn std::error::Error>> {
    let (mut pipe, _mime) =
        get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text)?;
    let mut buf = String::new();
    pipe.read_to_string(&mut buf)?;
    Ok(buf)
}

pub mod watch;
