from __future__ import annotations

import os
import unittest
from unittest.mock import MagicMock, patch

import duckdb

from telemetry import (
    BYTE_BUCKETS,
    DURATION_BUCKETS_SECONDS,
    ROW_COUNT_BUCKETS,
    DuckDBTelemetry,
    NoopDuckDBTelemetry,
    create_duckdb_telemetry,
)


class RecordingHistogram:
    def __init__(self) -> None:
        self.values: list[int | float] = []

    def record(self, amount: int | float, attributes: object = None) -> None:
        self.values.append(amount)


class ProfileMetricsTest(unittest.TestCase):
    def test_configures_unit_specific_histogram_boundaries(self) -> None:
        provider = MagicMock()
        meter = provider.get_meter.return_value

        DuckDBTelemetry(provider)

        instruments = {
            call.kwargs["name"]: call.kwargs
            for call in meter.create_histogram.call_args_list
        }
        self.assertEqual(
            instruments["duckdb.query.duration"][
                "explicit_bucket_boundaries_advisory"
            ],
            DURATION_BUCKETS_SECONDS,
        )
        self.assertEqual(
            instruments["duckdb.query.rows.returned"][
                "explicit_bucket_boundaries_advisory"
            ],
            ROW_COUNT_BUCKETS,
        )
        self.assertEqual(
            instruments["duckdb.query.io.read"][
                "explicit_bucket_boundaries_advisory"
            ],
            BYTE_BUCKETS,
        )

    def test_records_numeric_profile_values_and_ignores_metadata(self) -> None:
        telemetry = object.__new__(DuckDBTelemetry)
        latency = RecordingHistogram()
        rows = RecordingHistogram()
        telemetry.histograms = {"latency": latency, "rows_returned": rows}

        telemetry.record_profile(
            {
                "latency": 0.25,
                "rows_returned": 42,
                "query_name": "SELECT secret FROM customer",
                "children": [],
            }
        )

        self.assertEqual(latency.values, [0.25])
        self.assertEqual(rows.values, [42])

    def test_skips_missing_boolean_and_non_numeric_values(self) -> None:
        telemetry = object.__new__(DuckDBTelemetry)
        latency = RecordingHistogram()
        rows = RecordingHistogram()
        telemetry.histograms = {"latency": latency, "rows_returned": rows}

        telemetry.record_profile({"latency": True, "rows_returned": "12"})

        self.assertEqual(latency.values, [])
        self.assertEqual(rows.values, [])

    def test_reads_a_completed_duckdb_profile(self) -> None:
        telemetry = object.__new__(DuckDBTelemetry)
        latency = RecordingHistogram()
        rows = RecordingHistogram()
        telemetry.histograms = {"latency": latency, "rows_returned": rows}
        connection = duckdb.connect()
        try:
            connection.execute("SET enable_profiling = 'no_output'")
            connection.execute("SELECT * FROM range(3)").fetchall()
            telemetry.record_connection_profile(connection)
        finally:
            connection.close()

        self.assertEqual(len(latency.values), 1)
        self.assertGreater(latency.values[0], 0)
        self.assertEqual(rows.values, [3])

    def test_is_disabled_without_an_otlp_endpoint(self) -> None:
        with patch.dict(os.environ, {}, clear=True):
            self.assertIsInstance(create_duckdb_telemetry(), NoopDuckDBTelemetry)


if __name__ == "__main__":
    unittest.main()
