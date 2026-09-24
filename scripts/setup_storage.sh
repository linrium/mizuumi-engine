#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace=rustfs
release=rustfs
chart_version=0.12.0

for command in kubectl helm; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing to install local RustFS values on non-local context: $context" >&2; exit 1 ;;
esac

if ! kubectl -n "$namespace" get secret rustfs-credentials >/dev/null 2>&1; then
  echo "Missing rustfs-credentials Secret. Run ./scripts/bootstrap_storage.sh first." >&2
  exit 1
fi
if ! kubectl -n "$namespace" get secret rustfs-vault-ca >/dev/null 2>&1; then
  echo "Missing rustfs-vault-ca Secret. Run ./scripts/bootstrap_storage.sh first." >&2
  exit 1
fi

echo "Using Kubernetes context: $context"
helm repo add rustfs https://charts.rustfs.com --force-update
helm repo update rustfs
helm upgrade --install "$release" rustfs/rustfs \
  --version "$chart_version" \
  --values "$repo_root/k8s/storage/values.yaml" \
  --namespace "$namespace" --create-namespace \
  --wait --timeout 10m

echo "RustFS is ready. Run ./scripts/forward.sh for ports 9000 (S3/API) and 9001 (Console)."
