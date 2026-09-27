#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

case "${1:-}" in
  -h|--help)
    cat <<'EOF'
Usage: ./scripts/setup_spark_managed.sh [application Helm options]

Initializes the demo schema, builds the Spark image, installs the Spark
Operator, and submits the managed-table example. Extra arguments are passed
to the application chart's helm upgrade --install command.

The SPARK_* and UNITYCATALOG_* environment overrides from setup_spark.sh apply.
EOF
    exit 0
    ;;
esac

export SPARK_UNITY_SCHEMAS=demo
export SPARK_RELEASE="${SPARK_RELEASE:-spark-managed}"
export SPARK_APPLICATION_SCRIPT="$repo_root/packages/spark/examples/managed/main.py"
export SPARK_APPLICATION_SCRIPT_CONFIGMAP=unity-catalog-managed-script
export SPARK_APPLICATION_SCRIPT_MOUNT_PATH=/opt/spark/work-dir/examples/managed

exec "$repo_root/scripts/setup_spark.sh" \
  --set-string application.name=unity-catalog-managed \
  --set-string application.mainApplicationFile=local:///opt/spark/work-dir/examples/managed/main.py \
  "$@"
