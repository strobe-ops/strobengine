from __future__ import annotations

import os
import sys

from strobengine._strobengine import TestSummary

_HAS_RICH = False
try:
    from rich.console import Console
    from rich.table import Table

    _HAS_RICH = True
except ImportError:
    pass


def print_summary(
    summary: TestSummary,
    url: str,
    duration_secs: int | None = None,
) -> None:
    if _HAS_RICH:
        _print_rich(summary, url, duration_secs)
    else:
        _print_plain(summary, url, duration_secs)


def _format_number(n: int) -> str:
    return f"{n:,}"


def _error_rate(total: int, errors: int) -> str:
    if total == 0:
        return "0.00%"
    return f"{errors / total * 100:.2f}%"


def _print_rich(
    summary: TestSummary,
    url: str,
    duration_secs: int | None,
) -> None:
    console = Console()

    table = Table(title="Load Test Results", show_lines=True, padding=(0, 1))
    table.add_column("Metric", style="bold cyan", no_wrap=True)
    table.add_column("Value", justify="right")

    table.add_row("Target URL", url)
    table.add_row("Total Requests", _format_number(summary.total_requests))

    if summary.total_errors > 0:
        rate = _error_rate(summary.total_requests, summary.total_errors)
        table.add_row(
            "Errors",
            f"[bold red]{_format_number(summary.total_errors)} ({rate})[/]",
        )
    else:
        table.add_row(
            "Errors",
            f"[green]{_format_number(summary.total_errors)} (0.00%)[/]",
        )

    if duration_secs is not None and duration_secs > 0:
        rps = summary.total_requests / duration_secs
        table.add_row("Requests/sec", f"{rps:.1f}")

    table.add_row("Avg Latency", f"{summary.average_latency_ms:.2f} ms")
    table.add_row("P95 Latency", f"{summary.p95_latency_ms:.2f} ms")
    table.add_row("P99 Latency", f"{summary.p99_latency_ms:.2f} ms")

    console.print()
    console.print(table)
    console.print()


def _print_plain(
    summary: TestSummary,
    url: str,
    duration_secs: int | None,
) -> None:
    use_color = (
        not os.environ.get("NO_COLOR")
        and hasattr(sys.stdout, "isatty")
        and sys.stdout.isatty()
    )

    RED = "\033[91m" if use_color else ""
    GREEN = "\033[92m" if use_color else ""
    BOLD = "\033[1m" if use_color else ""
    RESET = "\033[0m" if use_color else ""

    width = 44
    sep = "=" * width

    lines = [
        f"{BOLD}{'Load Test Results':^{width}}{RESET}",
        sep,
        f"  Target URL:     {url}",
        f"  Total Requests: {_format_number(summary.total_requests)}",
    ]

    if summary.total_errors > 0:
        rate = _error_rate(summary.total_requests, summary.total_errors)
        lines.append(
            f"  Errors:         {RED}{_format_number(summary.total_errors)} ({rate}){RESET}"
        )
    else:
        lines.append(
            f"  Errors:         {GREEN}{_format_number(summary.total_errors)} (0.00%){RESET}"
        )

    if duration_secs is not None and duration_secs > 0:
        rps = summary.total_requests / duration_secs
        lines.append(f"  Requests/sec:   {rps:.1f}")

    lines.append(f"  Avg Latency:    {summary.average_latency_ms:.2f} ms")
    lines.append(f"  P95 Latency:    {summary.p95_latency_ms:.2f} ms")
    lines.append(f"  P99 Latency:    {summary.p99_latency_ms:.2f} ms")
    lines.append(sep)

    print("\n".join(lines))
