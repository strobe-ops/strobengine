use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod chaos;
mod config;
mod logging;
mod metrics;
mod progress;
mod worker;

use std::io::IsTerminal;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use http::Method;
use http::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use pyo3::prelude::*;
use tokio_util::sync::CancellationToken;

use crate::chaos::ChaosEngine;
use crate::config::{LoadProfile, TestConfig};
use crate::metrics::{LiveCounters, RequestMetric};

fn parse_method(method_str: &str) -> PyResult<Method> {
    method_str.to_uppercase().parse().map_err(|_| {
        pyo3::exceptions::PyValueError::new_err(format!("Invalid HTTP method: {method_str}"))
    })
}

fn parse_body(body: Option<String>) -> Option<bytes::Bytes> {
    body.map(bytes::Bytes::from)
}

fn parse_headers(headers: Option<Vec<(String, String)>>) -> PyResult<HeaderMap> {
    let mut header_map = HeaderMap::new();

    if let Some(h) = headers {
        for (k, v) in h {
            let name = HeaderName::from_bytes(k.as_bytes()).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid header name '{k}': {e}"))
            })?;

            let val = HeaderValue::from_str(&v).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid header value for '{k}': {e}"
                ))
            })?;

            header_map.append(name, val);
        }
    }
    Ok(header_map)
}

fn build_client(
    concurrency: usize,
    timeout_secs: u64,
    header_map: HeaderMap,
) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .pool_max_idle_per_host(concurrency)
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(5))
        .tcp_nodelay(true)
        .http2_keep_alive_interval(Duration::from_secs(10))
        .http2_keep_alive_timeout(Duration::from_secs(5))
        .default_headers(header_map)
        .build()
}

#[pyfunction]
fn init_logging(level: String, log_file: Option<String>) {
    logging::init_tracing(&level, log_file.as_deref());
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn run_load_test(py: Python<'_>, config: TestConfig) -> PyResult<metrics::TestSummary> {
    py.detach(move || {
        let url = config.url;
        let concurrency = config.concurrency;
        let duration_secs = config.duration_secs;
        let timeout_secs = config.timeout_secs;
        let chaos = ChaosEngine::new(config.chaos, config.chaos_rate);
        let no_progress = config.no_progress;

        // Parse and validate HTTP method, body, headers before test starts
        let method = parse_method(&config.method)?;
        let body = parse_body(config.body);
        let mut header_map = parse_headers(config.headers)?;

        // Auto-insert Content-Type when body is present
        if body.is_some() && !header_map.contains_key(CONTENT_TYPE) {
            header_map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }

        tracing::info!(
            url,
            concurrency,
            duration_secs,
            timeout_secs,
            method = %method,
            chaos = config.chaos,
            "starting constant load test"
        );

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        // Build the client OUTSIDE the async closure
        let client = build_client(concurrency, timeout_secs, header_map)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        // Track whether cancellation was triggered by user SIGINT
        let interrupted = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let interrupted_check = Arc::clone(&interrupted);

        let result = rt.block_on(async move {
            tracing::debug!("http client created");

            let counters = Arc::new(LiveCounters::new());
            let duration = Duration::from_secs(duration_secs);
            let test_start = Instant::now();

            // Top-level cancellation token for SIGINT handling
            let cancel_token = CancellationToken::new();

            // Spawn SIGINT listener with double Ctrl+C safety hatch
            let token_clone = cancel_token.clone();
            let interrupted_clone = interrupted.clone();
            tokio::spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() {
                    tracing::info!("received SIGINT, initiating graceful shutdown");
                    interrupted_clone.store(true, Ordering::SeqCst);
                    token_clone.cancel();

                    if tokio::signal::ctrl_c().await.is_ok() {
                        tracing::warn!("received second SIGINT, shutdown already in progress");
                    }
                }
            });

            let (tx, rx) = tokio::sync::mpsc::channel::<RequestMetric>(8192);

            let aggregator = tokio::spawn(async move {
                let mut latencies = Vec::new();
                let mut rx = rx;
                while let Some(metric) = rx.recv().await {
                    latencies.push(metric.latency_micros);
                }
                latencies
            });

            // Spawn progress render task (only on TTY when enabled)
            let use_progress = !no_progress && std::io::stderr().is_terminal();
            let pb = if use_progress {
                Some(progress::create_progress_bar(duration))
            } else {
                None
            };
            let render_handle = pb.as_ref().map(|pb| {
                tokio::spawn(progress::render_loop(
                    pb.clone(),
                    Arc::clone(&counters),
                    test_start,
                    duration,
                    cancel_token.clone(),
                ))
            });

            let mut handles = Vec::with_capacity(concurrency);
            for _ in 0..concurrency {
                handles.push(tokio::spawn(worker::worker_loop(
                    client.clone(),
                    url.clone(),
                    method.clone(),
                    body.clone(),
                    Arc::clone(&counters),
                    tx.clone(),
                    duration,
                    cancel_token.clone(),
                    chaos,
                )));
            }

            tracing::debug!(workers = concurrency, "worker tasks spawned");

            drop(tx);

            for handle in handles {
                if let Err(e) = handle.await {
                    tracing::warn!(error = %e, "worker task panicked");
                }
            }

            // Wait for render task to finish naturally
            #[allow(clippy::collapsible_if)]
            if let Some(handle) = render_handle {
                if let Err(e) = handle.await {
                    tracing::debug!(error = %e, "render task panicked");
                }
            }

            let latencies = aggregator
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            let total = counters.total_requests.load(Ordering::Relaxed);
            let errors = counters.errors.load(Ordering::Relaxed);

            tracing::info!(total, errors, "constant load test completed");

            Ok::<metrics::TestSummary, pyo3::PyErr>(metrics::calculate_summary(
                total, errors, latencies,
            ))
        })?;

        // If SIGINT was received, raise KeyboardInterrupt to Python
        if interrupted_check.load(Ordering::SeqCst) {
            return Err(pyo3::exceptions::PyKeyboardInterrupt::new_err(
                "load test interrupted by user",
            ));
        }

        Ok(result)
    })
}

