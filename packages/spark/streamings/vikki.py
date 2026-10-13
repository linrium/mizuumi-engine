"""Consume vn.vikki.t24.events into vikki.bronze.events."""

from common import JobConfig, run


if __name__ == "__main__":
    run(JobConfig(source="vikki", topic="vn.vikki.t24.events"))
