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
if [[ ! -f "$tls_dir/tls.crt" || ! -f "$tls_dir/tls.key" ]]; then
  if [[ -e "$tls_dir/tls.crt" || -e "$tls_dir/tls.key" ]]; then
    echo "Incomplete Keycloak TLS keypair in $tls_dir; refusing to replace it." >&2
    exit 1
  fi
  umask 077
  mkdir -p "$tls_dir"
  openssl req -x509 -newkey rsa:3072 -sha256 -nodes -days 365 \
    -config "$repo_root/k8s/auth/tls.cnf" \
    -keyout "$tls_dir/tls.key" -out "$tls_dir/tls.crt"
  cp "$tls_dir/tls.crt" "$tls_dir/ca.crt"
  echo "Generated local Keycloak TLS certificate in $tls_dir."
fi
if [[ ! -f "$tls_dir/ca.crt" ]]; then
  echo "Missing $tls_dir/ca.crt; refusing to deploy." >&2
  exit 1
fi
kubectl -n "$namespace" create secret generic keycloak-tls \
  --from-file=tls.crt="$tls_dir/tls.crt" \
  --from-file=tls.key="$tls_dir/tls.key" \
  --dry-run=client -o yaml | kubectl apply -f -

helm upgrade --install "$release" "$repo_root/k8s/auth" \
  --namespace "$namespace" --wait --timeout 10m

echo "Keycloak is ready. Run: ./scripts/forward.sh to forward Keycloak, Vault, and RustFS."
echo "Open https://localhost:8080 (trust k8s/auth/tls/ca.crt in your browser)."
