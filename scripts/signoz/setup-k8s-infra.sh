#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace=signoz-infra
release=signoz-k8s-infra
chart_version=0.17.1
signoz_namespace=signoz
signoz_collector_service=signoz-otel-collector

for command in kubectl helm; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|docker-desktop)
    ;;
  *)
    echo "Refusing to install SigNoz k8s-infra into unsupported context: $context" >&2
    echo "Select Docker Desktop or a kind context and try again." >&2
    exit 1
    ;;
esac
echo "Using Kubernetes context: $context"

if ! kubectl -n "$signoz_namespace" get service "$signoz_collector_service" >/dev/null 2>&1; then
  echo "SigNoz collector Service $signoz_namespace/$signoz_collector_service was not found." >&2
  echo "Install SigNoz first with: $repo_root/scripts/signoz/setup.sh" >&2
  exit 1
fi

helm repo add signoz https://charts.signoz.io --force-update
helm repo update signoz

kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f -

helm upgrade --install "$release" signoz/k8s-infra \
  --version "$chart_version" \
  --values "$repo_root/k8s/signoz/k8s-infra-values.yaml" \
  --namespace "$namespace" \
  --create-namespace \
  --wait \
  --timeout 10m

echo
kubectl get pods -n "$namespace"
echo
echo "SigNoz k8s-infra is ready."
echo "Verify: $repo_root/scripts/signoz/verify-k8s-infra.sh"
