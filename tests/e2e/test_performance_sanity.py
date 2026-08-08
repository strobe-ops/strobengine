import pytest

from strobengine.engine import RequestOptions, StrobEngine


class TestPerformanceSanity:
    """
    Sanity tests to ensure engine stability and metric correctness under load.
    """

    @pytest.mark.asyncio
    async def test_quick_run_latency_distribution(self, mock_server: str):
        """
        Performs a 2-second sanity run and asserts that the collected
        percentile metrics are logically consistent.
        """
        # Initialize the engine with a quick 2-second duration.
        # We use a stable 200 OK endpoint from the mock server.
        engine = StrobEngine.load_test(
            url=f"{mock_server}/status/200",
            concurrency=5,
            duration=2,
            options=RequestOptions(no_progress=True),
        )

        summary = await engine.run_async()

        assert summary.total_requests > 0, (
            "Engine should have completed at least one request."
        )
        assert summary.total_errors == 0, (
            "Unexpected errors occurred during a sanity run against /status/200."
        )

        assert summary.average_latency_ms >= 0, "Average latency must be non-negative."
        assert summary.average_latency_ms <= summary.p95_latency_ms, (
            "Metric error: Average > P95."
        )
        assert summary.p95_latency_ms <= summary.p99_latency_ms, (
            "Metric error: P95 > P99."
        )

    @pytest.mark.asyncio
    async def test_summary_structure_completeness(self, mock_server: str):
        """
        Ensures the returned TestSummary contains all required metric fields.
        """
        engine = StrobEngine.load_test(
            url=f"{mock_server}/status/200",
            concurrency=2,
            duration=1,
            options=RequestOptions(no_progress=True),
        )
        summary = await engine.run_async()

        # Check for existence of core metrics.
        assert hasattr(summary, "total_requests")
        assert hasattr(summary, "total_errors")
        assert hasattr(summary, "average_latency_ms")
        assert hasattr(summary, "p95_latency_ms")
        assert hasattr(summary, "p99_latency_ms")
