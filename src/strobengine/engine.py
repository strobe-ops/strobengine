import asyncio

from strobengine._strobengine import (
    LoadProfile,
    TestConfig,
    TestSummary,
    run_load_profiles,
    run_load_test,
)


class StrobEngine:
    def __init__(
        self,
        url: str,
        concurrency: int = 10,
        duration: int = 10,
        timeout: int = 10,
        profile: LoadProfile | None = None,
    ) -> None:
        self._url = url
        self._timeout = timeout

        if profile is None:
            if concurrency <= 0:
                raise ValueError("Concurrency must be greater than 0")
            if duration <= 0:
                raise ValueError("Duration must be greater than 0")
            if timeout <= 0:
                raise ValueError("Timeout must be greater than 0")

            self.config = TestConfig(
                url=url,
                concurrency=concurrency,
                duration_secs=duration,
                timeout_secs=timeout,
            )
            self._profile = None
        else:
            if timeout <= 0:
                raise ValueError("Timeout must be greater than 0")

            self.config = None
            self._profile = profile

    @classmethod
    def load_test(
        cls,
        url: str,
        concurrency: int = 10,
        duration: int = 10,
        timeout: int = 10,
    ) -> "StrobEngine":
        return cls(url=url, concurrency=concurrency, duration=duration, timeout=timeout)

    @classmethod
    def stress_test(
        cls,
        url: str,
        start_concurrency: int = 10,
        max_concurrency: int = 200,
        ramp_duration: int = 60,
        hold_duration: int = 30,
        timeout: int = 10,
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
        return cls(url=url, timeout=timeout, profile=profile)

    @classmethod
    def spike_test(
        cls,
        url: str,
        baseline: int = 5,
        peak_concurrency: int = 500,
        pre_spike_duration: int = 5,
        spike_duration: int = 10,
        post_spike_duration: int = 5,
        timeout: int = 10,
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
        return cls(url=url, timeout=timeout, profile=profile)

    def run(self) -> TestSummary:
        if self._profile is not None:
            return run_load_profiles(self._url, self._timeout, self._profile)
        return run_load_test(self.config)

    async def run_async(self) -> TestSummary:
        return await asyncio.to_thread(self.run)
