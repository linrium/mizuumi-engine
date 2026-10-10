#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace=signoz
release=signoz
chart_version=0.145.0
storage_class=standard

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
    echo "Refusing to install SigNoz into unsupported context: $context" >&2
    echo "Select Docker Desktop or a kind context and try again." >&2
    exit 1
    ;;
esac
echo "Using Kubernetes context: $context"

if ! kubectl get storageclass "$storage_class" >/dev/null 2>&1; then
  echo "Required StorageClass '$storage_class' was not found." >&2
  echo "Available StorageClasses:" >&2
  kubectl get storageclass >&2 || true
  exit 1
fi

helm repo add signoz https://charts.signoz.io --force-update
helm repo update signoz

kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f -

helm upgrade --install "$release" signoz/signoz \
  --version "$chart_version" \
  --values "$repo_root/k8s/signoz/values.yaml" \
  --namespace "$namespace" \
  --create-namespace \
  --wait \
  --timeout 1h

echo
kubectl get pods -n "$namespace"
echo
echo "SigNoz is ready."
echo "UI:   https://console.signoz.mizuumi.test"
echo "OTLP: https://otlp.signoz.mizuumi.test"
echo "If needed, apply the shared gateway with: $repo_root/scripts/auth/setup.sh"
echo "Configure local DNS with: sudo $repo_root/scripts/configure_workstation_dns.sh"
echo "Test: $repo_root/scripts/signoz/verify.sh"
