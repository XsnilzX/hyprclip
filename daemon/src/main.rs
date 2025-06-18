use shared::ClipStore;
use std::process::Command;
use std::{thread, time::Duration};

fn get_clipboard() -> Option<String> {
    let output = Command::new("wl-paste").arg("--no-newline").output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

fn main() {
    let mut store = ClipStore::new(20);

    loop {
        if let Some(clip) = get_clipboard() {
            store.push(clip);
        }
        thread::sleep(Duration::from_secs(1));
    }
}
