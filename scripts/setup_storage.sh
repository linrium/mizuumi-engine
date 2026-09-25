#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace=storage
release=rustfs
chart_version=1.0.0

for command in kubectl helm openssl; do
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

tls_dir="$repo_root/k8s/storage/tls"
previous_cert="$(openssl x509 -in "$tls_dir/rustfs_cert.pem" -noout -fingerprint -sha256 2>/dev/null || true)"
"$repo_root/scripts/ensure_local_tls.sh" "$tls_dir" \
  "$repo_root/k8s/storage/tls.cnf" "$repo_root/k8s/storage/ca.cnf" \
  rustfs_cert.pem rustfs_key.pem
current_cert="$(openssl x509 -in "$tls_dir/rustfs_cert.pem" -noout -fingerprint -sha256)"
kubectl -n "$namespace" create secret generic rustfs-tls \
  --from-file=rustfs_cert.pem="$tls_dir/rustfs_cert.pem" \
  --from-file=rustfs_key.pem="$tls_dir/rustfs_key.pem" \
  --dry-run=client -o yaml | kubectl apply -f -
"$repo_root/scripts/update_gateway_trust.sh"

echo "Using Kubernetes context: $context"
helm repo add rustfs https://charts.rustfs.com --force-update
helm repo update rustfs
helm_conflict_args=()
if [[ "$(helm version --short)" == v4* ]]; then
  helm_conflict_args+=(--force-conflicts)
fi
helm upgrade --install "$release" rustfs/rustfs \
  --version "$chart_version" \
  --values "$repo_root/k8s/storage/values.yaml" \
  "${helm_conflict_args[@]}" \
  --namespace "$namespace" --create-namespace \
  --timeout 10m

# Chart 1.0.0 hardcodes HTTP probes. Remove the obsolete loopback sidecar
# during migration from older installations.
kubectl -n "$namespace" patch deployment "$release" --type=strategic --field-manager=helm -p \
  '{"spec":{"template":{"spec":{"containers":[{"name":"rustfs","livenessProbe":{"httpGet":{"scheme":"HTTPS"}},"readinessProbe":{"httpGet":{"scheme":"HTTPS"}}},{"name":"keycloak-loopback","$patch":"delete"}]}}}}'
if [[ -n "$previous_cert" && "$previous_cert" != "$current_cert" ]]; then
  kubectl -n "$namespace" rollout restart deployment/"$release"
fi
kubectl -n "$namespace" rollout status deployment/"$release" --timeout=10m

echo "RustFS is ready. Run ./scripts/forward.sh, then use https://storage.mizuumi.test and https://api.storage.mizuumi.test."
