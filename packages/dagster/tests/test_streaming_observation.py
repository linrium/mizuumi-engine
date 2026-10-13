import unittest
from types import SimpleNamespace

from kubernetes.client.exceptions import ApiException

from sovico.definitions import (
    STREAMING_SOURCES,
    _application_observation,
    _build_external_streaming_assets,
)


class _Log:
    def warning(self, *_args):
        pass


class _Context:
    log = _Log()


class _CustomObjectsApi:
    def __init__(self, application=None, error=None):
        self.application = application
        self.error = error

    def get_namespaced_custom_object(self, **_kwargs):
        if self.error:
            raise self.error
        return self.application


class _CoreApi:
    def __init__(self, pod=None):
        self.pod = pod

    def read_namespaced_pod(self, **_kwargs):
        return self.pod


def _metadata_value(observation, key):
    return observation.metadata[key].value


class StreamingObservationTest(unittest.TestCase):
    def test_external_assets_connect_topic_to_bronze(self):
        source = STREAMING_SOURCES[0]
        assets = _build_external_streaming_assets()
        bronze = next(asset for asset in assets if asset.key == source.bronze_asset_key)

        self.assertEqual(
            [dependency.asset_key for dependency in bronze.deps],
            [source.topic_asset_key],
        )

    def test_running_application_emits_healthy_observation(self):
        source = STREAMING_SOURCES[0]
        application = {
            "status": {
                "applicationState": {"state": "RUNNING"},
                "driverInfo": {
                    "podName": "crm-events-bronze-driver",
                    "webUIServiceName": "crm-events-bronze-ui-svc",
                },
                "executionAttempts": 1,
            }
        }
        pod = SimpleNamespace(
            metadata=SimpleNamespace(creation_timestamp=None),
            status=SimpleNamespace(
                phase="Running",
                container_statuses=[SimpleNamespace(restart_count=2)],
            ),
        )

        observation = _application_observation(
            source,
            _CustomObjectsApi(application=application),
            _CoreApi(pod=pod),
            _Context(),
        )

        self.assertEqual(observation.asset_key, source.bronze_asset_key)
        self.assertEqual(_metadata_value(observation, "stream_health"), "healthy")
        self.assertEqual(_metadata_value(observation, "application_state"), "RUNNING")
        self.assertEqual(_metadata_value(observation, "driver_pod_phase"), "Running")
        self.assertEqual(_metadata_value(observation, "driver_restart_count"), 2)

    def test_missing_application_emits_missing_observation(self):
        source = STREAMING_SOURCES[0]
        observation = _application_observation(
            source,
            _CustomObjectsApi(error=ApiException(status=404)),
            _CoreApi(),
            _Context(),
        )

        self.assertEqual(_metadata_value(observation, "stream_health"), "missing")
        self.assertEqual(_metadata_value(observation, "application_state"), "NOT_FOUND")


if __name__ == "__main__":
    unittest.main()
