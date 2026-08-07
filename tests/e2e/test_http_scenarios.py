import aiohttp
import pytest

from strobengine.engine import RequestOptions, StrobEngine


class TestHighConcurrency:
    async def test_multiple_vus_hit_status_200(self, mock_server: str):
        engine = StrobEngine.load_test(
            url=f"{mock_server}/status/200",
            concurrency=4,
            duration=3,
            options=RequestOptions(no_progress=True),
        )
        summary = await engine.run_async()

        assert summary.total_requests > 0
        assert summary.total_errors == 0
        assert summary.average_latency_ms >= 0


class TestErrorCounting:
    async def test_500_status_counted_as_error(self, mock_server: str):
        engine = StrobEngine.load_test(
            url=f"{mock_server}/status/500",
            concurrency=2,
            duration=3,
            options=RequestOptions(no_progress=True),
        )
        summary = await engine.run_async()

        assert summary.total_requests > 0
        assert summary.total_errors == summary.total_requests


class TestTimeoutHandling:
    async def test_timeout_counts_errors_when_delay_exceeds_timeout(
        self, mock_server: str
    ):
        engine = StrobEngine.load_test(
            url=f"{mock_server}/delay/5",
            concurrency=2,
            duration=3,
            options=RequestOptions(timeout=1, no_progress=True),
        )
        summary = await engine.run_async()

        assert summary.total_requests > 0
        assert summary.total_errors == summary.total_requests


class TestPayloadAndMethodForwarding:
    @pytest.mark.parametrize("method", ["GET", "POST", "PUT", "PATCH", "DELETE"])
    async def test_http_methods_and_payload_forwarding(
        self, mock_server: str, method: str
    ):
        headers = [("X-Custom-Test", "e2e-value")]
        body = '{"key": "e2e-payload"}' if method in ("POST", "PUT", "PATCH") else None

        engine = StrobEngine.load_test(
            url=f"{mock_server}/echo",
            concurrency=1,
            duration=2,
            options=RequestOptions(
                method=method,
                body=body,
                headers=headers,
                no_progress=True,
            ),
        )
        summary = await engine.run_async()
        assert summary.total_errors == 0

        async with (
            aiohttp.ClientSession() as session,
            session.get(f"{mock_server}/last-echo") as resp,
        ):
            echo = await resp.json()

        assert echo["method"] == method
        assert echo["headers"].get("x-custom-test") == "e2e-value"

        if body:
            assert echo["body"] == {"key": "e2e-payload"}
