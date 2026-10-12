#!/usr/bin/env bash
set -euo pipefail

redpanda_namespace="${REDPANDA_NAMESPACE:-redpanda}"
redpanda_pod="${REDPANDA_POD:-redpanda-0}"
dry_run=false
topics=(
  "${SYNTHETICS_TOPIC_VN_HDBANK:-vn.synthetics.hdbank}"
  "${SYNTHETICS_TOPIC_VN_VIKKI:-vn.synthetics.vikki}"
  "${SYNTHETICS_TOPIC_VN_VIETJETAIR:-vn.synthetics.vietjetair}"
  "${SYNTHETICS_TOPIC_TH_HDBANK:-th.synthetics.hdbank}"
  "${SYNTHETICS_TOPIC_TH_VIKKI:-th.synthetics.vikki}"
  "${SYNTHETICS_TOPIC_TH_VIETJETAIR:-th.synthetics.vietjetair}"
)

usage() {
  cat <<'EOF'
Usage: ./scripts/synthetics/cleanup.sh [--dry-run]

Deletes only the Vietnam and Thailand HDBank, Vikki, and VietJet Air topics
used by the synthetics service. The Helm release and namespace are not changed.

Options:
  --dry-run   List topics that would be deleted without changing Redpanda.
  -h, --help  Show this help.

Environment overrides:
  REDPANDA_NAMESPACE             Redpanda namespace (default: redpanda)
  REDPANDA_POD                   Redpanda pod (default: redpanda-0)
  SYNTHETICS_TOPIC_VN_HDBANK       Vietnam HDBank topic
  SYNTHETICS_TOPIC_VN_VIKKI        Vietnam Vikki topic
  SYNTHETICS_TOPIC_VN_VIETJETAIR   Vietnam VietJet Air topic
  SYNTHETICS_TOPIC_TH_HDBANK       Thailand HDBank topic
  SYNTHETICS_TOPIC_TH_VIKKI        Thailand Vikki topic
  SYNTHETICS_TOPIC_TH_VIETJETAIR   Thailand VietJet Air topic
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

command -v kubectl >/dev/null 2>&1 || { echo "Missing required command: kubectl" >&2; exit 1; }

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing synthetics topic cleanup on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

if ! kubectl -n "$redpanda_namespace" wait \
  --for=condition=Ready "pod/$redpanda_pod" --timeout=3m >/dev/null 2>&1; then
  echo "Redpanda pod $redpanda_namespace/$redpanda_pod is not ready." >&2
  exit 1
fi

topics_found=0
for topic in "${topics[@]}"; do
  if ! kubectl -n "$redpanda_namespace" exec "$redpanda_pod" -c redpanda -- \
    rpk topic describe "$topic" >/dev/null 2>&1; then
    echo "Redpanda topic is already absent: $topic"
    continue
  fi

  topics_found=$((topics_found + 1))
  if [[ "$dry_run" == true ]]; then
    echo "Would delete Redpanda topic: $topic"
  else
    echo "Deleting Redpanda topic: $topic"
    kubectl -n "$redpanda_namespace" exec "$redpanda_pod" -c redpanda -- \
      rpk topic delete "$topic"
  fi
done

if (( topics_found == 0 )); then
  echo "No synthetics Redpanda topics were found."
elif [[ "$dry_run" == true ]]; then
  echo "Dry run complete; no topics were deleted."
else
  echo "Synthetics Redpanda topic cleanup complete."
fi
