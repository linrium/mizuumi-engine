#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

case "${1:-}" in
  -h|--help)
    cat <<'EOF'
Usage: ./scripts/setup_spark_streaming.sh [application Helm options]

Creates the payments catalog and bronze schema in Unity Catalog, builds the
Spark image, installs the Spark Operator, and submits the Redpanda ->
bronze streaming example. Extra arguments are passed to the application
chart's helm upgrade --install command.

The SPARK_* and UNITYCATALOG_* environment overrides from setup_spark.sh apply.
EOF
    exit 0
    ;;
esac

export SPARK_UNITY_CATALOG="${SPARK_UNITY_CATALOG:-payments}"
export SPARK_UNITY_SCHEMAS="${SPARK_UNITY_SCHEMAS:-bronze}"
export SPARK_UNITY_STORAGE_ROOT="${SPARK_UNITY_STORAGE_ROOT:-s3://unitycatalog/payments}"
export SPARK_UNITY_EXTERNAL_LOCATION="${SPARK_UNITY_EXTERNAL_LOCATION:-rustfs_payments}"
export SPARK_RELEASE="${SPARK_RELEASE:-spark-streaming}"
export SPARK_APPLICATION_SCRIPT="$repo_root/packages/spark/examples/streaming/main.py"
export SPARK_APPLICATION_SOURCE="${SPARK_APPLICATION_SOURCE:-rustfs}"
export SPARK_APPLICATION_URL_SECRET="${SPARK_APPLICATION_URL_SECRET:-t24-payment-events-bronze-artifact}"
export SPARK_APPLICATION_SCRIPT_CONFIGMAP=t24-payment-events-bronze-script
export SPARK_APPLICATION_SCRIPT_MOUNT_PATH=/opt/spark/work-dir/examples/streaming

exec "$repo_root/scripts/setup_spark.sh" \
  --set-string application.name=t24-payment-events-bronze \
  --values "$repo_root/k8s/spark/streaming-values.yaml" \
  "$@"
