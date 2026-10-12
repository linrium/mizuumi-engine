#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

case "${1:-}" in
  -h|--help)
    cat <<'EOF'
Usage: ./scripts/spark/setup-managed.sh [application Helm options]

Initializes the demo schema, builds the Spark image, installs the Spark
Operator, and submits the managed-table example. Extra arguments are passed
to the application chart's helm upgrade --install command.

The SPARK_* and UNITYCATALOG_* environment overrides from scripts/spark/setup.sh apply.
EOF
    exit 0
    ;;
esac

export SPARK_UNITY_SCHEMAS=demo
export SPARK_RELEASE="${SPARK_RELEASE:-spark-managed}"
export SPARK_APPLICATION_SCRIPT="$repo_root/packages/spark/examples/managed/main.py"
# Default to `configmap`: the script is baked into a ConfigMap at install
# time and mounted into the driver pod. The rustfs source relies on a
# presigned URL that expires (max 7 days), which breaks any scheduler that
# re-triggers this SparkApplication after the URL lifetime.
export SPARK_APPLICATION_SOURCE="${SPARK_APPLICATION_SOURCE:-configmap}"
export SPARK_APPLICATION_URL_SECRET="${SPARK_APPLICATION_URL_SECRET:-unity-catalog-managed-artifact}"
export SPARK_APPLICATION_SCRIPT_CONFIGMAP=unity-catalog-managed-script
export SPARK_APPLICATION_SCRIPT_MOUNT_PATH=/opt/spark/work-dir/examples/managed

exec "$repo_root/scripts/spark/setup.sh" \
  --set-string application.name=unity-catalog-managed \
  --values "$repo_root/k8s/spark/managed-values.yaml" \
  "$@"
