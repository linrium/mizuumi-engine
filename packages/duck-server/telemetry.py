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


class Counter(Protocol):
    def add(
        self, amount: int | float, attributes: Mapping[str, str] | None = None
    ) -> None: ...


DURATION_BUCKETS_SECONDS = (
    0.001,
    0.0025,
    0.005,
    0.01,
    0.025,
    0.05,
    0.1,
    0.25,
    0.5,
    1,
    2.5,
    5,
    10,
    30,
    60,
    120,
    300,
)

ROW_COUNT_BUCKETS = (
    0,
    1,
    2,
    3,
    4,
    5,
    10,
    25,
    50,
    100,
    250,
    500,
    1_000,
    2_500,
    5_000,
    10_000,
    25_000,
    50_000,
    100_000,
    250_000,
    500_000,
    1_000_000,
    2_500_000,
    5_000_000,
    10_000_000,
)

BYTE_BUCKETS = (
    0,
    64,
    128,
    256,
    512,
    1_024,
    2_048,
    4_096,
    8_192,
    16_384,
    32_768,
    65_536,
    131_072,
    262_144,
    524_288,
    1_048_576,
    2_097_152,
    4_194_304,
    8_388_608,
    16_777_216,
    33_554_432,
    67_108_864,
    134_217_728,
    268_435_456,
    536_870_912,
    1_073_741_824,
    2_147_483_648,
    4_294_967_296,
    8_589_934_592,
    17_179_869_184,
)


PROFILE_METRICS = {
    "latency": (
        "duckdb.query.duration",
        "s",
        "DuckDB query latency",
        DURATION_BUCKETS_SECONDS,
    ),
    "cpu_time": (
        "duckdb.query.cpu.time",
        "s",
        "DuckDB query CPU time",
        DURATION_BUCKETS_SECONDS,
    ),
    "blocked_thread_time": (
        "duckdb.query.blocked_thread.time",
        "s",
        "Time a DuckDB query waited for an execution thread",
        DURATION_BUCKETS_SECONDS,
    ),
    "rows_returned": (
        "duckdb.query.rows.returned",
        "{row}",
        "Rows returned by a DuckDB query",
        ROW_COUNT_BUCKETS,
    ),
    "cumulative_rows_scanned": (
        "duckdb.query.rows.scanned",
        "{row}",
        "Rows scanned by a DuckDB query and its operators",
        ROW_COUNT_BUCKETS,
    ),
    "cumulative_cardinality": (
        "duckdb.query.cardinality.cumulative",
        "{row}",
        "Cumulative cardinality across a DuckDB query's operators",
        ROW_COUNT_BUCKETS,
    ),
    "result_set_size": (
        "duckdb.query.result.size",
        "By",
        "DuckDB query result size",
        BYTE_BUCKETS,
    ),
    "system_peak_buffer_memory": (
        "duckdb.query.memory.peak_buffer",
        "By",
        "Peak DuckDB buffer-manager memory during a query",
        BYTE_BUCKETS,
    ),
    "system_peak_temp_dir_size": (
        "duckdb.query.temp_directory.peak_size",
        "By",
        "Peak DuckDB temporary-directory size during a query",
        BYTE_BUCKETS,
    ),
    "total_memory_allocated": (
        "duckdb.query.memory.allocated",
        "By",
        "Total DuckDB buffer-manager memory allocated during a query",
        BYTE_BUCKETS,
    ),
    "total_bytes_read": (
        "duckdb.query.io.read",
        "By",
        "Bytes read by DuckDB during a query",
        BYTE_BUCKETS,
    ),
    "total_bytes_written": (
        "duckdb.query.io.written",
        "By",
        "Bytes written by DuckDB during a query",
        BYTE_BUCKETS,
    ),
}

# These engine-neutral instruments are the contract used by the Control Plane
# when DuckDB and Spark are shown side by side. Keep the native DuckDB metrics
# above as diagnostics; the canonical names describe only values with matching
# units and sufficiently similar semantics across engines.
CANONICAL_PROFILE_METRICS = {
    "latency": (
        "mizuumi.compute.operation.duration",
        "s",
        "Wall-clock duration of a completed compute operation",
        DURATION_BUCKETS_SECONDS,
    ),
    "cpu_time": (
        "mizuumi.compute.operation.cpu.time",
        "s",
        "CPU time consumed by a completed compute operation",
        DURATION_BUCKETS_SECONDS,
    ),
    "cumulative_rows_scanned": (
        "mizuumi.compute.operation.input.rows",
        "{row}",
        "Rows read by a completed compute operation",
        ROW_COUNT_BUCKETS,
    ),
    "rows_returned": (
        "mizuumi.compute.operation.output.rows",
        "{row}",
        "Rows produced by a completed compute operation",
        ROW_COUNT_BUCKETS,
    ),
    "total_bytes_read": (
        "mizuumi.compute.operation.input.bytes",
        "By",
        "Bytes read by a completed compute operation",
        BYTE_BUCKETS,
    ),
    "total_bytes_written": (
        "mizuumi.compute.operation.output.bytes",
        "By",
        "Bytes written by a completed compute operation",
        BYTE_BUCKETS,
    ),
    "system_peak_buffer_memory": (
        "mizuumi.compute.operation.memory.peak",
        "By",
        "Peak managed memory used by a completed compute operation",
        BYTE_BUCKETS,
    ),
    "system_peak_temp_dir_size": (
        "mizuumi.compute.operation.spill",
        "By",
        "Peak temporary spill storage used by a completed compute operation",
        BYTE_BUCKETS,
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
                explicit_bucket_boundaries_advisory=bucket_boundaries,
            )
            for profile_name, (
                otel_name,
                unit,
                description,
                bucket_boundaries,
            ) in PROFILE_METRICS.items()
        }
        self.canonical_histograms: dict[str, Histogram] = {
            profile_name: meter.create_histogram(
                name=otel_name,
                unit=unit,
                description=description,
                explicit_bucket_boundaries_advisory=bucket_boundaries,
            )
            for profile_name, (
                otel_name,
                unit,
                description,
                bucket_boundaries,
            ) in CANONICAL_PROFILE_METRICS.items()
        }
        self.operation_count: Counter = meter.create_counter(
            name="mizuumi.compute.operation.count",
            unit="{operation}",
            description="Completed compute operations by status",
        )

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
            self.record_profile(
                profile,
                {
                    "mizuumi.compute.engine": "duckdb",
                    "mizuumi.workload.kind": "query",
                    "mizuumi.workload.name": "ad_hoc_query",
                    "status": "success",
                },
            )
        except Exception as exc:
            # Telemetry must never turn a successful query into a failed request.
            print(f"DuckDB metrics collection error: {exc}", file=sys.stderr)

    def record_profile(
        self,
        profile: Mapping[str, object],
        attributes: Mapping[str, str] | None = None,
    ) -> None:
        for profile_name, histogram in self.histograms.items():
            value = profile.get(profile_name)
            if isinstance(value, (int, float)) and not isinstance(value, bool):
                histogram.record(value, attributes)
        for profile_name, histogram in self.canonical_histograms.items():
            value = profile.get(profile_name)
            if isinstance(value, (int, float)) and not isinstance(value, bool):
                histogram.record(value, attributes)
        self.operation_count.add(1, attributes)

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
