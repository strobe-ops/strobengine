import json
from unittest.mock import Mock, patch

import pytest

from strobengine.cli import main


def _make_summary(**kwargs):
    defaults = {
        "total_requests": 500,
        "total_errors": 2,
        "average_latency_ms": 15.0,
        "p95_latency_ms": 30.0,
        "p99_latency_ms": 45.0,
    }
    defaults.update(kwargs)
    return Mock(**defaults)


class TestCLIParsing:
    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_basic_args(self, MockEngine, mock_print):
        MockEngine.return_value.run.return_value = _make_summary()
        main(["http://example.com"])
        MockEngine.assert_called_once_with(
            url="http://example.com", concurrency=10, duration=10, timeout=10
        )

    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_custom_flags(self, MockEngine, mock_print):
        MockEngine.return_value.run.return_value = _make_summary()
        main(["http://test.io", "-c", "50", "-d", "30", "-t", "5"])
        MockEngine.assert_called_once_with(
            url="http://test.io", concurrency=50, duration=30, timeout=5
        )


class TestCLIValidation:
    def test_missing_url(self):
        with pytest.raises(SystemExit) as exc_info:
            main([])
        assert exc_info.value.code == 2

    def test_invalid_concurrency(self):
        with pytest.raises(SystemExit) as exc_info:
            main(["http://example.com", "-c", "0"])
        assert exc_info.value.code == 2

    def test_negative_duration(self):
        with pytest.raises(SystemExit) as exc_info:
            main(["http://example.com", "-d", "-1"])
        assert exc_info.value.code == 2

    def test_invalid_timeout(self):
        with pytest.raises(SystemExit) as exc_info:
            main(["http://example.com", "-t", "0"])
        assert exc_info.value.code == 2


class TestCLIJsonOutput:
    @patch("strobengine.cli.StrobEngine")
    def test_json_flag(self, MockEngine, capsys):
        MockEngine.return_value.run.return_value = _make_summary()
        main(["http://example.com", "--json"])
        output = capsys.readouterr().out
        data = json.loads(output)
        assert data["url"] == "http://example.com"
        assert data["total_requests"] == 500
        assert data["total_errors"] == 2
        assert data["average_latency_ms"] == 15.0
        assert data["p95_latency_ms"] == 30.0
        assert data["p99_latency_ms"] == 45.0

    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_json_not_set_uses_print_summary(self, MockEngine, mock_print):
        MockEngine.return_value.run.return_value = _make_summary()
        main(["http://example.com"])
        mock_print.assert_called_once()
