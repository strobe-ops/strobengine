use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tokio_util::sync::CancellationToken;

use crate::metrics::{LiveCounters, RequestMetric};

pub async fn worker_loop(
    client: reqwest::Client,
    url: String,
    counters: Arc<LiveCounters>,
    tx: tokio::sync::mpsc::Sender<RequestMetric>,
    duration: Duration,
    token: CancellationToken,
) {
    tracing::debug!("worker spawned");

    let start = Instant::now();

    while start.elapsed() < duration && !token.is_cancelled() {
        counters.total_requests.fetch_add(1, Ordering::Relaxed);

        let req_start = Instant::now();
        let (status_code, is_error) = match client.get(&url).send().await {
            Ok(res) => {
                let code = res.status().as_u16();
                let errored = !res.status().is_success();
                if errored {
                    counters.errors.fetch_add(1, Ordering::Relaxed);
                    tracing::warn!(status_code = code, "non-success HTTP status");
                }
                (code, errored)
            }
            Err(_) => {
                counters.errors.fetch_add(1, Ordering::Relaxed);
                tracing::warn!("request failed");
                (0, true)
            }
        };

        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                status = status_code,
                latency_us = req_start.elapsed().as_micros(),
                "request completed"
            );
        }

        let metric = RequestMetric {
            status_code,
            latency_micros: req_start.elapsed().as_micros(),
            is_error,
        };

        let _ = tx.send(metric).await;
    }
}
