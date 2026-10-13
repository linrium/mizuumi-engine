#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

case "${1:-}" in
  -h|--help)
    cat <<'EOF'
Usage: ./scripts/spark/deploy-jobs.sh [Helm options]

Runs the CRM, HDBank, Vikki, and VietJet bronze-to-silver ETL jobs once.
Each invocation rebuilds the silver tables from the current bronze data and exits.
The Spark image is built once and reused by the remaining applications.
Extra arguments are passed to every Spark application Helm release.
EOF
    exit 0
    ;;
esac

build_image="${SPARK_BUILD_IMAGE:-true}"
run_revision="$(date +%s)-$$"
for source_name in crm hdbank vikki vietjet; do
  application_name="$source_name-bronze-to-silver"
  echo "Deploying $source_name bronze-to-silver ETL job..."
  SPARK_BUILD_IMAGE="$build_image" \
  SPARK_UNITY_CATALOG="$source_name" \
  SPARK_UNITY_SCHEMAS="bronze silver" \
  SPARK_UNITY_STORAGE_ROOT="s3://unitycatalog/$source_name" \
  SPARK_UNITY_EXTERNAL_LOCATION="rustfs_$source_name" \
  SPARK_RELEASE="spark-$source_name-etl" \
  SPARK_CREDENTIALS_SECRET="spark-$source_name-runtime-credentials" \
  SPARK_APPLICATION_SOURCE=image \
    "$repo_root/scripts/spark/setup.sh" \
      --values "$repo_root/k8s/spark/job-values.yaml" \
      --set-string "application.name=$application_name" \
      --set-string \
        "application.mainApplicationFile=local:///opt/spark/work-dir/jobs/$source_name.py" \
      --set-string "application.scriptRevision=$run_revision" \
      --set-string "flare.driverServiceName=$application_name-driver" \
      --set-string "flare.executorServiceName=$application_name-executor" \
      "$@"
  build_image=false
done

echo "All source-domain bronze-to-silver ETL jobs are deployed."
echo "Watch: kubectl -n ${SPARK_NAMESPACE:-spark} get sparkapplications -w"
