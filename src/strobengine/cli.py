from __future__ import annotations

import json
from typing import Annotated

import typer

from strobengine.engine import StrobEngine
from strobengine.reporter import print_summary


def _get_version() -> str:
    from importlib.metadata import PackageNotFoundError, version

    try:
        return version("strobengine")
    except PackageNotFoundError:
        return "0.0.0-dev"


def _version_callback(value: bool) -> None:
    if value:
        typer.echo(f"strobengine {_get_version()}")
        raise typer.Exit()


app = typer.Typer(
    name="strobengine",
    help="High-performance load testing engine powered by Rust.",
    no_args_is_help=True,
)

KNOWN_SUBCOMMANDS = {"load", "stress", "spike"}
HELP_FLAGS = {"-h", "--help"}
VERSION_FLAGS = {"-V", "--version"}


@app.callback()
def _global_options(
    version: Annotated[
        bool,
        typer.Option(
            "-V",
            "--version",
            help="Show version and exit",
            is_eager=True,
            callback=_version_callback,
        ),
    ] = False,
) -> None:
    pass


def _output_results(summary, url: str, duration_secs: int, json_output: bool) -> None:
    if json_output:
        print(
            json.dumps(
                {
                    "url": url,
                    "total_requests": summary.total_requests,
                    "total_errors": summary.total_errors,
                    "average_latency_ms": summary.average_latency_ms,
                    "p95_latency_ms": summary.p95_latency_ms,
                    "p99_latency_ms": summary.p99_latency_ms,
                },
                indent=2,
            )
        )
    else:
        print_summary(summary, url=url, duration_secs=duration_secs)


@app.command()
def load(
    url: Annotated[str, typer.Argument(help="Target HTTP/HTTPS URL")],
    concurrency: Annotated[
        int,
        typer.Option("-c", "--concurrency", min=1, help="Number of concurrent workers"),
    ] = 10,
    duration: Annotated[
        int,
        typer.Option("-d", "--duration", min=1, help="Test duration in seconds"),
    ] = 10,
    timeout: Annotated[
        int,
        typer.Option("-t", "--timeout", min=1, help="Request timeout in seconds"),
    ] = 10,
    json_output: Annotated[
        bool, typer.Option("--json", help="Output raw JSON results")
    ] = False,
) -> None:
    engine = StrobEngine.load_test(
        url=url, concurrency=concurrency, duration=duration, timeout=timeout
    )
    summary = engine.run()
    _output_results(summary, url, duration, json_output)


@app.command()
def stress(
    url: Annotated[str, typer.Argument(help="Target HTTP/HTTPS URL")],
    start: Annotated[
        int,
        typer.Option("--from", help="Starting concurrency", min=1),
    ] = 10,
    target: Annotated[
        int,
        typer.Option("--to", help="Target concurrency", min=1),
    ] = 200,
    ramp: Annotated[
        int,
        typer.Option("--ramp", help="Ramp duration in seconds", min=1),
    ] = 60,
    hold: Annotated[
        int,
        typer.Option("--hold", help="Hold duration in seconds", min=0),
    ] = 30,
    timeout: Annotated[
        int,
        typer.Option("-t", "--timeout", help="Request timeout in seconds", min=1),
    ] = 10,
    json_output: Annotated[
        bool, typer.Option("--json", help="Output raw JSON results")
    ] = False,
) -> None:
    engine = StrobEngine.stress_test(
        url=url,
        start_concurrency=start,
        max_concurrency=target,
        ramp_duration=ramp,
        hold_duration=hold,
        timeout=timeout,
    )
    summary = engine.run()
    _output_results(summary, url, ramp + hold, json_output)


@app.command()
def spike(
    url: Annotated[str, typer.Argument(help="Target HTTP/HTTPS URL")],
    baseline: Annotated[
        int,
        typer.Option("--baseline", help="Baseline concurrency", min=1),
    ] = 5,
    peak: Annotated[
        int,
        typer.Option("--peak", help="Peak concurrency", min=1),
    ] = 500,
    pre_spike: Annotated[
        int,
        typer.Option("--pre-spike", help="Pre-spike duration in seconds", min=0),
    ] = 5,
    spike_duration: Annotated[
        int,
        typer.Option("--spike-duration", help="Spike duration in seconds", min=1),
    ] = 10,
    post_spike: Annotated[
        int,
        typer.Option("--post-spike", help="Post-spike duration in seconds", min=0),
    ] = 5,
    timeout: Annotated[
        int,
        typer.Option("-t", "--timeout", help="Request timeout in seconds", min=1),
    ] = 10,
    json_output: Annotated[
        bool, typer.Option("--json", help="Output raw JSON results")
    ] = False,
) -> None:
    engine = StrobEngine.spike_test(
        url=url,
        baseline=baseline,
        peak_concurrency=peak,
        pre_spike_duration=pre_spike,
        spike_duration=spike_duration,
        post_spike_duration=post_spike,
        timeout=timeout,
    )
    summary = engine.run()
    _output_results(summary, url, pre_spike + spike_duration + post_spike, json_output)


def main(argv: list[str] | None = None) -> None:
    if argv is None:
        import sys

        argv = sys.argv[1:]

    if argv and set(argv) & (HELP_FLAGS | VERSION_FLAGS):
        app(args=argv)
        return

    if argv and argv[0] not in KNOWN_SUBCOMMANDS:
        argv = ["load", *argv]

    try:
        app(args=argv)
    except SystemExit as e:
        raise e


if __name__ == "__main__":
    main()
