#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source_name="${1:-}"

usage() {
  cat <<'EOF'
Usage: ./scripts/spark/deploy-streaming-job.sh <crm|hdbank|vikki|vietjet> [Helm options]

Ensures the source topic and Unity Catalog resources exist, then deploys one
source-domain Spark Structured Streaming application. Extra arguments are
passed to the Spark application Helm release.
EOF
}

case "$source_name" in
  -h|--help|"") usage; [[ -n "$source_name" ]] && exit 0 || exit 2 ;;
  crm) topic="vn.crm.events" ;;
  hdbank) topic="vn.hdbank.t24.events" ;;
  vikki) topic="vn.vikki.t24.events" ;;
  vietjet) topic="vn.vietjet.events" ;;
  *) echo "Unknown streaming job: $source_name" >&2; usage >&2; exit 2 ;;
esac
shift

redpanda_namespace="${REDPANDA_NAMESPACE:-redpanda}"
redpanda_pod="${REDPANDA_POD:-redpanda-0}"
topic_partitions="${KAFKA_TOPIC_PARTITIONS:-3}"
topic_replicas="${KAFKA_TOPIC_REPLICAS:-1}"
application_name="$source_name-events-bronze"

echo "Ensuring Redpanda topic $topic..."
topic_create_output="$(kubectl -n "$redpanda_namespace" exec "$redpanda_pod" -c redpanda -- \
  rpk topic create "$topic" --partitions "$topic_partitions" --replicas "$topic_replicas" 2>&1)" || {
  if [[ "$topic_create_output" != *TOPIC_ALREADY_EXISTS* && "$topic_create_output" != *"already exists"* ]]; then
    printf '%s\n' "$topic_create_output" >&2
    exit 1
  fi
}

export SPARK_UNITY_CATALOG="$source_name"
export SPARK_UNITY_SCHEMAS="bronze"
export SPARK_UNITY_STORAGE_ROOT="s3://unitycatalog/$source_name"
export SPARK_UNITY_EXTERNAL_LOCATION="rustfs_$source_name"
export SPARK_RELEASE="spark-$source_name-streaming"
export SPARK_CREDENTIALS_SECRET="spark-$source_name-runtime-credentials"
export SPARK_APPLICATION_SOURCE="image"

exec "$repo_root/scripts/spark/setup.sh" \
  --values "$repo_root/k8s/spark/streaming-values.yaml" \
  --set-string "application.name=$application_name" \
  --set-string "application.mainApplicationFile=local:///opt/spark/work-dir/streamings/$source_name.py" \
  --set-string "application.env.KAFKA_TOPIC=$topic" \
  --set-string "flare.driverServiceName=$application_name-driver" \
  --set-string "flare.executorServiceName=$application_name-executor" \
  "$@"
