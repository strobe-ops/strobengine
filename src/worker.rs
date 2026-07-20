use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::metrics::{LiveCounters, RequestMetric};

pub async fn worker_loop(
    client: reqwest::Client,
    url: String,
    counters: Arc<LiveCounters>,
    tx: tokio::sync::mpsc::Sender<RequestMetric>,
    duration: Duration,
) {
    let start = Instant::now();

    while start.elapsed() < duration {
        counters.total_requests.fetch_add(1, Ordering::Relaxed);

        let req_start = Instant::now();
        let (status_code, is_error) = match client.get(&url).send().await {
            Ok(res) => {
                let code = res.status().as_u16();
                let errored = !res.status().is_success();
                if errored {
                    counters.errors.fetch_add(1, Ordering::Relaxed);
                }
                (code, errored)
            }
            Err(_) => {
                counters.errors.fetch_add(1, Ordering::Relaxed);
                (0, true)
            }
        };

        let metric = RequestMetric {
            status_code,
            latency_micros: req_start.elapsed().as_micros(),
            is_error,
        };

        let _ = tx.send(metric).await;
    }
}
