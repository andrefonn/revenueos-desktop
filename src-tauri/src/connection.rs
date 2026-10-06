use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct RequestGeneration(AtomicU64);

impl RequestGeneration {
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub fn begin(&self) -> u64 {
        self.0.fetch_add(1, Ordering::SeqCst).wrapping_add(1)
    }

    pub fn invalidate(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.0.load(Ordering::SeqCst) == generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_connection_attempt_invalidates_first_response() {
        let requests = RequestGeneration::new();
        let first = requests.begin();
        assert!(requests.is_current(first));
        let second = requests.begin();
        assert!(!requests.is_current(first));
        assert!(requests.is_current(second));
    }

    #[test]
    fn explicit_navigation_invalidates_pending_response() {
        let requests = RequestGeneration::new();
        let pending = requests.begin();
        requests.invalidate();
        assert!(!requests.is_current(pending));
        let retry = requests.begin();
        assert!(requests.is_current(retry));
        assert!(!requests.is_current(pending));
    }
}
