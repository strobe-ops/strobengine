use std::sync::atomic::AtomicUsize;

use pyo3::prelude::*;

#[allow(dead_code)]
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

#[pyclass(skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct TestSummary {
    #[pyo3(get)]
    pub total_requests: usize,
    #[pyo3(get)]
    pub total_errors: usize,
    #[pyo3(get)]
    pub average_latency_ms: f64,
    #[pyo3(get)]
    pub p95_latency_ms: f64,
    #[pyo3(get)]
    pub p99_latency_ms: f64,
}

pub fn calculate_summary(total: usize, errors: usize, mut latencies: Vec<u128>) -> TestSummary {
    if latencies.is_empty() {
        return TestSummary {
            total_requests: total,
            total_errors: errors,
            average_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
        };
    }

    latencies.sort_unstable();

    let len = latencies.len();
    let sum: u128 = latencies.iter().sum();
    let average_latency_ms = sum as f64 / len as f64 / 1000.0;

    let p95_idx = (len * 95 / 100).min(len - 1);
    let p99_idx = (len * 99 / 100).min(len - 1);

    TestSummary {
        total_requests: total,
        total_errors: errors,
        average_latency_ms,
        p95_latency_ms: latencies[p95_idx] as f64 / 1000.0,
        p99_latency_ms: latencies[p99_idx] as f64 / 1000.0,
    }
}
