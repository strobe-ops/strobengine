# strobengine

A high-performance HTTP load testing engine with a Python API and a bare-metal Rust core.

## Dependencies

- **Python** >= 3.11
- **Rust** (stable, with `cargo`)
- **uv** (Python package manager)

### Rust crates

| Crate | Version | Purpose |
|-------|---------|---------|
| pyo3 | 0.29 | Python FFI bindings (stable ABI, abi3-py39) |
| reqwest | 0.13 | HTTP client with connection pooling |
| tokio | 1.53 | Multi-threaded async runtime |

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

# Create the engine
engine = StrobEngine(
    url="http://localhost:8080/api/health",
    concurrency=50,
    duration=30,
    timeout=5,
)

# Run the test — Python GIL is released during execution
summary = engine.run()

# Print a formatted summary
print_summary(summary, url=engine.config.url, duration_secs=30)
```

For async contexts (FastAPI, Typer, etc.):

```python
summary = await engine.run_async()
```

### Pretty output with Rich

Install the optional `rich` dependency for colorized table output:

```bash
uv sync --extra rich
```

Without `rich`, `print_summary` falls back to clean plain-text formatting.

## CLI Usage

```bash
# Basic load test
strobengine http://localhost:8080/api/health

# Custom parameters
strobengine http://localhost:8080/api/health -c 50 -d 30 -t 5

# JSON output for CI/CD
strobengine http://localhost:8080/api/health --json
```

## Architecture

strobengine separates configuration, execution, and metrics into clean Rust modules, exposed to Python via PyO3:

- **`config`** -- `TestConfig` pyclass holding test parameters (URL, concurrency, duration, timeout).
- **`worker`** -- Async worker loops spawned across a multi-threaded Tokio runtime. Each worker fires HTTP requests via reqwest, records microsecond-precise latencies, and sends metrics through a bounded channel.
- **`metrics`** -- Lock-free atomic counters (`AtomicUsize`) track total requests and errors without contention. An aggregator task collects raw latencies, then `calculate_summary` computes average, p95, and p99 percentiles in Rust at bare-metal speed.

The Python GIL is released entirely via `py.detach()` during test execution, allowing the full Tokio thread pool to run concurrently without throttling Python.

## How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes
4. Run formatting and linting:
   ```bash
   cargo fmt
   cargo clippy --all-targets -- -D warnings
   ```
5. Commit your changes following [Conventional Commits](https://www.conventionalcommits.org/)
6. Push to your branch and open a Pull Request

## License

This project is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full text.
