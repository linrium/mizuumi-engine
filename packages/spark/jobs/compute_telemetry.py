"""Engine-neutral OpenTelemetry metrics for Mizuumi Spark batch operations."""

from __future__ import annotations

import os
import sys
from collections.abc import Mapping
from typing import Protocol


class ComputeTelemetry(Protocol):
    def record_operation(
        self,
        *,
        duration_seconds: float,
        workload_name: str,
        status: str,
        output_rows: int | None = None,
    ) -> None: ...

    def shutdown(self) -> None: ...


class NoopComputeTelemetry:
    def record_operation(
        self,
        *,
        duration_seconds: float,
        workload_name: str,
        status: str,
        output_rows: int | None = None,
    ) -> None:
        return

    def shutdown(self) -> None:
        return


class OtelComputeTelemetry:
    def __init__(self, provider: object, meter: object) -> None:
        self.provider = provider
        self.duration = meter.create_histogram(
            "mizuumi.compute.operation.duration",
            unit="s",
            description="Wall-clock duration of a completed compute operation",
        )
        self.output_rows = meter.create_histogram(
            "mizuumi.compute.operation.output.rows",
            unit="{row}",
            description="Rows produced by a completed compute operation",
        )
        self.operation_count = meter.create_counter(
            "mizuumi.compute.operation.count",
            unit="{operation}",
            description="Completed compute operations by status",
        )

    @staticmethod
    def attributes(workload_name: str, status: str) -> Mapping[str, str]:
        return {
            "mizuumi.compute.engine": "spark",
            "mizuumi.workload.kind": "batch",
            "mizuumi.workload.name": workload_name,
            "status": status,
        }

    def record_operation(
        self,
        *,
        duration_seconds: float,
        workload_name: str,
        status: str,
        output_rows: int | None = None,
    ) -> None:
        try:
            attributes = self.attributes(workload_name, status)
            self.duration.record(duration_seconds, attributes)
            if output_rows is not None:
                self.output_rows.record(output_rows, attributes)
            self.operation_count.add(1, attributes)
        except Exception as exc:
            print(f"Compute metrics recording error: {exc}", file=sys.stderr)

    def shutdown(self) -> None:
        try:
            self.provider.shutdown()
        except Exception as exc:
            print(f"Compute metrics shutdown error: {exc}", file=sys.stderr)


def create_compute_telemetry() -> ComputeTelemetry:
    """Create telemetry when the Spark driver has an OTLP endpoint configured."""
    if not (
        os.environ.get("OTEL_EXPORTER_OTLP_ENDPOINT")
        or os.environ.get("OTEL_EXPORTER_OTLP_METRICS_ENDPOINT")
    ):
        return NoopComputeTelemetry()

    try:
        from opentelemetry.exporter.otlp.proto.http.metric_exporter import (
            OTLPMetricExporter,
        )
        from opentelemetry.sdk.metrics import MeterProvider
        from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
        from opentelemetry.sdk.resources import Resource

        resource = Resource.create(
            {
                "service.name": os.environ.get("OTEL_SERVICE_NAME", "spark-driver"),
                "service.namespace": os.environ.get("OTEL_SERVICE_NAMESPACE", "spark"),
            }
        )
        reader = PeriodicExportingMetricReader(OTLPMetricExporter())
        provider = MeterProvider(resource=resource, metric_readers=[reader])
        return OtelComputeTelemetry(
            provider,
            provider.get_meter("mizuumi.spark.jobs", "0.1.0"),
        )
    except Exception as exc:
        # Metrics must never prevent a data job from running.
        print(f"Compute metrics initialization error: {exc}", file=sys.stderr)
        return NoopComputeTelemetry()
