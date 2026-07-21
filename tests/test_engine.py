from unittest.mock import Mock, patch

import pytest

from strobengine.engine import StrobEngine


def _make_summary(**kwargs):
    defaults = {
        "total_requests": 100,
        "total_errors": 0,
        "average_latency_ms": 10.0,
        "p95_latency_ms": 20.0,
        "p99_latency_ms": 30.0,
    }
    defaults.update(kwargs)
    return Mock(**defaults)


class TestStrobEngineInit:
    def test_default_values(self):
        engine = StrobEngine(url="http://example.com")
        assert engine.config.url == "http://example.com"
        assert engine.config.concurrency == 10
        assert engine.config.duration_secs == 10
        assert engine.config.timeout_secs == 10

    def test_custom_values(self):
        engine = StrobEngine(
            url="http://test.io", concurrency=50, duration=30, timeout=5
        )
        assert engine.config.url == "http://test.io"
        assert engine.config.concurrency == 50
        assert engine.config.duration_secs == 30
        assert engine.config.timeout_secs == 5

    def test_invalid_concurrency(self):
        with pytest.raises(ValueError, match="Concurrency must be greater than 0"):
            StrobEngine(url="http://example.com", concurrency=0)

    def test_negative_concurrency(self):
        with pytest.raises(ValueError, match="Concurrency must be greater than 0"):
            StrobEngine(url="http://example.com", concurrency=-5)

    def test_invalid_duration(self):
        with pytest.raises(ValueError, match="Duration must be greater than 0"):
            StrobEngine(url="http://example.com", duration=0)

    def test_invalid_timeout(self):
        with pytest.raises(ValueError, match="Timeout must be greater than 0"):
            StrobEngine(url="http://example.com", timeout=0)


class TestStrobEngineRun:
    @patch("strobengine.engine.run_load_test")
    def test_run_returns_summary(self, mock_run):
        expected = _make_summary()
        mock_run.return_value = expected

        engine = StrobEngine(url="http://example.com")
        result = engine.run()

        mock_run.assert_called_once_with(engine.config)
        assert result is expected

    @patch("strobengine.engine.run_load_test")
    def test_run_with_custom_params(self, mock_run):
        mock_run.return_value = _make_summary()

        engine = StrobEngine(url="http://test.io", concurrency=25)
        engine.run()

        config = mock_run.call_args[0][0]
        assert config.url == "http://test.io"
        assert config.concurrency == 25


class TestStrobEngineRunAsync:
    @patch("strobengine.engine.run_load_test")
    async def test_run_async_returns_summary(self, mock_run):
        expected = _make_summary()
        mock_run.return_value = expected

        engine = StrobEngine(url="http://example.com")
        result = await engine.run_async()

        assert result is expected
