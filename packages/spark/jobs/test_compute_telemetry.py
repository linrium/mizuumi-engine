from __future__ import annotations

import unittest

from compute_telemetry import OtelComputeTelemetry


class RecordingInstrument:
    def __init__(self) -> None:
        self.values: list[tuple[float, dict[str, str]]] = []

    def record(self, value: float, attributes: dict[str, str]) -> None:
        self.values.append((value, attributes))

    def add(self, value: float, attributes: dict[str, str]) -> None:
        self.values.append((value, attributes))


class RecordingMeter:
    def __init__(self) -> None:
        self.instruments: dict[str, RecordingInstrument] = {}

    def create_histogram(self, name: str, **_: object) -> RecordingInstrument:
        instrument = RecordingInstrument()
        self.instruments[name] = instrument
        return instrument

    def create_counter(self, name: str, **_: object) -> RecordingInstrument:
        instrument = RecordingInstrument()
        self.instruments[name] = instrument
        return instrument


class RecordingProvider:
    def __init__(self) -> None:
        self.stopped = False

    def shutdown(self) -> None:
        self.stopped = True


class ComputeTelemetryTests(unittest.TestCase):
    def test_records_canonical_operation_metrics_with_shared_attributes(self) -> None:
        provider = RecordingProvider()
        meter = RecordingMeter()
        telemetry = OtelComputeTelemetry(provider, meter)

        telemetry.record_operation(
            duration_seconds=1.25,
            workload_name="crm.silver.contacts",
            status="success",
            output_rows=42,
        )

        duration, attributes = meter.instruments[
            "mizuumi.compute.operation.duration"
        ].values[0]
        self.assertEqual(duration, 1.25)
        self.assertEqual(attributes["mizuumi.compute.engine"], "spark")
        self.assertEqual(attributes["mizuumi.workload.name"], "crm.silver.contacts")
        self.assertEqual(
            meter.instruments["mizuumi.compute.operation.output.rows"].values[0][0],
            42,
        )
        self.assertEqual(
            meter.instruments["mizuumi.compute.operation.count"].values[0][0], 1
        )

        telemetry.shutdown()
        self.assertTrue(provider.stopped)


if __name__ == "__main__":
    unittest.main()
