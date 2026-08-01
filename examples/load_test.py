"""Basic constant load test -- fires N concurrent GET requests for D seconds."""

from strobengine import StrobEngine
from strobengine.reporter import print_summary

# 50 concurrent workers hitting the endpoint for 30 seconds
engine = StrobEngine(
    url="http://localhost:8080/get",
    concurrency=50,
    duration=30,
)
summary = engine.run()

print_summary(summary, url=engine._url, duration_secs=30)
