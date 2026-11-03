use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_CONTEXT: AtomicU64 = AtomicU64::new(1);
pub(super) fn new_context() -> u64 {
    NEXT_CONTEXT.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug)]
pub struct DiagnosticCheckpoint {
    pub(super) context: u64,
    pub(super) serial: usize,
}
impl DiagnosticCheckpoint {
    pub fn is_after(self, earlier: Self) -> bool {
        self.context == earlier.context && self.serial > earlier.serial
    }
}
