import asyncio

from strobengine._strobengine import TestConfig, TestSummary, run_load_test


class StrobEngine:
    def __init__(
        self,
        url: str,
        concurrency: int = 10,
        duration: int = 10,
        timeout: int = 10,
    ) -> None:
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

    def run(self) -> TestSummary:
        return run_load_test(self.config)

    async def run_async(self) -> TestSummary:
        return await asyncio.to_thread(run_load_test, self.config)
