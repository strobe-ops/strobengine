from strobengine.cli import _resolve_log_level


class TestLogLevelResolution:
    def test_quiet_returns_off(self) -> None:
        assert _resolve_log_level(0, True) == "off"

    def test_no_verbose_returns_warn(self) -> None:
        assert _resolve_log_level(0, False) == "warn"

    def test_single_v_returns_info(self) -> None:
        assert _resolve_log_level(1, False) == "info"

    def test_double_v_returns_debug(self) -> None:
        assert _resolve_log_level(2, False) == "debug"

    def test_triple_v_returns_trace(self) -> None:
        assert _resolve_log_level(3, False) == "trace"

    def test_quad_v_returns_trace(self) -> None:
        assert _resolve_log_level(4, False) == "trace"
