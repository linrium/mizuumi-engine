"""Dagster asset that runs the managed Unity Catalog Spark job via spark-submit.

Each materialization launches a short-lived Kubernetes Job (via
``execute_k8s_job``) in the Spark namespace running the ``mizuumi/spark``
image. The Job executes ``spark-submit --deploy-mode cluster`` against the
in-cluster API server, which spawns the Spark driver pod and blocks until
the application finishes (``spark.kubernetes.submission.waitAppCompletion``).
"""

import os

from dagster import (
    AssetSelection,
    Definitions,
    ScheduleDefinition,
    define_asset_job,
    graph_asset,
)
from dagster_k8s import k8s_job_op


SPARK_NAMESPACE = os.environ.get("SPARK_NAMESPACE", "spark")
SPARK_IMAGE = os.environ.get(
    "SPARK_IMAGE", "mizuumi/spark:4.0.1-uc-kafka-rustfs-baked"
)
SPARK_APP_NAME = os.environ.get("SPARK_APP_NAME", "unity-catalog-managed")
SPARK_SERVICE_ACCOUNT = os.environ.get(
    "SPARK_SERVICE_ACCOUNT", "dagster-spark-submit"
)
SPARK_CREDENTIALS_SECRET = os.environ.get(
    "SPARK_CREDENTIALS_SECRET", "spark-runtime-credentials"
)
SPARK_TRUST_SECRET = os.environ.get("SPARK_TRUST_SECRET", "spark-rustfs-trust")

SPARK_SCRIPT_CONFIGMAP = os.environ.get(
    "SPARK_SCRIPT_CONFIGMAP", "managed-orders-spark-script"
)
SPARK_SCRIPT_FILENAME = os.environ.get("SPARK_SCRIPT_FILENAME", "main.py")
SPARK_SCRIPT_MOUNT_PATH = os.environ.get(
    "SPARK_SCRIPT_MOUNT_PATH", "/opt/spark/work-dir/managed"
)

