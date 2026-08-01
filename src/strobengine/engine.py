import asyncio
from dataclasses import dataclass, field

from strobengine._strobengine import (
    LoadProfile,
    TestConfig,
    TestSummary,
    run_load_profiles,
    run_load_test,
)


@dataclass
class RequestOptions:
    """Encapsulates common HTTP and execution parameters with validation."""

    timeout: int = 10
    chaos: bool = False
    no_progress: bool = False
    method: str = "GET"
    body: str | None = None
    headers: list[tuple[str, str]] = field(default_factory=list)

    def __post_init__(self) -> None:
        if self.timeout <= 0:
            raise ValueError("timeout must be greater than 0")


class StrobEngine:
    def __init__(
        self,
        url: str,
        concurrency: int = 10,
        duration: int = 10,
        options: RequestOptions | None = None,
        profile: LoadProfile | None = None,
    ) -> None:
        self._url = url
        self._options = options
        self._profile = profile

        if profile is None:
            if concurrency <= 0:
                raise ValueError("Concurrency must be greater than 0")
            if duration <= 0:
                raise ValueError("Duration must be greater than 0")

            self.config = TestConfig(
                url=url,
                concurrency=concurrency,
                duration_secs=duration,
                timeout_secs=self._options.timeout,
                chaos=self._options.chaos,
                no_progress=self._options.no_progress,
                method=self._options.method,
                body=self._options.body,
                headers=self._options.headers,
            )
            self._profile = None
        else:
            self.config = None

    @classmethod
    def load_test(
        cls,
        url: str,
        concurrency: int = 10,
        duration: int = 10,
        options: RequestOptions | None = None,
    ) -> "StrobEngine":
        return cls(
            url=url,
            concurrency=concurrency,
            duration=duration,
            options=options,
        )

    @classmethod
    def stress_test(
        cls,
        url: str,
        start_concurrency: int = 10,
        max_concurrency: int = 200,
        ramp_duration: int = 60,
        hold_duration: int = 30,
        options: RequestOptions | None = None,
    ) -> "StrobEngine":
        if start_concurrency <= 0:
            raise ValueError("start_concurrency must be greater than 0")
        if max_concurrency <= 0:
            raise ValueError("max_concurrency must be greater than 0")
        if start_concurrency > max_concurrency:
            raise ValueError("start_concurrency must be <= max_concurrency")
        if ramp_duration < 0:
            raise ValueError("ramp_duration must be >= 0")
        if hold_duration < 0:
            raise ValueError("hold_duration must be >= 0")

        profile = LoadProfile.ramp(
            start_concurrency=start_concurrency,
            target_concurrency=max_concurrency,
            ramp_secs=ramp_duration,
            hold_secs=hold_duration,
        )
        return cls(
            url=url,
            profile=profile,
            options=options,
        )

    @classmethod
    def spike_test(
        cls,
        url: str,
        baseline: int = 5,
        peak_concurrency: int = 500,
        pre_spike_duration: int = 5,
        spike_duration: int = 10,
        post_spike_duration: int = 5,
        options: RequestOptions | None = None,
    ) -> "StrobEngine":
        if baseline <= 0:
            raise ValueError("baseline must be greater than 0")
        if peak_concurrency <= 0:
            raise ValueError("peak_concurrency must be greater than 0")
        if pre_spike_duration < 0:
            raise ValueError("pre_spike_duration must be >= 0")
        if spike_duration < 0:
            raise ValueError("spike_duration must be >= 0")
        if post_spike_duration < 0:
            raise ValueError("post_spike_duration must be >= 0")

        profile = LoadProfile.spike(
            baseline_concurrency=baseline,
            peak_concurrency=peak_concurrency,
            pre_spike_secs=pre_spike_duration,
            spike_secs=spike_duration,
            post_spike_secs=post_spike_duration,
        )
        return cls(
            url=url,
            profile=profile,
            options=options,
        )

    def run(self) -> TestSummary:
        opts = self._options
        if self._profile is not None:
            return run_load_profiles(
                self._url,
                self._timeout,
                self._profile,
                opts._chaos,
                no_progress=opts._no_progress,
                method=opts._method,
                body=opts._body,
                headers=opts._headers,
            )
        return run_load_test(self.config)

    async def run_async(self) -> TestSummary:
        return await asyncio.to_thread(self.run)
