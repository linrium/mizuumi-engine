"""Dagster assets for the Sovico bronze-to-silver Spark jobs.

The Spark image contains the applications under ``/opt/spark/work-dir/jobs``.
Materializing an asset starts a short-lived Kubernetes Job that runs a
blocking ``spark-submit`` for the corresponding source catalog.
"""

from __future__ import annotations

import os
import shlex
from collections.abc import Mapping
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any

from dagster import (
    AssetIn,
    AssetKey,
    AssetObservation,
    AssetOut,
    AssetSpec,
    AssetsDefinition,
    DefaultSensorStatus,
    Definitions,
    In,
    Nothing,
    Out,
    Output,
    SensorEvaluationContext,
    SensorResult,
    graph_multi_asset,
    op,
    sensor,
)
from dagster_k8s import k8s_job_op
from kubernetes import client, config
from kubernetes.client.exceptions import ApiException


SPARK_NAMESPACE = os.environ.get("SPARK_NAMESPACE", "spark")
SPARK_IMAGE = os.environ.get(
    "SPARK_IMAGE", "mizuumi/spark:4.0.1-uc-kafka-rustfs-baked"
)
SPARK_SERVICE_ACCOUNT = os.environ.get(
    "SPARK_SERVICE_ACCOUNT", "dagster-spark-submit"
)
SPARK_TRUST_SECRET = os.environ.get("SPARK_TRUST_SECRET", "spark-rustfs-trust")
SPARK_JOBS_PATH = os.environ.get("SPARK_JOBS_PATH", "/opt/spark/work-dir/jobs")

UNITY_CATALOG_URI = os.environ.get(
    "UNITY_CATALOG_URI",
    "http://unitycatalog-server.tower.svc.cluster.local:8080",
)
S3_ENDPOINT = os.environ.get(
    "S3_ENDPOINT", "https://rustfs-svc.storage.svc.cluster.local:9000"
)
S3_REGION = os.environ.get("S3_REGION", "us-east-1")
KEYCLOAK_TOKEN_URL = os.environ.get(
    "KEYCLOAK_TOKEN_URL",
    "https://auth.mizuumi.test/realms/sovico/protocol/openid-connect/token",
)
KEYCLOAK_CA_PATH = os.environ.get(
    "KEYCLOAK_CA_PATH", "/opt/spark/trust/keycloak-ca.crt"
)
SPARK_DEPENDENCIES_PATH = os.environ.get(
    "SPARK_DEPENDENCIES_PATH", "/opt/spark/dependencies/0.6.0/*"
)

BRONZE_SCHEMA_NAME = os.environ.get("BRONZE_SCHEMA_NAME", "bronze")
BRONZE_TABLE_NAME = os.environ.get("BRONZE_TABLE_NAME", "events")
SILVER_SCHEMA_NAME = os.environ.get("SILVER_SCHEMA_NAME", "silver")

JVM_TRUST_OPTIONS = (
    "-Djavax.net.ssl.trustStore=/opt/spark/trust/cacerts "
    "-Djavax.net.ssl.trustStorePassword=changeit"
)

SPARK_APPLICATION_GROUP = "sparkoperator.k8s.io"
SPARK_APPLICATION_VERSION = "v1beta2"
SPARK_APPLICATION_PLURAL = "sparkapplications"


@dataclass(frozen=True)
class StreamingSource:
    name: str
    topic: str

    @property
    def application_name(self) -> str:
        return f"{self.name}-events-bronze"

    @property
    def topic_asset_key(self) -> AssetKey:
        return AssetKey(["redpanda", "topics", self.topic])

    @property
    def bronze_asset_key(self) -> AssetKey:
        return AssetKey([self.name, "bronze", "events"])


STREAMING_SOURCES = (
    StreamingSource("crm", "vn.crm.events"),
    StreamingSource("hdbank", "vn.hdbank.t24.events"),
    StreamingSource("vikki", "vn.vikki.t24.events"),
    StreamingSource("vietjet", "vn.vietjet.events"),
)
SOURCES = tuple(source.name for source in STREAMING_SOURCES)