UNITY_CATALOG = os.environ.get("UNITY_CATALOG_NAME", "unity")
UNITY_CATALOG_URI = os.environ.get(
    "UNITY_CATALOG_URI",
    "http://unitycatalog-server.tower.svc.cluster.local:8080",
)
UNITY_STORAGE_ROOT = os.environ.get(
    "UNITY_STORAGE_ROOT", "s3://unitycatalog/spark"
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

JVM_TRUST_OPTIONS = (
    "-Djavax.net.ssl.trustStore=/opt/spark/trust/cacerts "
    "-Djavax.net.ssl.trustStorePassword=changeit"
)

DRIVER_POD_TEMPLATE_PATH = "/tmp/driver-pod-template.yaml"


def _spark_conf():
    return {
        # Delta + Unity Catalog
        "spark.sql.extensions": "io.delta.sql.DeltaSparkSessionExtension",
        "spark.sql.catalog.spark_catalog": "org.apache.spark.sql.delta.catalog.DeltaCatalog",
        f"spark.sql.catalog.{UNITY_CATALOG}": "io.unitycatalog.spark.UCSingleCatalog",
        f"spark.sql.catalog.{UNITY_CATALOG}.uri": UNITY_CATALOG_URI,
        f"spark.sql.catalog.{UNITY_CATALOG}.renewCredential.enabled": "true",
        f"spark.sql.catalog.{UNITY_CATALOG}.credScopedFs.enabled": "true",
        f"spark.sql.catalog.{UNITY_CATALOG}.deltaRestApi.enabled": "false",
        "spark.sql.defaultCatalog": UNITY_CATALOG,
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
        "spark.kubernetes.authenticate.driver.serviceAccountName": SPARK_SERVICE_ACCOUNT,
        "spark.kubernetes.submission.waitAppCompletion": "true",
        "spark.kubernetes.executor.deleteOnTermination": "true",
        # Trust store on both sides
        "spark.driver.extraJavaOptions": JVM_TRUST_OPTIONS,
        "spark.executor.extraJavaOptions": JVM_TRUST_OPTIONS,
        f"spark.kubernetes.driver.secrets.{SPARK_TRUST_SECRET}": "/opt/spark/trust",
        f"spark.kubernetes.executor.secrets.{SPARK_TRUST_SECRET}": "/opt/spark/trust",
        # Driver env from the credentials Secret
        "spark.kubernetes.driver.secretKeyRef.KEYCLOAK_CLIENT_ID":
            f"{SPARK_CREDENTIALS_SECRET}:KEYCLOAK_CLIENT_ID",
        "spark.kubernetes.driver.secretKeyRef.KEYCLOAK_CLIENT_SECRET":
            f"{SPARK_CREDENTIALS_SECRET}:KEYCLOAK_CLIENT_SECRET",
        "spark.kubernetes.driver.secretKeyRef.UNITY_CATALOG_NAME":
            f"{SPARK_CREDENTIALS_SECRET}:UNITY_CATALOG_NAME",
        "spark.kubernetes.driver.secretKeyRef.S3_ACCESS_KEY":
            f"{SPARK_CREDENTIALS_SECRET}:S3_ACCESS_KEY",
        "spark.kubernetes.driver.secretKeyRef.S3_SECRET_KEY":
            f"{SPARK_CREDENTIALS_SECRET}:S3_SECRET_KEY",
        # Executor env from the credentials Secret (S3 only)
        "spark.kubernetes.executor.secretKeyRef.S3_ACCESS_KEY":
            f"{SPARK_CREDENTIALS_SECRET}:S3_ACCESS_KEY",
        "spark.kubernetes.executor.secretKeyRef.S3_SECRET_KEY":
            f"{SPARK_CREDENTIALS_SECRET}:S3_SECRET_KEY",
        # Literal driver env
        "spark.kubernetes.driverEnv.KEYCLOAK_TOKEN_URL": KEYCLOAK_TOKEN_URL,
        "spark.kubernetes.driverEnv.KEYCLOAK_CA_CERT": KEYCLOAK_CA_PATH,
        "spark.kubernetes.driverEnv.UNITY_STORAGE_ROOT": UNITY_STORAGE_ROOT,
        "spark.kubernetes.driverEnv.SPARK_DIST_CLASSPATH": SPARK_DEPENDENCIES_PATH,
        "spark.executorEnv.SPARK_DIST_CLASSPATH": SPARK_DEPENDENCIES_PATH,
        # Script ConfigMap is mounted via the driver pod template (configMap
        # volumes are not supported by `spark.kubernetes.driver.volumes.*`).
        "spark.kubernetes.driver.podTemplateFile": DRIVER_POD_TEMPLATE_PATH,
    }


def _driver_pod_template() -> str:
    return (
        "apiVersion: v1\n"
        "kind: Pod\n"
        "spec:\n"
        "  containers:\n"
        "    - name: spark-kubernetes-driver\n"
        "      volumeMounts:\n"
        "        - name: application-script\n"
        f"          mountPath: {SPARK_SCRIPT_MOUNT_PATH}\n"
        "          readOnly: true\n"
        "  volumes:\n"
        "    - name: application-script\n"
        "      configMap:\n"
        f"        name: {SPARK_SCRIPT_CONFIGMAP}\n"
    )


def _submit_shell_command() -> str:
    """Render the driver pod template, then run the blocking Spark submission."""
    main_py = f"local://{SPARK_SCRIPT_MOUNT_PATH}/{SPARK_SCRIPT_FILENAME}"
    conf_flags = " ".join(
        f'--conf "{key}={value}"' for key, value in _spark_conf().items()
    )
    pod_template = _driver_pod_template()
    return (
        "set -euo pipefail\n"
        f"cat > {DRIVER_POD_TEMPLATE_PATH} <<'POD_TEMPLATE_EOF'\n"
        f"{pod_template}"
        "POD_TEMPLATE_EOF\n"
        "exec /opt/spark/bin/spark-submit "
        "--master k8s://https://kubernetes.default.svc "
        "--deploy-mode cluster "
        f"--name {SPARK_APP_NAME} "
        f"{conf_flags} "
        f"{main_py}"
    )


spark_submit_op = k8s_job_op.configured(
    {
        "namespace": SPARK_NAMESPACE,
        "image": SPARK_IMAGE,
        "command": ["/bin/bash", "-c", _submit_shell_command()],
        "image_pull_policy": "Always",
        "service_account_name": SPARK_SERVICE_ACCOUNT,
        "labels": {"app.kubernetes.io/name": f"{SPARK_APP_NAME}-submit"},
        # The Dagster run pod inherits namespace-local env/envFrom entries from
        # the user-code deployment. Do not copy those references into the
        # Spark namespace, where their ConfigMaps do not exist.
        "container_config": {"env": [], "envFrom": []},
        "merge_behavior": "SHALLOW",
        # execute_k8s_job deliberately skips pod-log streaming for OnFailure.
        "pod_spec_config": {
            "automountServiceAccountToken": True,
            "restartPolicy": "OnFailure",
        },
    },
    name="submit_managed_orders_spark_job",
)


@graph_asset(
    key=["unity", "demo", "orders"],
    kinds={"spark"},
    group_name="unity_catalog",
    description=(
        "Managed Delta table in Unity Catalog. Each materialization launches "
        "a blocking `spark-submit` Kubernetes Job and succeeds when Spark "
        "finishes successfully."
    ),
)
def unity_catalog_managed_orders():
    return spark_submit_op()


unity_catalog_managed_orders_job = define_asset_job(
    name="unity_catalog_managed_orders_job",
    selection=AssetSelection.assets(unity_catalog_managed_orders),
)

unity_catalog_managed_orders_schedule = ScheduleDefinition(
    name="unity_catalog_managed_orders_every_10_minutes",
    cron_schedule="*/10 * * * *",
    job=unity_catalog_managed_orders_job,
    execution_timezone="UTC",
    description="Re-materialize unity.demo.orders every 10 minutes.",
)


defs = Definitions(
    assets=[unity_catalog_managed_orders],
    jobs=[unity_catalog_managed_orders_job],
    schedules=[unity_catalog_managed_orders_schedule],
)
