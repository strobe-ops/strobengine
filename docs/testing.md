# Testing

## Quick Start

```bash
# Rust unit tests
cargo test

# Python unit tests
uv run pytest -v
```

## Test Structure

| Directory | Scope | Description |
|-----------|-------|-------------|
| `tests/test_engine.py` | Unit | `StrobEngine` config, factory methods, run dispatch (mocked Rust layer) |
| `tests/test_cli.py` | Integration | CLI subcommands via `typer.testing.CliRunner` against `local_server` |
| `tests/test_logging.py` | Unit | Log level resolution (`_resolve_log_level`) |
| `tests/test_reporter.py` | Unit | `format_number`, `error_rate`, `print_summary`, rich fallback |
| `tests/e2e/` | E2E | Full stack: Rust engine -> aiohttp mock server -> `TestSummary` assertions |

## Running Tests

```bash
# All tests
uv run pytest -v

# Only e2e tests
uv run pytest tests/e2e/ -v

# Specific test class
uv run pytest tests/e2e/test_http_scenarios.py::TestHighConcurrency -v

# Rust tests only
cargo test
```

## E2E Test Server

`tests/e2e/mock_server.py` provides an aiohttp-based HTTP server with deterministic
endpoints for testing the Rust engine against real HTTP responses.

### Endpoints

| Endpoint | Behavior |
|----------|----------|
| `/status/{code}` | Returns requested HTTP status code |
| `/delay/{seconds}` | Sleeps N seconds before responding |
| `/echo` | Echoes back request method, headers (lowercased), and body |
| `/last-echo` | Returns the last request captured by `/echo` |
| `/flaky` | Alternates between 200 OK and 500 on consecutive requests |

### Architecture

The server runs on a dynamic free port (`127.0.0.1:0`) in a background thread with
its own asyncio event loop. This avoids conflicts with the Rust engine's Tokio runtime
when it calls `asyncio.to_thread` internally.

The session-scoped `mock_server` fixture in `tests/e2e/conftest.py` manages the
server lifecycle -- it starts before the first test and shuts down after the last.

### Writing E2E Tests

```python
from strobengine.engine import RequestOptions, StrobEngine

async def test_example(self, mock_server: str):
    engine = StrobEngine.load_test(
        url=f"{mock_server}/status/200",
        concurrency=4,
        duration=3,
        options=RequestOptions(no_progress=True),
    )
    summary = await engine.run_async()

    assert summary.total_requests > 0
    assert summary.total_errors == 0
```

Use `concurrency=1` and `duration=2` for fast, deterministic tests. The
`no_progress=True` option suppresses the stderr progress bar during test runs.

## CI / Pre-Push Checks

Pre-commit hooks run automatically on `git push` (configured in
`.pre-commit-config.yaml`).

```bash
# Full check suite (same as make check)
pre-commit run --hook-stage pre-push --all-files

# Auto-fix formatting issues
make fix
```

### Hooks

| Hook | What it runs |
|------|-------------|
| `cargo-fmt` | `cargo fmt --check` |
| `cargo-clippy` | `cargo clippy --all-targets --all-features -- -D warnings` |
| `cargo-test` | `cargo test` |
| `ruff-check` | `uv run ruff check .` |
| `ruff-format` | `uv run ruff format --check .` |
| `pytest` | `uv run pytest -v` |
