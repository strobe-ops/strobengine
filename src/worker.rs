use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tokio_util::sync::CancellationToken;

use crate::chaos::{ChaosEngine, ChaosFault};
use crate::metrics::{LiveCounters, RequestMetric};

static CORRUPTED_BODY: &[u8] = b"{\"payload\": \"\\xff\\xfe\\xbd\\xef\"}";
static CHAOS_HEADER: &str = "x-chaos-fault";
static BAD_HEADER_VALUE: &str = "invalid-header-value";

/// RAII Guard that automatically decrements `active_workers` when dropped.
struct WorkerGuard(Arc<LiveCounters>);

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        self.0.active_workers.fetch_sub(1, Ordering::Relaxed);
    }
}

pub async fn worker_loop(
    client: reqwest::Client,
    url: String,
    counters: Arc<LiveCounters>,
    tx: tokio::sync::mpsc::Sender<RequestMetric>,
    duration: Duration,
    token: CancellationToken,
    chaos: ChaosEngine,
) {
    tracing::debug!("worker spawned");

    counters.active_workers.fetch_add(1, Ordering::Relaxed);
    let _guard = WorkerGuard(Arc::clone(&counters));

    // Pre-warm: establish TCP/TLS connection before measurement starts.
    tracing::trace!("pre-warming connection");
    let _ = client.get(&url).send().await;

    let start = Instant::now();

    while start.elapsed() < duration && !token.is_cancelled() {
        counters.total_requests.fetch_add(1, Ordering::Relaxed);

        let req_start = Instant::now();

        let request = match chaos.select_fault() {
            Some(ChaosFault::LatencySpike { duration_ms }) => {
                tracing::trace!(duration_ms, "chaos: latency spike injected");
                tokio::time::sleep(Duration::from_millis(duration_ms)).await;
                client.get(&url)
            }
            Some(ChaosFault::CorruptedPayload) => {
                tracing::trace!("chaos: corrupted payload injected");
                client
                    .post(&url)
                    .header(CHAOS_HEADER, "corrupted-payload")
                    .body(CORRUPTED_BODY)
            }
            Some(ChaosFault::MetadataCorruption) => {
                tracing::trace!("chaos: metadata corruption injected");
                client.get(&url).header(CHAOS_HEADER, BAD_HEADER_VALUE)
            }
            Some(ChaosFault::ConnectionDrop) => {
                tracing::trace!("chaos: connection drop injected");
                // 1ns timeout forces reqwest to immediately abort with a timeout error
                client.get(&url).timeout(Duration::from_nanos(1))
            }
            None => client.get(&url),
        };

        let (status_code, is_error) = match request.send().await {
            Ok(res) => {
                let code = res.status().as_u16();
                let errored = !res.status().is_success();
                // Drain response body to ensure clean connection return to pool
                let _ = res.bytes().await;
                if errored {
                    counters.errors.fetch_add(1, Ordering::Relaxed);
                    tracing::debug!(status_code = code, "non-success HTTP status");
                }
                (code, errored)
            }
            Err(_) => {
                counters.errors.fetch_add(1, Ordering::Relaxed);
                tracing::debug!("request failed");
                (0, true)
            }
        };

        let latency_micros = u64::try_from(req_start.elapsed().as_micros()).unwrap_or(u64::MAX);

        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                status = status_code,
                latency_us = latency_micros,
                "request completed"
            );
        }

        // Update live counters for progress rendering
        counters.completed_requests.fetch_add(1, Ordering::Relaxed);
        counters
            .latency_sum_micros
            .fetch_add(latency_micros, Ordering::Relaxed);
        counters.latency_count.fetch_add(1, Ordering::Relaxed);

        let metric = RequestMetric {
            status_code,
            latency_micros: latency_micros as u128,
            is_error,
        };

        let _ = tx.send(metric).await;
    }
}
