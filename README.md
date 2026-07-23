# strobengine

A high-performance HTTP load testing engine with a Python API and a bare-metal Rust core.

## Dependencies

- **Python** >= 3.13
- **Rust** (stable, with `cargo`)
- **uv** (Python package manager)

### Rust crates

| Crate | Version | Purpose |
|-------|---------|---------|
| pyo3 | 0.29 | Python FFI bindings (stable ABI, abi3-py313) |
| reqwest | 0.13 | HTTP client with connection pooling |
| tokio | 1.53 | Multi-threaded async runtime |
| tokio-util | 0.7 | CancellationToken for graceful worker shutdown |
| tracing | 0.1 | Structured logging instrumentation |
| tracing-subscriber | 0.3 | Log formatting and output (stderr/file) |

## Installation & Compilation

```bash
# Clone the repository
git clone https://github.com/riccione/strobengine.git
cd strobengine

# Build the native extension and install the package
uv sync
```

`uv sync` invokes [maturin](https://github.com/PyO3/maturin) under the hood, which compiles the Rust code into a native Python extension module and installs it into your virtual environment.

## Quick Start Usage

```python
from strobengine import StrobEngine, print_summary

# Constant load test
engine = StrobEngine(url="http://localhost:8080/api/health", concurrency=50, duration=30)
summary = engine.run()

# Ramp/stress test (10 -> 200 workers over 60s, hold 30s)
engine = StrobEngine.stress_test(
    "http://localhost:8080/api/health",
    start_concurrency=10, max_concurrency=200,
    ramp_duration=60, hold_duration=30,
)
summary = engine.run()

# Spike test (baseline 5 -> peak 500 -> back to 5)
engine = StrobEngine.spike_test(
    "http://localhost:8080/api/health",
    baseline=5, peak_concurrency=500,
    pre_spike_duration=5, spike_duration=10, post_spike_duration=5,
)
summary = engine.run()

print_summary(summary, url=engine._url, duration_secs=30)
```

For async contexts (FastAPI, Typer, etc.):

```python
summary = await engine.run_async()
```

## CLI Usage

```bash
# Constant load test (default subcommand)
strobengine http://localhost:8080/api/health -c 50 -d 30

# Ramp/stress test
strobengine stress http://localhost:8080/api/health --from 10 --to 500 --ramp 60 --hold 30

# Spike test
strobengine spike http://localhost:8080/api/health --baseline 5 --peak 1000 --pre-spike 5 --spike-duration 10 --post-spike 5

# JSON output for CI/CD
strobengine load http://localhost:8080/api/health --json
```

By default, this spawns **10 concurrent workers** for **10 seconds** with a **10-second request timeout**. Results are displayed as a formatted table with total requests, errors, requests/sec, and latency percentiles (avg, p95, p99).

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `load` | Constant load test (default if no subcommand given) |
| `stress` | Ramp from starting to target concurrency, then hold |
| `spike` | Baseline -> peak -> baseline |

### Load Subcommand Options

| Flag | Default | Description |
|------|---------|-------------|
| `-c`, `--concurrency` | `10` | Number of concurrent workers |
| `-d`, `--duration` | `10` | Duration in seconds |
| `-t`, `--timeout` | `10` | Per-request timeout in seconds |
| `--json` | off | Output raw JSON instead of formatted table |

### Stress Subcommand Options

| Flag | Default | Description |
|------|---------|-------------|
| `--from` | `10` | Starting concurrency |
| `--to` | `200` | Target concurrency |
| `--ramp` | `60` | Ramp duration in seconds |
| `--hold` | `30` | Hold duration at target concurrency |
| `-t`, `--timeout` | `10` | Per-request timeout in seconds |
| `--json` | off | Output raw JSON |

### Spike Subcommand Options

| Flag | Default | Description |
|------|---------|-------------|
| `--baseline` | `5` | Baseline concurrency |
| `--peak` | `500` | Peak concurrency |
| `--pre-spike` | `5` | Pre-spike duration in seconds |
| `--spike-duration` | `10` | Spike duration in seconds |
| `--post-spike` | `5` | Post-spike duration in seconds |
| `-t`, `--timeout` | `10` | Per-request timeout in seconds |
| `--json` | off | Output raw JSON |

### Global Options

| Flag | Default | Description |
|------|---------|-------------|
| `-v`, `-vv`, `-vvv` | warn | Increase verbosity (INFO, DEBUG, TRACE) |
| `-q`, `--quiet` | off | Suppress all output |
| `--log-file <path>` | none | Write logs to file |
| `-V`, `--version` | off | Show version and exit |

Logs stream to **stderr** by default, keeping stdout clean for JSON output piping:

```bash
strobengine load http://localhost:8080/api/health -vv --json > results.json
```

## Architecture

strobengine separates configuration, execution, and metrics into clean Rust modules, exposed to Python via PyO3:

- **`config`** -- `TestConfig` for static load, `LoadProfile` enum for dynamic profiles (Constant, Ramp, Spike) with target concurrency interpolation.
- **`worker`** -- Async worker loops accepting `CancellationToken` for graceful shutdown. Workers finish in-flight requests before exiting.
- **`metrics`** -- Lock-free atomic counters (`AtomicUsize`) track total requests and errors without contention. An aggregator task collects raw latencies, then `calculate_summary` computes average, p95, and p99 percentiles in Rust at bare-metal speed.
- **Orchestrator** -- Supervisor task ticks every 200ms, calculates target concurrency from the active profile curve, spawns/aborts workers dynamically.

The Python GIL is released entirely via `py.detach()` during test execution, allowing the full Tokio thread pool to run concurrently without throttling Python.

## Testing

Run the full test suite with:

```bash
# Rust unit tests
cargo test

# Python unit tests
uv run pytest -v
```

## How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes
4. Run formatting, linting, and tests:
   ```bash
   cargo fmt
   cargo clippy --all-targets -- -D warnings
   cargo test
   uv run pytest -v
   ```
5. Commit your changes following [Conventional Commits](https://www.conventionalcommits.org/)
6. Push to your branch and open a Pull Request

## License

This project is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full text.