SILVER_TABLES = {
    "crm": {
        "contacts": "Typed CRM contact snapshots.",
        "campaign_interactions": "Typed CRM campaign interaction records.",
    },
    "hdbank": {
        "customers": "Typed HDBank customer snapshots.",
        "loans": "Typed HDBank loan snapshots.",
        "loan_repayments": "Typed HDBank loan repayment records.",
    },
    "vikki": {
        "customers": "Typed Vikki customer snapshots.",
        "accounts": "Typed Vikki account snapshots.",
        "wallet_transactions": "Typed Vikki wallet transaction records.",
    },
    "vietjet": {
        "passengers": "Typed VietJet passenger snapshots.",
        "bookings": "Typed VietJet booking records.",
    },
}


def _credentials_secret(source: str) -> str:
    environment_name = f"SPARK_{source.upper()}_CREDENTIALS_SECRET"
    return os.environ.get(environment_name, f"spark-{source}-runtime-credentials")


def _spark_conf(source: str) -> dict[str, str]:
    credentials_secret = _credentials_secret(source)
    return {
        # Delta + Unity Catalog
        "spark.sql.extensions": "io.delta.sql.DeltaSparkSessionExtension",
        "spark.sql.catalog.spark_catalog": (
            "org.apache.spark.sql.delta.catalog.DeltaCatalog"
        ),
        f"spark.sql.catalog.{source}": "io.unitycatalog.spark.UCSingleCatalog",
        f"spark.sql.catalog.{source}.uri": UNITY_CATALOG_URI,
        f"spark.sql.catalog.{source}.renewCredential.enabled": "true",
        f"spark.sql.catalog.{source}.credScopedFs.enabled": "true",
        f"spark.sql.catalog.{source}.deltaRestApi.enabled": "false",
        "spark.sql.defaultCatalog": source,
        # S3A / RustFS
        "spark.hadoop.fs.s3.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
        "spark.hadoop.fs.s3a.impl": "org.apache.hadoop.fs.s3a.S3AFileSystem",
        "spark.hadoop.fs.s3a.endpoint": S3_ENDPOINT,
        "spark.hadoop.fs.s3a.endpoint.region": S3_REGION,
        "spark.hadoop.fs.s3a.path.style.access": "true",
        "spark.hadoop.fs.s3a.connection.ssl.enabled": "true",
        # Kubernetes submission
        "spark.kubernetes.namespace": SPARK_NAMESPACE,
        "spark.kubernetes.container.image": SPARK_IMAGE,
        "spark.kubernetes.container.image.pullPolicy": "Always",
        "spark.kubernetes.authenticate.driver.serviceAccountName": (
            SPARK_SERVICE_ACCOUNT
        ),
        "spark.kubernetes.submission.waitAppCompletion": "true",
        "spark.kubernetes.executor.deleteOnTermination": "true",
        # Trust store on both sides
        "spark.driver.extraJavaOptions": JVM_TRUST_OPTIONS,
        "spark.executor.extraJavaOptions": JVM_TRUST_OPTIONS,
        f"spark.kubernetes.driver.secrets.{SPARK_TRUST_SECRET}": "/opt/spark/trust",
        f"spark.kubernetes.executor.secrets.{SPARK_TRUST_SECRET}": "/opt/spark/trust",
        # Driver env from the source catalog's credentials Secret
        "spark.kubernetes.driver.secretKeyRef.KEYCLOAK_CLIENT_ID": (
            f"{credentials_secret}:KEYCLOAK_CLIENT_ID"
        ),
        "spark.kubernetes.driver.secretKeyRef.KEYCLOAK_CLIENT_SECRET": (
            f"{credentials_secret}:KEYCLOAK_CLIENT_SECRET"
        ),
        "spark.kubernetes.driver.secretKeyRef.CONTROLPLANE_CLIENT_ID": (
            f"{credentials_secret}:CONTROLPLANE_CLIENT_ID"
        ),
        "spark.kubernetes.driver.secretKeyRef.CONTROLPLANE_CLIENT_SECRET": (
            f"{credentials_secret}:CONTROLPLANE_CLIENT_SECRET"
        ),
        "spark.kubernetes.driver.secretKeyRef.UNITY_CATALOG_NAME": (
            f"{credentials_secret}:UNITY_CATALOG_NAME"
        ),
        "spark.kubernetes.driver.secretKeyRef.S3_ACCESS_KEY": (
            f"{credentials_secret}:S3_ACCESS_KEY"
        ),
        "spark.kubernetes.driver.secretKeyRef.S3_SECRET_KEY": (
            f"{credentials_secret}:S3_SECRET_KEY"
        ),
        # Executors only need the object-store credentials.
        "spark.kubernetes.executor.secretKeyRef.S3_ACCESS_KEY": (
            f"{credentials_secret}:S3_ACCESS_KEY"
        ),
        "spark.kubernetes.executor.secretKeyRef.S3_SECRET_KEY": (
            f"{credentials_secret}:S3_SECRET_KEY"
        ),
        # Literal application env
        "spark.kubernetes.driverEnv.KEYCLOAK_TOKEN_URL": KEYCLOAK_TOKEN_URL,
        "spark.kubernetes.driverEnv.KEYCLOAK_CA_CERT": KEYCLOAK_CA_PATH,
        "spark.kubernetes.driverEnv.UNITY_STORAGE_ROOT": (
            f"s3://unitycatalog/{source}"
        ),
        "spark.kubernetes.driverEnv.BRONZE_SCHEMA_NAME": BRONZE_SCHEMA_NAME,
        "spark.kubernetes.driverEnv.BRONZE_TABLE_NAME": BRONZE_TABLE_NAME,
        "spark.kubernetes.driverEnv.SILVER_SCHEMA_NAME": SILVER_SCHEMA_NAME,
        "spark.kubernetes.driverEnv.SPARK_DIST_CLASSPATH": (
            SPARK_DEPENDENCIES_PATH
        ),
        "spark.executorEnv.SPARK_DIST_CLASSPATH": SPARK_DEPENDENCIES_PATH,
    }


