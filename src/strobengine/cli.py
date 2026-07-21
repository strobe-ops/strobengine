from __future__ import annotations

import argparse
import json

from strobengine.engine import StrobEngine
from strobengine.reporter import print_summary


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        prog="strobengine",
        description="HTTP load testing engine with a bare-metal Rust core",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument("url", help="Target URL to test")
    parser.add_argument(
        "-c", "--concurrency", type=int, default=10, help="Concurrent workers"
    )
    parser.add_argument(
        "-d", "--duration", type=int, default=10, help="Duration in seconds"
    )
    parser.add_argument(
        "-t", "--timeout", type=int, default=10, help="Request timeout in seconds"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        dest="json_output",
        help="Output raw JSON instead of formatted table",
    )

    args = parser.parse_args(argv)

    if args.concurrency <= 0:
        parser.error("--concurrency must be greater than 0")
    if args.duration <= 0:
        parser.error("--duration must be greater than 0")
    if args.timeout <= 0:
        parser.error("--timeout must be greater than 0")

    engine = StrobEngine(
        url=args.url,
        concurrency=args.concurrency,
        duration=args.duration,
        timeout=args.timeout,
    )
    summary = engine.run()

    if args.json_output:
        print(
            json.dumps(
                {
                    "url": args.url,
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
        print_summary(summary, url=args.url, duration_secs=args.duration)


if __name__ == "__main__":
    main()
