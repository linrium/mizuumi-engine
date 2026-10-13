"""Consume vn.crm.events into crm.bronze.events."""

from common import JobConfig, run


if __name__ == "__main__":
    run(JobConfig(source="crm", topic="vn.crm.events"))