def _submit_shell_command(source: str) -> str:
    conf_flags = " ".join(
        f"--conf {shlex.quote(f'{key}={value}')}"
        for key, value in _spark_conf(source).items()
    )
    application_name = f"{source}-bronze-to-silver"
    application_file = f"local://{SPARK_JOBS_PATH}/{source}.py"
    return (
        "exec /opt/spark/bin/spark-submit "
        "--master k8s://https://kubernetes.default.svc "
        "--deploy-mode cluster "
        f"--name {shlex.quote(application_name)} "
        f"{conf_flags} "
        f"{shlex.quote(application_file)}"
    )


def _submit_op_config(source: str) -> Mapping[str, object]:
    application_name = f"{source}-bronze-to-silver"
    return {
        "namespace": SPARK_NAMESPACE,
        "image": SPARK_IMAGE,
        "command": ["/bin/bash", "-c", _submit_shell_command(source)],
        "image_pull_policy": "Always",
        "service_account_name": SPARK_SERVICE_ACCOUNT,
        "labels": {"app.kubernetes.io/name": f"{application_name}-submit"},
        # Do not copy namespace-local env references from the user-code pod
        # into the Spark namespace.
        "container_config": {"env": [], "envFrom": []},
        "merge_behavior": "SHALLOW",
        "pod_spec_config": {
            "automountServiceAccountToken": True,
            "restartPolicy": "OnFailure",
        },
    }


