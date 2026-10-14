"""Export DuckDB query profiler metrics through OpenTelemetry."""

from __future__ import annotations

import json
import os
import sys
from collections.abc import Mapping
from typing import Protocol

import duckdb
from opentelemetry.exporter.otlp.proto.http.metric_exporter import OTLPMetricExporter
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.sdk.resources import Resource


class Histogram(Protocol):
    def record(
        self, amount: int | float, attributes: Mapping[str, str] | None = None
    ) -> None: ...


PROFILE_METRICS = {
    "latency": ("duckdb.query.duration", "s", "DuckDB query latency"),
    "cpu_time": ("duckdb.query.cpu.time", "s", "DuckDB query CPU time"),
    "blocked_thread_time": (
        "duckdb.query.blocked_thread.time",
        "s",
        "Time a DuckDB query waited for an execution thread",
    ),
    "rows_returned": (
        "duckdb.query.rows.returned",
        "{row}",
        "Rows returned by a DuckDB query",
    ),
    "cumulative_rows_scanned": (
        "duckdb.query.rows.scanned",
        "{row}",
        "Rows scanned by a DuckDB query and its operators",
    ),
    "cumulative_cardinality": (
        "duckdb.query.cardinality.cumulative",
        "{row}",
        "Cumulative cardinality across a DuckDB query's operators",
    ),
    "result_set_size": (
        "duckdb.query.result.size",
        "By",
        "DuckDB query result size",
    ),
    "system_peak_buffer_memory": (
        "duckdb.query.memory.peak_buffer",
        "By",
        "Peak DuckDB buffer-manager memory during a query",
    ),
    "system_peak_temp_dir_size": (
        "duckdb.query.temp_directory.peak_size",
        "By",
        "Peak DuckDB temporary-directory size during a query",
    ),
    "total_memory_allocated": (
        "duckdb.query.memory.allocated",
        "By",
        "Total DuckDB buffer-manager memory allocated during a query",
    ),
    "total_bytes_read": (
        "duckdb.query.io.read",
        "By",
        "Bytes read by DuckDB during a query",
    ),
    "total_bytes_written": (
        "duckdb.query.io.written",
        "By",
        "Bytes written by DuckDB during a query",
    ),
}


class DuckDBTelemetry:
    def __init__(self, provider: MeterProvider) -> None:
        self.provider = provider
        meter = provider.get_meter("mizuumi.duck_server", "0.1.0")
        self.histograms: dict[str, Histogram] = {
            profile_name: meter.create_histogram(
                name=otel_name,
                unit=unit,
                description=description,
            )
            for profile_name, (otel_name, unit, description) in PROFILE_METRICS.items()
        }

    def record_connection_profile(
        self, connection: duckdb.DuckDBPyConnection
    ) -> None:
        try:
            raw_profile = duckdb.get_profiling_information(
                format="json", connection=connection
            )
            profile = json.loads(raw_profile)
            if not isinstance(profile, dict):
                raise ValueError("DuckDB profiling information is not an object")
            self.record_profile(profile)
        except Exception as exc:
            # Telemetry must never turn a successful query into a failed request.
            print(f"DuckDB metrics collection error: {exc}", file=sys.stderr)

    def record_profile(self, profile: Mapping[str, object]) -> None:
        for profile_name, histogram in self.histograms.items():
            value = profile.get(profile_name)
            if isinstance(value, (int, float)) and not isinstance(value, bool):
                histogram.record(value)

    def shutdown(self) -> None:
        self.provider.shutdown()


class NoopDuckDBTelemetry:
    def record_connection_profile(
        self, _connection: duckdb.DuckDBPyConnection
    ) -> None:
        return

    def shutdown(self) -> None:
        return


def create_duckdb_telemetry() -> DuckDBTelemetry | NoopDuckDBTelemetry:
    if not (
        os.environ.get("OTEL_EXPORTER_OTLP_ENDPOINT")
        or os.environ.get("OTEL_EXPORTER_OTLP_METRICS_ENDPOINT")
    ):
        return NoopDuckDBTelemetry()

    try:
        resource = Resource.create(
            {
                "service.name": os.environ.get("OTEL_SERVICE_NAME", "duck-server"),
                "service.namespace": os.environ.get(
                    "OTEL_SERVICE_NAMESPACE", "duck-server"
                ),
                "service.version": duckdb.__version__,
            }
        )
        reader = PeriodicExportingMetricReader(OTLPMetricExporter())
        provider = MeterProvider(resource=resource, metric_readers=[reader])
        return DuckDBTelemetry(provider)
    except Exception as exc:
        print(f"OpenTelemetry metrics initialization error: {exc}", file=sys.stderr)
        return NoopDuckDBTelemetry()
