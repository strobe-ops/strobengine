mod config;
mod metrics;
mod worker;

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use pyo3::prelude::*;

use crate::config::TestConfig;
use crate::metrics::{LiveCounters, RequestMetric};

#[pyfunction]
fn run_load_test(py: Python<'_>, config: TestConfig) -> PyResult<metrics::TestSummary> {
    py.detach(move || {
        let url = config.url;
        let concurrency = config.concurrency;
        let duration_secs = config.duration_secs;
        let timeout_secs = config.timeout_secs;

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        rt.block_on(async move {
            let client = reqwest::Client::builder()
                .pool_max_idle_per_host(concurrency)
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            let counters = Arc::new(LiveCounters::new());
            let duration = Duration::from_secs(duration_secs);

            let (tx, rx) = tokio::sync::mpsc::channel::<RequestMetric>(8192);

            let aggregator = tokio::spawn(async move {
                let mut latencies = Vec::new();
                let mut rx = rx;
                while let Some(metric) = rx.recv().await {
                    latencies.push(metric.latency_micros);
                }
                latencies
            });

            let mut handles = Vec::with_capacity(concurrency);
            for _ in 0..concurrency {
                handles.push(tokio::spawn(worker::worker_loop(
                    client.clone(),
                    url.clone(),
                    Arc::clone(&counters),
                    tx.clone(),
                    duration,
                )));
            }

            drop(tx);

            for handle in handles {
                let _ = handle.await;
            }

            let latencies = aggregator
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            let total = counters.total_requests.load(Ordering::Relaxed);
            let errors = counters.errors.load(Ordering::Relaxed);

            Ok(metrics::calculate_summary(total, errors, latencies))
        })
    })
}

/// A Python module implemented in Rust. The name of this module must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn _strobengine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_load_test, m)?)?;
    m.add_class::<config::TestConfig>()?;
    m.add_class::<metrics::TestSummary>()?;
    Ok(())
}
