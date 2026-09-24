#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace=rustfs
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
"$repo_root/scripts/ensure_local_tls.sh" "$tls_dir" \
  "$repo_root/k8s/storage/tls.cnf" "$repo_root/k8s/storage/ca.cnf" \
  rustfs_cert.pem rustfs_key.pem
kubectl -n "$namespace" create secret generic rustfs-tls \
  --from-file=rustfs_cert.pem="$tls_dir/rustfs_cert.pem" \
  --from-file=rustfs_key.pem="$tls_dir/rustfs_key.pem" \
  --dry-run=client -o yaml | kubectl apply -f -

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

# Chart 1.0.0 hardcodes HTTP probes and has no sidecar setting. Keycloak's
# public issuer is localhost, so RustFS needs a pod-local tunnel to that issuer.
kubectl -n "$namespace" patch deployment "$release" --type=strategic --field-manager=helm -p \
  '{"spec":{"template":{"spec":{"containers":[{"name":"rustfs","livenessProbe":{"httpGet":{"scheme":"HTTPS"}},"readinessProbe":{"httpGet":{"scheme":"HTTPS"}}},{"name":"keycloak-loopback","image":"alpine/socat:1.8.0.3","command":["socat"],"args":["TCP-LISTEN:8080,bind=127.0.0.1,fork,reuseaddr","TCP:keycloak.auth.svc.cluster.local:8080"],"resources":{"requests":{"cpu":"10m","memory":"16Mi"},"limits":{"memory":"64Mi"}},"securityContext":{"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]},"runAsNonRoot":true}}]}}}}'
kubectl -n "$namespace" rollout status deployment/"$release" --timeout=10m

echo "RustFS is ready over HTTPS. Run ./scripts/forward.sh for ports 9000 (S3/API) and 9001 (Console)."
