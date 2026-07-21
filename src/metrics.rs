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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_latencies_returns_zeros() {
        let s = calculate_summary(10, 3, vec![]);
        assert_eq!(s.total_requests, 10);
        assert_eq!(s.total_errors, 3);
        assert_eq!(s.average_latency_ms, 0.0);
        assert_eq!(s.p95_latency_ms, 0.0);
        assert_eq!(s.p99_latency_ms, 0.0);
    }

    #[test]
    fn single_request() {
        let s = calculate_summary(1, 0, vec![5000]);
        assert_eq!(s.total_requests, 1);
        assert_eq!(s.average_latency_ms, 5.0);
        assert_eq!(s.p95_latency_ms, 5.0);
        assert_eq!(s.p99_latency_ms, 5.0);
    }

    #[test]
    fn two_requests() {
        let s = calculate_summary(2, 0, vec![1000, 2000]);
        assert_eq!(s.average_latency_ms, 1.5);
        // p95_idx = (2 * 95 / 100).min(1) = 1, value = 2000
        assert_eq!(s.p95_latency_ms, 2.0);
        // p99_idx = (2 * 99 / 100).min(1) = 1, value = 2000
        assert_eq!(s.p99_latency_ms, 2.0);
    }

    #[test]
    fn uniform_hundred_values() {
        let latencies: Vec<u128> = (1..=100).collect();
        let s = calculate_summary(100, 0, latencies);
        // sum = 5050, average = 5050 / 100 / 1000 = 0.0505
        assert!((s.average_latency_ms - 0.0505).abs() < 1e-6);
        // p95 index = 95, value = 96 microseconds = 0.096 ms
        assert!((s.p95_latency_ms - 0.096).abs() < 1e-6);
        // p99 index = 99, value = 100 microseconds = 0.1 ms
        assert!((s.p99_latency_ms - 0.1).abs() < 1e-6);
    }

    #[test]
    fn all_errors() {
        let s = calculate_summary(5, 5, vec![100, 200, 300]);
        assert_eq!(s.total_requests, 5);
        assert_eq!(s.total_errors, 5);
    }

    #[test]
    fn microsecond_to_millisecond_conversion() {
        let s = calculate_summary(1, 0, vec![12345]);
        assert!((s.average_latency_ms - 12.345).abs() < 1e-6);
    }

    #[test]
    fn unsorted_latencies_are_sorted() {
        let s = calculate_summary(3, 0, vec![3000, 1000, 2000]);
        // sorted = [1000, 2000, 3000], p95_idx = 2, p99_idx = 2
        assert_eq!(s.p95_latency_ms, 3.0);
        assert_eq!(s.p99_latency_ms, 3.0);
    }
}