def _build_asset(source: str) -> AssetsDefinition:
    silver_tables = SILVER_TABLES[source]
    submit_op = k8s_job_op.configured(
        _submit_op_config(source),
        name=f"submit_{source}_bronze_to_silver_spark_job",
    )

    @op(
        name=f"mark_{source}_silver_tables_materialized",
        ins={"start_after": In(Nothing)},
        out={table: Out(Nothing) for table in silver_tables},
    )
    def mark_silver_tables_materialized():
        # This op only runs after spark-submit succeeds. Its Nothing outputs
        # let Dagster record every physical table without loading table data.
        for table in silver_tables:
            yield Output(None, output_name=table)

    def source_assets(bronze_events):
        submitted = submit_op(start_after=bronze_events)
        materialized = mark_silver_tables_materialized(start_after=submitted)
        return {table: getattr(materialized, table) for table in silver_tables}

    source_assets.__name__ = f"{source}_bronze_to_silver"
    return graph_multi_asset(
        name=f"{source}_bronze_to_silver",
        ins={
            "bronze_events": AssetIn(
                key=AssetKey([source, "bronze", "events"]),
                dagster_type=Nothing,
            )
        },
        outs={
            table: AssetOut.from_spec(
                AssetSpec(
                    key=[source, "silver", table],
                    kinds={"delta", "spark", "unity-catalog"},
                    description=description,
                    metadata={
                        "table": f"{source}.silver.{table}",
                        "catalog_type": "Unity Catalog",
                        "spark_application": f"{source}-bronze-to-silver",
                    },
                ),
                dagster_type=Nothing,
            )
            for table, description in silver_tables.items()
        },
        group_name="sovico",
        can_subset=False,
    )(source_assets)


def _build_external_streaming_assets() -> list[AssetSpec]:
    assets: list[AssetSpec] = []
    for source in STREAMING_SOURCES:
        assets.extend(
            [
                AssetSpec(
                    key=source.topic_asset_key,
                    group_name="sovico_streaming",
                    kinds={"kafka", "redpanda", "streaming"},
                    description=(
                        f"Redpanda topic consumed by the {source.name} "
                        "Spark Structured Streaming application."
                    ),
                    metadata={
                        "topic": source.topic,
                        "bootstrap_servers": (
                            "redpanda.redpanda.svc.cluster.local:9092"
                        ),
                        "managed_by": "Redpanda",
                    },
                ),
                AssetSpec(
                    key=source.bronze_asset_key,
                    deps=[source.topic_asset_key],
                    group_name="sovico_streaming",
                    kinds={"delta", "spark", "streaming"},
                    description=(
                        f"Unity Catalog bronze table continuously populated from "
                        f"{source.topic} by an external Spark application."
                    ),
                    metadata={
                        "table": f"{source.name}.bronze.events",
                        "catalog_type": "Unity Catalog",
                        "spark_application": source.application_name,
                        "kubernetes_namespace": SPARK_NAMESPACE,
                        "checkpoint": (
                            f"s3://unitycatalog/{source.name}/_checkpoints/"
                            f"{source.topic.replace('.', '_')}"
                        ),
                    },
                ),
            ]
        )
    return assets


def _copy_if_present(
    metadata: dict[str, Any], key: str, value: Any
) -> None:
    if value is not None and value != "":
        metadata[key] = value


