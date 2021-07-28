use core::sync::atomic::{AtomicBool, Ordering::SeqCst};

#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicButtonState(AtomicBool);

impl AtomicButtonState {
    pub const fn new() -> Self {
        AtomicButtonState(AtomicBool::new(false))
    }

    pub fn set(&self) {
        self.0.store(true, SeqCst);
    }

    pub fn get_and_clear(&self) -> bool {
        self.0.swap(false, SeqCst)
    }
}
