#!/usr/bin/env bash
set -euo pipefail

namespace="${DAGSTER_NAMESPACE:-dagster}"
release="${DAGSTER_RELEASE:-dagster}"
dry_run=false

usage() {
  cat <<'EOF'
Usage: ./scripts/dagster/cleanup-run-pods.sh [--dry-run]

Deletes terminal Dagster run-worker Jobs and their owned pods. Active runs and
long-lived Dagster services are not affected.

Options:
  --dry-run  List the Jobs that would be deleted without changing the cluster.
  -h, --help Show this help.

Environment overrides:
  DAGSTER_NAMESPACE  Namespace containing Dagster (default: dagster)
  DAGSTER_RELEASE    Helm release label to select (default: dagster)
EOF
}

while (($# > 0)); do
  case "$1" in
    --dry-run) dry_run=true ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 1 ;;
  esac
  shift
done

for command in kubectl jq; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

selector="app.kubernetes.io/component=run_worker,app.kubernetes.io/instance=$release"
job_names="$(
  kubectl -n "$namespace" get jobs -l "$selector" -o json |
    jq -r '
      .items[]
      | select(any(.status.conditions[]?;
          .status == "True" and (.type == "Complete" or .type == "Failed")))
      | .metadata.name
    '
)"

if [[ -z "$job_names" ]]; then
  echo "No terminal Dagster run-worker Jobs found in namespace $namespace."
  exit 0
fi

if [[ "$dry_run" == true ]]; then
  echo "Terminal Dagster run-worker Jobs in namespace $namespace:"
  while IFS= read -r job_name; do
    echo "$job_name"
  done <<< "$job_names"
  exit 0
fi

while IFS= read -r job_name; do
  kubectl -n "$namespace" delete job "$job_name"
done <<< "$job_names"