def _application_observation(
    source: StreamingSource,
    custom_api,
    core_api,
    context: SensorEvaluationContext,
) -> AssetObservation:
    observed_at = datetime.now(timezone.utc).isoformat()
    metadata: dict[str, Any] = {
        "observed_at": observed_at,
        "spark_application": source.application_name,
        "kubernetes_namespace": SPARK_NAMESPACE,
        "topic": source.topic,
        "table": f"{source.name}.bronze.events",
    }

    try:
        application = custom_api.get_namespaced_custom_object(
            group=SPARK_APPLICATION_GROUP,
            version=SPARK_APPLICATION_VERSION,
            namespace=SPARK_NAMESPACE,
            plural=SPARK_APPLICATION_PLURAL,
            name=source.application_name,
        )
    except ApiException as error:
        if error.status != 404:
            raise
        context.log.warning(
            "SparkApplication %s/%s does not exist",
            SPARK_NAMESPACE,
            source.application_name,
        )
        metadata.update(
            {
                "stream_health": "missing",
                "application_state": "NOT_FOUND",
            }
        )
        return AssetObservation(asset_key=source.bronze_asset_key, metadata=metadata)

    status = application.get("status") or {}
    application_state = (status.get("applicationState") or {}).get("state", "UNKNOWN")
    driver_info = status.get("driverInfo") or {}
    driver_pod_name = driver_info.get("podName")

    metadata["application_state"] = application_state
    _copy_if_present(
        metadata,
        "application_error",
        status.get("errorMessage")
        or (status.get("applicationState") or {}).get("errorMessage"),
    )
    _copy_if_present(metadata, "spark_application_id", status.get("sparkApplicationId"))
    _copy_if_present(metadata, "last_submission_at", status.get("lastSubmissionAttemptTime"))
    _copy_if_present(metadata, "termination_at", status.get("terminationTime"))
    _copy_if_present(metadata, "execution_attempts", status.get("executionAttempts"))
    _copy_if_present(metadata, "submission_attempts", status.get("submissionAttempts"))
    _copy_if_present(metadata, "driver_pod", driver_pod_name)
    _copy_if_present(metadata, "spark_ui_service", driver_info.get("webUIServiceName"))

    driver_phase = None
    if driver_pod_name:
        try:
            driver_pod = core_api.read_namespaced_pod(
                name=driver_pod_name,
                namespace=SPARK_NAMESPACE,
            )
        except ApiException as error:
            if error.status != 404:
                raise
            context.log.warning(
                "Driver pod %s/%s does not exist",
                SPARK_NAMESPACE,
                driver_pod_name,
            )
            driver_phase = "NOT_FOUND"
            metadata["driver_pod_phase"] = "NOT_FOUND"
        else:
            driver_phase = driver_pod.status.phase
            _copy_if_present(metadata, "driver_pod_phase", driver_phase)
            container_statuses = driver_pod.status.container_statuses or []
            metadata["driver_restart_count"] = sum(
                container_status.restart_count
                for container_status in container_statuses
            )
            if driver_pod.metadata.creation_timestamp:
                metadata["driver_started_at"] = (
                    driver_pod.metadata.creation_timestamp.isoformat()
                )

    if application_state == "RUNNING" and driver_phase == "Running":
        stream_health = "healthy"
    elif application_state in {"FAILED", "FAILED_SUBMISSION", "INVALIDATING"}:
        stream_health = "failed"
    else:
        stream_health = "degraded"
    metadata["stream_health"] = stream_health

    return AssetObservation(asset_key=source.bronze_asset_key, metadata=metadata)


@sensor(
    name="external_spark_streaming_observation",
    minimum_interval_seconds=60,
    default_status=DefaultSensorStatus.RUNNING,
    description=(
        "Observe externally managed Spark Structured Streaming applications "
        "and attach their Kubernetes health to bronze assets."
    ),
)
def external_spark_streaming_observation(
    context: SensorEvaluationContext,
) -> SensorResult:
    config.load_incluster_config()
    custom_api = client.CustomObjectsApi()
    core_api = client.CoreV1Api()
    observations = [
        _application_observation(source, custom_api, core_api, context)
        for source in STREAMING_SOURCES
    ]
    return SensorResult(asset_events=observations)


(
    crm_bronze_to_silver,
    hdbank_bronze_to_silver,
    vikki_bronze_to_silver,
    vietjet_bronze_to_silver,
) = (_build_asset(source) for source in SOURCES)

sovico_assets = [
    crm_bronze_to_silver,
    hdbank_bronze_to_silver,
    vikki_bronze_to_silver,
    vietjet_bronze_to_silver,
]

external_streaming_assets = _build_external_streaming_assets()

defs = Definitions(
    assets=[*external_streaming_assets, *sovico_assets],
    sensors=[external_spark_streaming_observation],
)
