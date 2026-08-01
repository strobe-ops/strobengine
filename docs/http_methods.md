# HTTP Method Support

strobengine supports the full suite of standard HTTP methods for load testing.
Methods are validated once at configuration time and routed through a
zero-allocation hot path during test execution.

## Supported Methods

| Method | Description |
|--------|-------------|
| `GET` | Retrieve a resource (default) |
| `POST` | Create a resource or submit data |
| `PUT` | Replace a resource entirely |
| `DELETE` | Remove a resource |
| `PATCH` | Partially update a resource |
| `HEAD` | Retrieve headers only (no body) |
| `OPTIONS` | Describe communication options (CORS preflight) |

Methods are **case-insensitive** — `post`, `Post`, and `POST` are all equivalent.

## Run server for test

```bash
podman run -d --rm -p 8080:8080 --name httpbin docker.io/mccutchen/go-httpbin

```

[go-httpbin](https://github.com/mccutchen/go-httpbin)

## Python API

### Constant Load Test

```python
engine = StrobEngine(
    url="http://localhost:8080/post",
    method="POST",
    body='{"name": "test", "value": 42}',
    headers=[("Authorization", "Bearer token123")],
)
summary = engine.run()
```

### Stress Test

```python
engine = StrobEngine.stress_test(
    url="http://localhost:8080/post",
    method="POST",
    body='{"load": "test"}',
    start_concurrency=10,
    max_concurrency=200,
    ramp_duration=60,
    hold_duration=30,
)
summary = engine.run()
```

### Spike Test

```python
engine = StrobEngine.spike_test(
    url="http://localhost:8080/delete",
    method="DELETE",
    baseline=5,
    peak_concurrency=500,
)
summary = engine.run()
```

## CLI Usage

```bash
# GET
uv run strobengine load http://localhost:8080/get

# POST with JSON body
uv run strobengine load http://localhost:8080/post \
  --method POST \
  --body '{"test":"payload"}' \
  --header "Content-Type: application/json"

# PUT with custom headers
uv run strobengine load http://localhost:8080/put \
  --method PUT \
  --body '{"update": true}' \
  --header "Authorization: Bearer token123"

# PATCH
uv run strobengine load http://localhost:8080/patch \
  --method PATCH \
  --body '{"patch": 1}'

# DELETE
uv run strobengine load http://localhost:8080/delete --method DELETE

# HEAD (check endpoint without downloading body)
uv run strobengine load http://localhost:8080/headers --method HEAD

# OPTIONS (CORS preflight)
uv run strobengine load http://localhost:8080/anything --method OPTIONS

# Stress test with POST
uv run strobengine stress http://localhost:8080/post \
  --method POST --body '{"load": "test"}' --from 10 --to 200

# Multiple headers (repeatable)
uv run strobengine load http://localhost:8080/post \
  --method POST \
  --body '{"key": "val"}' \
  --header "Authorization: Bearer tok" \
  --header "X-Custom: value"
```

## Request Body

The `--body` flag accepts a raw string that is sent as the request body. The body is serialized **once** before the test begins and stored as a zero-copy `bytes::Bytes` buffer. Each request clones the buffer via atomic reference-count increment — no heap allocation per request.

```bash
# JSON body
uv run strobengine load http://localhost:8080/post \
  --method POST --body '{"test":"payload"}'

# Form data
uv run strobengine load http://localhost:8080/post \
  --method POST --body 'field1=value1&field2=value2'
```

**Auto Content-Type:** If you provide `--body` without an explicit `Content-Type` header, strobengine automatically inserts `Content-Type: application/json`.

## Custom Headers

The `--header` flag accepts headers in `Key: Value` format and can be repeated:

```bash
uv run strobengine load http://localhost:8080/post \
  --method POST \
  --body '{"key": "val"}' \
  --header "Authorization: Bearer mytoken" \
  --header "X-Request-ID: 12345" \
  --header "Accept: application/json"
```

**Parsing rules:**
- Headers are split on the **first colon** only, so values containing colons work correctly (e.g., `Host: localhost:8080`)
- Whitespace around the key and value is stripped automatically
- Invalid header names or values produce a clear error message before the test starts

## Performance Characteristics

- **Method routing:** Uses `http::Method` enum instances — zero-cost enum copy per request
- **Static payloads:** Body is stored as `bytes::Bytes`, cloned via atomic reference-count increment (zero heap allocation per request)
- **Headers:** Pre-allocated as a `HeaderMap` and set on the `reqwest::Client` via `default_headers()` — zero per-request header allocation
- **Validation:** Method, body, and headers are parsed and validated once before the test starts. Invalid input produces a clear error at configuration time, not during the hot loop.

## Chaos Interaction

When `--chaos` is enabled alongside a custom HTTP method:

| Chaos Fault | Behavior |
|-------------|----------|
| `LatencySpike` | Uses your configured method |
| `CorruptedPayload` | **Overrides to POST** with a malformed body (tests server error handling) |
| `MetadataCorruption` | Uses your configured method + adds invalid headers |
| `ConnectionDrop` | Uses your configured method with 1ns timeout |
