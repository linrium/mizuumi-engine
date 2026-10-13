#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

case "${1:-}" in
  -h|--help)
    cat <<'EOF'
Usage: ./scripts/spark/deploy-streamings.sh [Helm options]

Deploys the CRM, HDBank, Vikki, and VietJet source-domain streaming jobs.
The Spark image is built once, then reused for the remaining applications.
Extra arguments are passed to every Spark application Helm release.
EOF
    exit 0
    ;;
esac

build_image="${SPARK_BUILD_IMAGE:-true}"
for source_name in crm hdbank vikki vietjet; do
  echo "Deploying $source_name streaming job..."
  SPARK_BUILD_IMAGE="$build_image" \
    "$repo_root/scripts/spark/deploy-streaming-job.sh" "$source_name" "$@"
  build_image=false
done

echo "All source-domain streaming jobs are deployed."
echo "Watch: kubectl -n ${SPARK_NAMESPACE:-spark} get sparkapplications -w"
