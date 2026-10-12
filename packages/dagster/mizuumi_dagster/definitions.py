import os

from dagster import Definitions, ScheduleDefinition, job
from dagster_k8s import k8s_job_op


SPARK_NAMESPACE = os.environ.get("SPARK_NAMESPACE", "spark")
SPARK_APPLICATION_NAME = os.environ.get(
    "SPARK_APPLICATION_NAME", "unity-catalog-managed"
)
RERUN_IMAGE = os.environ.get("DAGSTER_KUBECTL_IMAGE", "alpine/k8s:1.31.2")
SERVICE_ACCOUNT_NAME = os.environ.get(
    "DAGSTER_SERVICE_ACCOUNT", "mizuumi-dagster"
)

# Snapshot the live SparkApplication, strip server-managed fields, delete it,
# then re-apply. The Spark Operator runs a SparkApplication exactly once, so
# recreating the custom resource is the standard way to re-trigger it.
RERUN_SCRIPT = r"""
set -euo pipefail
kubectl -n "$SPARK_NAMESPACE" get sparkapplication "$SPARK_APPLICATION_NAME" -o yaml \
  | yq 'del(
      .status,
      .metadata.resourceVersion,
      .metadata.uid,
      .metadata.generation,
      .metadata.creationTimestamp,
      .metadata.managedFields,
      .metadata.selfLink
    )' > /tmp/sparkapp.yaml
kubectl -n "$SPARK_NAMESPACE" delete sparkapplication "$SPARK_APPLICATION_NAME" \
  --ignore-not-found --wait=true
kubectl -n "$SPARK_NAMESPACE" apply -f /tmp/sparkapp.yaml
"""

rerun_unity_catalog_managed = k8s_job_op.configured(
    {
        "image": RERUN_IMAGE,
        "command": ["/bin/sh", "-c"],
        "args": [RERUN_SCRIPT],
        "env_vars": [
            f"SPARK_NAMESPACE={SPARK_NAMESPACE}",
            f"SPARK_APPLICATION_NAME={SPARK_APPLICATION_NAME}",
        ],
        "service_account_name": SERVICE_ACCOUNT_NAME,
    },
    name="rerun_unity_catalog_managed",
)


@job
def unity_catalog_managed_job() -> None:
    rerun_unity_catalog_managed()


unity_catalog_managed_schedule = ScheduleDefinition(
    name="unity_catalog_managed_every_10_minutes",
    cron_schedule="*/10 * * * *",
    job=unity_catalog_managed_job,
    execution_timezone="UTC",
    description="Re-trigger the unity-catalog-managed SparkApplication every 10 minutes.",
)


defs = Definitions(
    jobs=[unity_catalog_managed_job],
    schedules=[unity_catalog_managed_schedule],
)
