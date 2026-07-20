use std::sync::atomic::AtomicUsize;

pub struct RequestMetric {
    pub status_code: u16,
    pub latency_micros: u128,
    pub is_error: bool,
}

pub struct LiveCounters {
    pub total_requests: AtomicUsize,
    pub errors: AtomicUsize,
}

impl LiveCounters {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
        }
    }
}
