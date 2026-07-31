class LoadProfile:
    @staticmethod
    def constant(concurrency: int = 10, duration_secs: int = 10) -> LoadProfile: ...
    @staticmethod
    def ramp(
        start_concurrency: int,
        target_concurrency: int,
        ramp_secs: int,
        hold_secs: int,
    ) -> LoadProfile: ...
    @staticmethod
    def spike(
        baseline_concurrency: int,
        peak_concurrency: int,
        pre_spike_secs: int,
        spike_secs: int,
        post_spike_secs: int,
    ) -> LoadProfile: ...
    def total_duration(self) -> int: ...
    def max_concurrency(self) -> int: ...
    def target_concurrency(self, elapsed: float) -> int: ...

class TestConfig:
    url: str
    concurrency: int
    duration_secs: int
    timeout_secs: int
    chaos: bool
    chaos_rate: float
    no_progress: bool
    method: str
    body: str | None
    headers: dict[str, str] | None
    def __init__(
        self,
        url: str,
        concurrency: int = 10,
        duration_secs: int = 10,
        timeout_secs: int = 10,
        chaos: bool = False,
        chaos_rate: float = 0.1,
        no_progress: bool = False,
        method: str = "GET",
        body: str | None = None,
        headers: dict[str, str] | None = None,
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

def init_logging(level: str, log_file: str | None = None) -> None: ...
def run_load_test(config: TestConfig) -> TestSummary: ...
def run_load_profiles(
    url: str,
    timeout_secs: int,
    profile: LoadProfile,
    chaos: bool = False,
    chaos_rate: float = 0.1,
    no_progress: bool = False,
    method: str = "GET",
    body: str | None = None,
    headers: dict[str, str] | None = None,
) -> TestSummary: ...
