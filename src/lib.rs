mod config;
mod metrics;
mod worker;

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use pyo3::prelude::*;
use tokio_util::sync::CancellationToken;

use crate::config::{LoadProfile, TestConfig};
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
                let token = CancellationToken::new();
                handles.push((
                    tokio::spawn(worker::worker_loop(
                        client.clone(),
                        url.clone(),
                        Arc::clone(&counters),
                        tx.clone(),
                        duration,
                        token.clone(),
                    )),
                    token,
                ));
            }

            drop(tx);

            for (handle, _token) in handles {
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

#[pyfunction]
fn run_load_profiles(
    py: Python<'_>,
    url: String,
    timeout_secs: u64,
    profile: LoadProfile,
) -> PyResult<metrics::TestSummary> {
    py.detach(move || {
        let max_concurrency = profile.max_concurrency();
        let total_duration_secs = profile.total_duration();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        rt.block_on(async move {
            let client = reqwest::Client::builder()
                .pool_max_idle_per_host(max_concurrency)
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            let counters = Arc::new(LiveCounters::new());
            let total_duration = Duration::from_secs(total_duration_secs);

            let (tx, rx) = tokio::sync::mpsc::channel::<RequestMetric>(8192);

            let aggregator = tokio::spawn(async move {
                let mut latencies = Vec::new();
                let mut rx = rx;
                while let Some(metric) = rx.recv().await {
                    latencies.push(metric.latency_micros);
                }
                latencies
            });

            let counters_clone = Arc::clone(&counters);
            let client_clone = client.clone();
            let url_clone = url.clone();

            let supervisor = tokio::spawn(async move {
                let mut tokens: Vec<CancellationToken> = Vec::new();
                let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();
                let mut current_concurrency = 0usize;
                let start = Instant::now();

                loop {
                    let elapsed = start.elapsed();
                    if elapsed >= total_duration {
                        break;
                    }

                    let target = profile.target_concurrency(elapsed);

                    while current_concurrency < target {
                        let child_token = CancellationToken::new();
                        let remaining = total_duration.saturating_sub(elapsed);
                        let handle = tokio::spawn(worker::worker_loop(
                            client_clone.clone(),
                            url_clone.clone(),
                            Arc::clone(&counters_clone),
                            tx.clone(),
                            remaining,
                            child_token.clone(),
                        ));
                        tokens.push(child_token);
                        handles.push(handle);
                        current_concurrency += 1;
                    }

                    while current_concurrency > target {
                        if let Some(token) = tokens.pop() {
                            token.cancel();
                            if let Some(handle) = handles.pop() {
                                let _ = handle.await;
                            }
                            current_concurrency -= 1;
                        }
                    }

                    tokio::time::sleep(Duration::from_millis(200)).await;
                }

                for token in tokens {
                    token.cancel();
                }
                for handle in handles {
                    let _ = handle.await;
                }

                drop(tx);
            });

            supervisor
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

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
    m.add_function(wrap_pyfunction!(run_load_profiles, m)?)?;
    m.add_class::<config::TestConfig>()?;
    m.add_class::<config::LoadProfile>()?;
    m.add_class::<metrics::TestSummary>()?;
    Ok(())
}
