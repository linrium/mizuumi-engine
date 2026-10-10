#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace=redpanda
release=redpanda
chart_version=5.9.18

for command in kubectl helm; do
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
    echo "Refusing to install Redpanda into non-local context: $context" >&2
    exit 1
    ;;
esac
echo "Using Kubernetes context: $context"

helm repo add redpanda https://charts.redpanda.com --force-update
helm repo update redpanda

kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f -

helm upgrade --install "$release" redpanda/redpanda \
  --version "$chart_version" \
  --values "$repo_root/k8s/redpanda/values.yaml" \
  --namespace "$namespace" --create-namespace \
  --timeout 5m

echo "Waiting for redpanda-0 to be Running..."
for attempt in $(seq 1 120); do
  pod_phase="$(kubectl -n "$namespace" get pod "$release-0" -o jsonpath='{.status.phase}' 2>/dev/null || true)"
  if [[ "$pod_phase" == "Running" ]]; then
    ready="$(kubectl -n "$namespace" get pod "$release-0" \
      -o jsonpath='{.status.containerStatuses[0].ready}' 2>/dev/null || true)"
    if [[ "$ready" == "true" ]]; then
      echo "redpanda-0 is Running and ready."
      break
    fi
  fi
  if (( attempt == 120 )); then
    echo "Timed out waiting for redpanda-0." >&2
    kubectl -n "$namespace" describe pod "$release-0" >&2
    exit 1
  fi
  sleep 5
done

echo ""
echo "Redpanda is ready."
echo ""
echo "Port-forward Kafka:          kubectl -n $namespace port-forward svc/$release 9092:9092"
echo "Port-forward Admin API:      kubectl -n $namespace port-forward svc/$release 9644:9644"
echo "Port-forward Schema Registry: kubectl -n $namespace port-forward svc/$release 8081:8081"
echo "Port-forward Pandaproxy:     kubectl -n $namespace port-forward svc/$release 8082:8082"
echo "Port-forward Console UI:     kubectl -n $namespace port-forward svc/${release}-console 8080:8080"
echo ""
echo "rpk (in-cluster):  kubectl exec -n $namespace $release-0 -- rpk topic list"
echo "rpk (local):       rpk topic list --brokers localhost:9092"
echo ""
echo "Bootstrap topics:  $repo_root/scripts/redpanda/bootstrap.sh"
