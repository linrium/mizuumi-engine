"""Consume vn.vietjet.events into vietjet.bronze.events."""

from common import JobConfig, run


if __name__ == "__main__":
    run(JobConfig(source="vietjet", topic="vn.vietjet.events"))
