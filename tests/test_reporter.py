from unittest.mock import Mock, patch

from strobengine.reporter import _error_rate, _format_number, print_summary


def _make_summary(**kwargs):
    defaults = {
        "total_requests": 1000,
        "total_errors": 5,
        "average_latency_ms": 12.34,
        "p95_latency_ms": 45.67,
        "p99_latency_ms": 89.01,
    }
    defaults.update(kwargs)
    return Mock(**defaults)


class TestFormatNumber:
    def test_thousands(self):
        assert _format_number(15234) == "15,234"

    def test_zero(self):
        assert _format_number(0) == "0"

    def test_small_number(self):
        assert _format_number(42) == "42"


class TestErrorRate:
    def test_with_errors(self):
        assert _error_rate(1000, 5) == "0.50%"

    def test_no_errors(self):
        assert _error_rate(1000, 0) == "0.00%"

    def test_zero_total(self):
        assert _error_rate(0, 0) == "0.00%"

    def test_high_error_rate(self):
        assert _error_rate(100, 50) == "50.00%"


class TestPrintSummary:
    def test_with_duration(self, capsys):
        summary = _make_summary()
        print_summary(summary, url="http://example.com", duration_secs=10)
        output = capsys.readouterr().out
        assert "http://example.com" in output
        assert "1,000" in output or "1000" in output

    def test_without_duration(self, capsys):
        summary = _make_summary()
        print_summary(summary, url="http://example.com")
        output = capsys.readouterr().out
        assert "http://example.com" in output

    def test_errors_highlighted(self, capsys):
        summary = _make_summary(total_errors=50)
        print_summary(summary, url="http://example.com")
        output = capsys.readouterr().out
        assert "50" in output

    def test_no_errors(self, capsys):
        summary = _make_summary(total_errors=0)
        print_summary(summary, url="http://example.com")
        output = capsys.readouterr().out
        assert "http://example.com" in output


class TestRichFallback:
    @patch("strobengine.reporter._HAS_RICH", False)
    def test_plain_fallback_used(self, capsys):
        summary = _make_summary()
        print_summary(summary, url="http://example.com")
        output = capsys.readouterr().out
        assert "Load Test Results" in output
        assert "http://example.com" in output
