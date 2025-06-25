use std::sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
};

static IGNORE_NEXT_CLIPBOARD: OnceLock<AtomicBool> = OnceLock::new();

pub fn set_ignore_flag() {
    IGNORE_NEXT_CLIPBOARD
        .get_or_init(|| AtomicBool::new(true))
        .store(true, Ordering::Relaxed);
}

pub fn take_ignore_flag() -> bool {
    IGNORE_NEXT_CLIPBOARD
        .get_or_init(|| AtomicBool::new(false))
        .swap(false, Ordering::Relaxed)
}
