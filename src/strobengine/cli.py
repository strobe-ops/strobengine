from __future__ import annotations

import argparse
import json

from strobengine.engine import StrobEngine
from strobengine.reporter import print_summary

KNOWN_SUBCOMMANDS = {"load", "stress", "spike"}


def _normalize_argv(argv: list[str] | None) -> list[str] | None:
    if argv is None:
        return None
    if not argv or argv[0] in KNOWN_SUBCOMMANDS:
        return argv
    return ["load", *argv]


def _add_common_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("url", help="Target URL to test")
    parser.add_argument(
        "-t", "--timeout", type=int, default=10, help="Request timeout in seconds"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        dest="json_output",
        help="Output raw JSON instead of formatted table",
    )


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="strobengine",
        description="HTTP load testing engine with a bare-metal Rust core",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command")

    load_parser = subparsers.add_parser(
        "load",
        help="Constant load test",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    _add_common_args(load_parser)
    load_parser.add_argument(
        "-c", "--concurrency", type=int, default=10, help="Concurrent workers"
    )
    load_parser.add_argument(
        "-d", "--duration", type=int, default=10, help="Duration in seconds"
    )

    stress_parser = subparsers.add_parser(
        "stress",
        help="Ramp/stress test",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    _add_common_args(stress_parser)
    stress_parser.add_argument(
        "--from",
        dest="start_concurrency",
        type=int,
        default=10,
        help="Starting concurrency",
    )
    stress_parser.add_argument(
        "--to",
        dest="max_concurrency",
        type=int,
        default=200,
        help="Target concurrency",
    )
    stress_parser.add_argument(
        "--ramp",
        dest="ramp_duration",
        type=int,
        default=60,
        help="Ramp duration in seconds",
    )
    stress_parser.add_argument(
        "--hold",
        dest="hold_duration",
        type=int,
        default=30,
        help="Hold duration in seconds",
    )

    spike_parser = subparsers.add_parser(
        "spike",
        help="Spike test",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    _add_common_args(spike_parser)
    spike_parser.add_argument(
        "--baseline", type=int, default=5, help="Baseline concurrency"
    )
    spike_parser.add_argument(
        "--peak",
        dest="peak_concurrency",
        type=int,
        default=500,
        help="Peak concurrency",
    )
    spike_parser.add_argument(
        "--pre-spike",
        dest="pre_spike_duration",
        type=int,
        default=5,
        help="Pre-spike duration in seconds",
    )
    spike_parser.add_argument(
        "--spike-duration",
        dest="spike_duration",
        type=int,
        default=10,
        help="Spike duration in seconds",
    )
    spike_parser.add_argument(
        "--post-spike",
        dest="post_spike_duration",
        type=int,
        default=5,
        help="Post-spike duration in seconds",
    )

    return parser


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


def main(argv: list[str] | None = None) -> None:
    argv = _normalize_argv(argv)
    parser = _build_parser()
    args = parser.parse_args(argv)

    if args.command is None:
        parser.print_help()
        return

    if args.timeout <= 0:
        parser.error("--timeout must be greater than 0")

    if args.command == "load":
        if args.concurrency <= 0:
            parser.error("--concurrency must be greater than 0")
        if args.duration <= 0:
            parser.error("--duration must be greater than 0")

        engine = StrobEngine.load_test(
            url=args.url,
            concurrency=args.concurrency,
            duration=args.duration,
            timeout=args.timeout,
        )
        summary = engine.run()
        _output_results(summary, args.url, args.duration, args.json_output)

    elif args.command == "stress":
        if args.start_concurrency <= 0:
            parser.error("--from must be greater than 0")
        if args.max_concurrency <= 0:
            parser.error("--to must be greater than 0")
        if args.start_concurrency > args.max_concurrency:
            parser.error("--from must be <= --to")

        engine = StrobEngine.stress_test(
            url=args.url,
            start_concurrency=args.start_concurrency,
            max_concurrency=args.max_concurrency,
            ramp_duration=args.ramp_duration,
            hold_duration=args.hold_duration,
            timeout=args.timeout,
        )
        summary = engine.run()
        duration = args.ramp_duration + args.hold_duration
        _output_results(summary, args.url, duration, args.json_output)

    elif args.command == "spike":
        if args.baseline <= 0:
            parser.error("--baseline must be greater than 0")
        if args.peak_concurrency <= 0:
            parser.error("--peak must be greater than 0")

        engine = StrobEngine.spike_test(
            url=args.url,
            baseline=args.baseline,
            peak_concurrency=args.peak_concurrency,
            pre_spike_duration=args.pre_spike_duration,
            spike_duration=args.spike_duration,
            post_spike_duration=args.post_spike_duration,
            timeout=args.timeout,
        )
        summary = engine.run()
        duration = (
            args.pre_spike_duration + args.spike_duration + args.post_spike_duration
        )
        _output_results(summary, args.url, duration, args.json_output)


if __name__ == "__main__":
    main()
