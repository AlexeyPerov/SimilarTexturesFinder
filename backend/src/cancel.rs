use std::sync::atomic::{AtomicBool, Ordering};

static CANCELLED: AtomicBool = AtomicBool::new(false);

pub fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::Relaxed)
}

pub fn setup_cancel_handler() {
    ctrlc::set_handler(|| {
        CANCELLED.store(true, Ordering::Relaxed);
    })
    .expect("ctrlc handler");
}
