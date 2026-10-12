#!/usr/bin/env bash
set -euo pipefail

namespace="${SYNTHETICS_NAMESPACE:-synthetics}"
redpanda_namespace="${REDPANDA_NAMESPACE:-redpanda}"
redpanda_pod="${REDPANDA_POD:-redpanda-0}"
topic_partitions="${SYNTHETICS_TOPIC_PARTITIONS:-3}"
topic_replicas="${SYNTHETICS_TOPIC_REPLICAS:-1}"
topics=(
  "${SYNTHETICS_TOPIC_VN_HDBANK:-vn.synthetics.hdbank}"
  "${SYNTHETICS_TOPIC_VN_VIKKI:-vn.synthetics.vikki}"
  "${SYNTHETICS_TOPIC_VN_VIETJETAIR:-vn.synthetics.vietjetair}"
  "${SYNTHETICS_TOPIC_TH_HDBANK:-th.synthetics.hdbank}"
  "${SYNTHETICS_TOPIC_TH_VIKKI:-th.synthetics.vikki}"
  "${SYNTHETICS_TOPIC_TH_VIETJETAIR:-th.synthetics.vietjetair}"
)

command -v kubectl >/dev/null 2>&1 || { echo "Missing required command: kubectl" >&2; exit 1; }

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local synthetics bootstrap on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f - >/dev/null

if ! kubectl -n "$redpanda_namespace" wait \
  --for=condition=Ready "pod/$redpanda_pod" --timeout=3m >/dev/null 2>&1; then
  echo "Redpanda pod $redpanda_namespace/$redpanda_pod is not ready." >&2
  echo "Run ./scripts/redpanda/setup.sh first." >&2
  exit 1
fi

for topic in "${topics[@]}"; do
  if kubectl -n "$redpanda_namespace" exec "$redpanda_pod" -c redpanda -- \
    rpk topic describe "$topic" >/dev/null 2>&1; then
    echo "Redpanda topic already exists: $topic"
  else
    echo "Creating Redpanda topic: $topic"
    kubectl -n "$redpanda_namespace" exec "$redpanda_pod" -c redpanda -- \
      rpk topic create "$topic" \
      --partitions "$topic_partitions" \
      --replicas "$topic_replicas"
  fi
done

echo "Synthetics bootstrap complete: namespace '$namespace' and six regional Redpanda topics are ready."
echo "Run ./scripts/synthetics/setup.sh."
