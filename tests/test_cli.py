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


class TestCLIBackwardCompat:
    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_no_subcommand_defaults_to_load(self, MockEngine, mock_print):
        MockEngine.load_test.return_value.run.return_value = _make_summary()
        main(["http://example.com"])
        MockEngine.load_test.assert_called_once_with(
            url="http://example.com", concurrency=10, duration=10, timeout=10
        )

    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_flags_before_url(self, MockEngine, mock_print):
        MockEngine.load_test.return_value.run.return_value = _make_summary()
        main(["-c", "50", "http://example.com"])
        MockEngine.load_test.assert_called_once_with(
            url="http://example.com", concurrency=50, duration=10, timeout=10
        )


class TestCLILoadSubcommand:
    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_load_default(self, MockEngine, mock_print):
        MockEngine.load_test.return_value.run.return_value = _make_summary()
        main(["load", "http://example.com"])
        MockEngine.load_test.assert_called_once_with(
            url="http://example.com", concurrency=10, duration=10, timeout=10
        )

    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_load_custom_flags(self, MockEngine, mock_print):
        MockEngine.load_test.return_value.run.return_value = _make_summary()
        main(["load", "http://test.io", "-c", "50", "-d", "30", "-t", "5"])
        MockEngine.load_test.assert_called_once_with(
            url="http://test.io", concurrency=50, duration=30, timeout=5
        )


class TestCLIStressSubcommand:
    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_stress_default(self, MockEngine, mock_print):
        MockEngine.stress_test.return_value.run.return_value = _make_summary()
        main(["stress", "http://example.com"])
        MockEngine.stress_test.assert_called_once_with(
            url="http://example.com",
            start_concurrency=10,
            max_concurrency=200,
            ramp_duration=60,
            hold_duration=30,
            timeout=10,
        )

    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_stress_custom_flags(self, MockEngine, mock_print):
        MockEngine.stress_test.return_value.run.return_value = _make_summary()
        main(
            [
                "stress",
                "http://test.io",
                "--from",
                "5",
                "--to",
                "500",
                "--ramp",
                "30",
                "--hold",
                "15",
                "-t",
                "5",
            ]
        )
        MockEngine.stress_test.assert_called_once_with(
            url="http://test.io",
            start_concurrency=5,
            max_concurrency=500,
            ramp_duration=30,
            hold_duration=15,
            timeout=5,
        )


class TestCLISpikeSubcommand:
    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_spike_default(self, MockEngine, mock_print):
        MockEngine.spike_test.return_value.run.return_value = _make_summary()
        main(["spike", "http://example.com"])
        MockEngine.spike_test.assert_called_once_with(
            url="http://example.com",
            baseline=5,
            peak_concurrency=500,
            pre_spike_duration=5,
            spike_duration=10,
            post_spike_duration=5,
            timeout=10,
        )

    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_spike_custom_flags(self, MockEngine, mock_print):
        MockEngine.spike_test.return_value.run.return_value = _make_summary()
        main(
            [
                "spike",
                "http://test.io",
                "--baseline",
                "10",
                "--peak",
                "1000",
                "--pre-spike",
                "3",
                "--spike-duration",
                "7",
                "--post-spike",
                "3",
                "-t",
                "5",
            ]
        )
        MockEngine.spike_test.assert_called_once_with(
            url="http://test.io",
            baseline=10,
            peak_concurrency=1000,
            pre_spike_duration=3,
            spike_duration=7,
            post_spike_duration=3,
            timeout=5,
        )


class TestCLIValidation:
    def test_missing_url(self):
        with pytest.raises(SystemExit) as exc_info:
            main(["load"])
        assert exc_info.value.code == 2

    def test_load_invalid_concurrency(self):
        with pytest.raises(SystemExit) as exc_info:
            main(["load", "http://example.com", "-c", "0"])
        assert exc_info.value.code == 2

    def test_stress_invalid_from(self):
        with pytest.raises(SystemExit) as exc_info:
            main(["stress", "http://example.com", "--from", "0"])
        assert exc_info.value.code == 2

    def test_spike_invalid_baseline(self):
        with pytest.raises(SystemExit) as exc_info:
            main(["spike", "http://example.com", "--baseline", "0"])
        assert exc_info.value.code == 2


class TestCLIJsonOutput:
    @patch("strobengine.cli.StrobEngine")
    def test_json_load(self, MockEngine, capsys):
        MockEngine.load_test.return_value.run.return_value = _make_summary()
        main(["load", "http://example.com", "--json"])
        output = capsys.readouterr().out
        data = json.loads(output)
        assert data["url"] == "http://example.com"
        assert data["total_requests"] == 500

    @patch("strobengine.cli.StrobEngine")
    def test_json_stress(self, MockEngine, capsys):
        MockEngine.stress_test.return_value.run.return_value = _make_summary()
        main(["stress", "http://example.com", "--json"])
        output = capsys.readouterr().out
        data = json.loads(output)
        assert data["url"] == "http://example.com"

    @patch("strobengine.cli.StrobEngine")
    def test_json_spike(self, MockEngine, capsys):
        MockEngine.spike_test.return_value.run.return_value = _make_summary()
        main(["spike", "http://example.com", "--json"])
        output = capsys.readouterr().out
        data = json.loads(output)
        assert data["url"] == "http://example.com"

    @patch("strobengine.cli.print_summary")
    @patch("strobengine.cli.StrobEngine")
    def test_json_not_set_uses_print_summary(self, MockEngine, mock_print):
        MockEngine.load_test.return_value.run.return_value = _make_summary()
        main(["load", "http://example.com"])
        mock_print.assert_called_once()
