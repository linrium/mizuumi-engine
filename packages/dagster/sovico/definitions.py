"""Dagster assets for the Sovico bronze-to-silver Spark jobs.

The Spark image contains the applications under ``/opt/spark/work-dir/jobs``.
Materializing an asset starts a short-lived Kubernetes Job that runs a
blocking ``spark-submit`` for the corresponding source catalog.
"""

from __future__ import annotations

import os
import shlex
from collections.abc import Mapping

from dagster import AssetsDefinition, Definitions, graph_asset
from dagster_k8s import k8s_job_op


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

SOURCES = ("crm", "hdbank", "vikki", "vietjet")


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
    submit_op = k8s_job_op.configured(
        _submit_op_config(source),
        name=f"submit_{source}_bronze_to_silver_spark_job",
    )

    def source_asset():
        return submit_op()

    source_asset.__name__ = f"{source}_bronze_to_silver"
    return graph_asset(
        key=[source, "silver"],
        kinds={"spark"},
        group_name="sovico",
        description=(
            f"Rebuild the {source} silver tables from {source}.bronze.events "
            "with the source-domain Spark batch job."
        ),
    )(source_asset)


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

defs = Definitions(assets=sovico_assets)
