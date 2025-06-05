use std::io::Read;
use wl_clipboard_rs::copy::{self, Source};
use wl_clipboard_rs::paste::{self, ClipboardType};

#[tokio::main]
async fn main() {
    // Clipboard auslesen
    let (mut reader, _) = paste::get_contents(ClipboardType::Regular).unwrap();
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();

    println!("Clipboard Inhalt: {}", contents);
}
