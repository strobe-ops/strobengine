from typing import List, Tuple

def run_load_test(url: str, concurrency: int, duration_secs: int) -> Tuple[int, int, List[int]]: ...

class TestConfig:
    url: str
    concurrency: int
    duration_secs: int
    timeout_secs: int
    def __init__(
        self,
        url: str,
        concurrency: int = 10,
        duration_secs: int = 10,
        timeout_secs: int = 10,
    ) -> None: ...

class TestSummary:
    @property
    def total_requests(self) -> int: ...
    @property
    def total_errors(self) -> int: ...
    @property
    def average_latency_ms(self) -> float: ...
    @property
    def p95_latency_ms(self) -> float: ...
    @property
    def p99_latency_ms(self) -> float: ...
