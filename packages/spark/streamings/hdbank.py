"""Consume vn.hdbank.t24.events into hdbank.bronze.events."""

from common import JobConfig, run


if __name__ == "__main__":
    run(JobConfig(source="hdbank", topic="vn.hdbank.t24.events"))
