#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace=redpanda
release=redpanda

for command in kubectl; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s)
    ;;
  *)
    echo "Refusing to bootstrap Redpanda in non-local context: $context" >&2
    exit 1
    ;;
esac
echo "Using Kubernetes context: $context"

rpk() {
  kubectl exec -n "$namespace" "$release-0" -- rpk "$@"
}

echo "Creating topics..."

rpk topic create test-events \
  --partitions 3 \
  --replicas 1

rpk topic create test-commands \
  --partitions 3 \
  --replicas 1

rpk topic create dead-letter \
  --partitions 1 \
  --replicas 1

echo ""
echo "Topics:"
rpk topic list

echo ""
echo "Producing test message to test-events..."
echo '{"event":"bootstrap","source":"bootstrap_redpanda.sh"}' | \
  rpk topic produce test-events --key bootstrap

echo ""
echo "Consuming from test-events (group bootstrap-verify, timeout 5s)..."
rpk topic consume test-events \
  --group bootstrap-verify \
  --num 1 \
  --fetch-max-wait 5s || true

echo ""
echo "Bootstrap complete."
