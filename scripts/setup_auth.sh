#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace=auth
release=keycloak
secret=keycloak-credentials
credentials_file="$repo_root/k8s/auth/credentials.env"

for command in kubectl helm openssl; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

echo "Using Kubernetes context: $(kubectl config current-context)"

if ! kubectl get namespace "$namespace" >/dev/null 2>&1; then
  kubectl create namespace "$namespace"
fi

if kubectl -n "$namespace" get secret "$secret" >/dev/null 2>&1; then
  echo "Using existing Secret: $secret"
else
  if [[ ! -f "$credentials_file" ]]; then
    echo "Missing $credentials_file" >&2
    echo "Copy k8s/auth/credentials.env.example to k8s/auth/credentials.env and set both passwords." >&2
    exit 1
  fi
  kubectl -n "$namespace" create secret generic "$secret" --from-env-file="$credentials_file"
fi

tls_dir="$repo_root/k8s/auth/tls"
previous_cert="$(openssl x509 -in "$tls_dir/tls.crt" -noout -fingerprint -sha256 2>/dev/null || true)"
"$repo_root/scripts/ensure_local_tls.sh" "$tls_dir" \
  "$repo_root/k8s/auth/tls.cnf" "$repo_root/k8s/auth/ca.cnf"
current_cert="$(openssl x509 -in "$tls_dir/tls.crt" -noout -fingerprint -sha256)"
kubectl -n "$namespace" create secret generic keycloak-tls \
  --from-file=tls.crt="$tls_dir/tls.crt" \
  --from-file=tls.key="$tls_dir/tls.key" \
  --dry-run=client -o yaml | kubectl apply -f -

helm upgrade --install "$release" "$repo_root/k8s/auth" \
  --namespace "$namespace" --wait --timeout 10m
if [[ -n "$previous_cert" && "$previous_cert" != "$current_cert" ]]; then
  kubectl -n "$namespace" rollout restart deployment/"$release"
  kubectl -n "$namespace" rollout status deployment/"$release" --timeout=5m
fi

echo "Keycloak is ready. Run: ./scripts/forward.sh to forward Keycloak, Vault, and RustFS."
echo "Open https://auth.localhost after starting ./scripts/forward.sh."