#[pyfunction]
#[pyo3(signature = (
    url,
    timeout_secs,
    profile,
    chaos=false,
    chaos_rate=crate::chaos::DEFAULT_CHAOS_RATE,
    no_progress=false,
    method="GET",
    body=None,
    headers=None,
))]
#[allow(clippy::too_many_arguments)]
fn run_load_profiles(
    py: Python<'_>,
    url: String,
    timeout_secs: u64,
    profile: LoadProfile,
    chaos: bool,
    chaos_rate: f32,
    no_progress: bool,
    method: &str,
    body: Option<String>,
    headers: Option<Vec<(String, String)>>,
) -> PyResult<metrics::TestSummary> {
    py.detach(move || {
        let max_concurrency = profile.max_concurrency();
        let total_duration_secs = profile.total_duration();
        let chaos_engine = ChaosEngine::new(chaos, chaos_rate);

        // Parse and validate HTTP method, body, headers before test starts
        let method = parse_method(method)?;
        let body = parse_body(body);
        let mut header_map = parse_headers(headers)?;

        // Auto-insert Content-Type when body is present
        if body.is_some() && !header_map.contains_key(CONTENT_TYPE) {
            header_map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }

        tracing::info!(
            url,
            timeout_secs,
            max_concurrency,
            total_duration_secs,
            method = %method,
            chaos,
            "starting profile load test"
        );

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let client = build_client(max_concurrency, timeout_secs, header_map)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        // Track whether cancellation was triggered by user SIGINT
        let interrupted = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let interrupted_check = Arc::clone(&interrupted);

        let result = rt.block_on(async move {
            tracing::debug!("http client created");

            let counters = Arc::new(LiveCounters::new());
            let total_duration = Duration::from_secs(total_duration_secs);
            let test_start = Instant::now();

            // Top-level cancellation token for SIGINT handling
            let cancel_token = CancellationToken::new();

            // Spawn SIGINT listener with double Ctrl+C safety hatch
            let token_clone = cancel_token.clone();
            let interrupted_clone = interrupted.clone();
            tokio::spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() {
                    tracing::info!("received SIGINT, initiating graceful shutdown");
                    interrupted_clone.store(true, Ordering::SeqCst);
                    token_clone.cancel();

                    if tokio::signal::ctrl_c().await.is_ok() {
                        tracing::warn!("received second SIGINT, shutdown already in progress");
                    }
                }
            });

            let (tx, rx) = tokio::sync::mpsc::channel::<RequestMetric>(8192);

            let aggregator = tokio::spawn(async move {
                let mut latencies = Vec::new();
                let mut rx = rx;
                while let Some(metric) = rx.recv().await {
                    latencies.push(metric.latency_micros);
                }
                latencies
            });

            // Spawn progress render task (only on TTY when enabled)
            let use_progress = !no_progress && std::io::stderr().is_terminal();
            let pb = if use_progress {
                Some(progress::create_progress_bar(total_duration))
            } else {
                None
            };
            let render_handle = pb.as_ref().map(|pb| {
                tokio::spawn(progress::render_loop(
                    pb.clone(),
                    Arc::clone(&counters),
                    test_start,
                    total_duration,
                    cancel_token.clone(),
                ))
            });

            let counters_clone = Arc::clone(&counters);
            let client_clone = client.clone();
            let url_clone = url.clone();
            let cancel_clone = cancel_token.clone();

            let supervisor = tokio::spawn(async move {
                let mut child_tokens: Vec<CancellationToken> = Vec::new();
                let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();
                let mut reaped_handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();
                let mut current_concurrency = 0usize;
                let start = Instant::now();

                loop {
                    let elapsed = start.elapsed();
                    if elapsed >= total_duration || cancel_clone.is_cancelled() {
                        break;
                    }

                    let target = profile.target_concurrency(elapsed);

                    while current_concurrency < target {
                        let child_token = CancellationToken::new();
                        let remaining = total_duration.saturating_sub(elapsed);
                        let handle = tokio::spawn(worker::worker_loop(
                            client_clone.clone(),
                            url_clone.clone(),
                            method.clone(),
                            body.clone(),
                            Arc::clone(&counters_clone),
                            tx.clone(),
                            remaining,
                            child_token.clone(),
                            chaos_engine,
                        ));
                        child_tokens.push(child_token);
                        handles.push(handle);
                        current_concurrency += 1;
                    }

                    while current_concurrency > target {
                        if let Some(token) = child_tokens.pop() {
                            token.cancel();
                            if let Some(handle) = handles.pop() {
                                reaped_handles.push(handle);
                            }
                            current_concurrency -= 1;
                        }
                    }

                    // Join reaped workers concurrently
                    for handle in reaped_handles.drain(..) {
                        if let Err(e) = handle.await {
                            tracing::debug!(error = %e, "cancelled worker panicked during shutdown");
                        }
                    }

                    tracing::debug!(current_concurrency, target, "supervisor tick");

                    tokio::time::sleep(Duration::from_millis(200)).await;
                }

                for token in child_tokens {
                    token.cancel();
                }
                for handle in handles {
                    if let Err(e) = handle.await {
                        tracing::warn!(error = %e, "worker task panicked");
                    }
                }

                drop(tx);
            });

            supervisor
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            // Wait for render task to finish naturally
            #[allow(clippy::collapsible_if)]
            if let Some(handle) = render_handle {
                if let Err(e) = handle.await {
                    tracing::debug!(error = %e, "render task panicked");
                }
            }

            let latencies = aggregator
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            let total = counters.total_requests.load(Ordering::Relaxed);
            let errors = counters.errors.load(Ordering::Relaxed);

            tracing::info!(total, errors, "profile load test completed");

            Ok::<metrics::TestSummary, pyo3::PyErr>(metrics::calculate_summary(
                total, errors, latencies,
            ))
        })?;

        // If SIGINT was received, raise KeyboardInterrupt to Python
        if interrupted_check.load(Ordering::SeqCst) {
            return Err(pyo3::exceptions::PyKeyboardInterrupt::new_err(
                "load test interrupted by user",
            ));
        }

        Ok(result)
    })
}

/// A Python module implemented in Rust. The name of this module must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn _strobengine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_logging, m)?)?;
    m.add_function(wrap_pyfunction!(run_load_test, m)?)?;
    m.add_function(wrap_pyfunction!(run_load_profiles, m)?)?;
    m.add_class::<config::TestConfig>()?;
    m.add_class::<config::LoadProfile>()?;
    m.add_class::<metrics::TestSummary>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn verify_allocator_runs() {
        let vec: Vec<u8> = vec![0; 1000];
        assert_eq!(vec.len(), 1000);
    }
}
