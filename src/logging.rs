use std::sync::Once;

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

static INIT_LOGGING: Once = Once::new();

pub fn init_tracing(level_str: &str, log_file: Option<&str>) {
    INIT_LOGGING.call_once(|| {
        let filter = EnvFilter::try_new(level_str).unwrap_or_else(|_| EnvFilter::new("warn"));

        let stderr_layer = fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(false)
            .compact();

        if let Some(path) = log_file
            && let Ok(file) = std::fs::File::create(path)
        {
            let file_layer = fmt::layer().with_writer(file).with_target(true).compact();

            tracing_subscriber::registry()
                .with(filter)
                .with(stderr_layer)
                .with(file_layer)
                .init();
            return;
        }

        tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer)
            .init();
    });
}
