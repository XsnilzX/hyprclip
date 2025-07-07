use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

static IGNORE_SINCE: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static SKIP_IMAGE_HASH: OnceLock<Mutex<Option<u64>>> = OnceLock::new();

pub fn set_ignore_flag() {
    let lock = IGNORE_SINCE.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().unwrap();
    *guard = Some(Instant::now());
}

pub fn should_ignore_recently(threshold: Duration) -> bool {
    let lock = IGNORE_SINCE.get_or_init(|| Mutex::new(None));
    let guard = lock.lock().unwrap();
    if let Some(instant) = *guard {
        if instant.elapsed() < threshold {
            return true;
        }
    }
    false
}

pub fn set_skip_image_hash(hash: u64) {
    let lock = SKIP_IMAGE_HASH.get_or_init(|| Mutex::new(None));
    *lock.lock().unwrap() = Some(hash);
}

pub fn take_skip_image_hash() -> Option<u64> {
    let lock = SKIP_IMAGE_HASH.get_or_init(|| Mutex::new(None));
    lock.lock().unwrap().take()
}
