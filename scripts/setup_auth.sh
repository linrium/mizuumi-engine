#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace=auth
release=keycloak
secret=keycloak-credentials
credentials_file="$repo_root/k8s/auth/credentials.env"

for command in kubectl helm jq openssl; do
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
previous_gateway_cert="$(openssl x509 -in "$tls_dir/gateway.crt" -noout -fingerprint -sha256 2>/dev/null || true)"
"$repo_root/scripts/ensure_local_tls.sh" "$tls_dir" \
  "$repo_root/k8s/auth/tls.cnf" "$repo_root/k8s/auth/ca.cnf"
"$repo_root/scripts/ensure_local_tls.sh" "$tls_dir" \
  "$repo_root/k8s/auth/gateway-tls.cnf" "$repo_root/k8s/auth/ca.cnf" \
  gateway.crt gateway.key
current_cert="$(openssl x509 -in "$tls_dir/tls.crt" -noout -fingerprint -sha256)"
current_gateway_cert="$(openssl x509 -in "$tls_dir/gateway.crt" -noout -fingerprint -sha256)"
kubectl -n "$namespace" create secret generic keycloak-tls \
  --from-file=tls.crt="$tls_dir/tls.crt" \
  --from-file=tls.key="$tls_dir/tls.key" \
  --from-file=ca.crt="$tls_dir/ca.crt" \
  --dry-run=client -o yaml | kubectl apply -f -
kubectl -n "$namespace" create secret generic local-gateway-tls \
  --from-file=tls.crt="$tls_dir/gateway.crt" \
  --from-file=tls.key="$tls_dir/gateway.key" \
  --from-file=ca.crt="$tls_dir/ca.crt" \
  --dry-run=client -o yaml | kubectl apply -f -
"$repo_root/scripts/update_gateway_trust.sh"

helm upgrade --install "$release" "$repo_root/k8s/auth" \
  --namespace "$namespace" --wait --timeout 10m
"$repo_root/scripts/configure_cluster_dns.sh"
if [[ -n "$previous_cert" && "$previous_cert" != "$current_cert" ]]; then
  kubectl -n "$namespace" rollout restart deployment/"$release"
  kubectl -n "$namespace" rollout status deployment/"$release" --timeout=5m
fi
if [[ -n "$previous_gateway_cert" && "$previous_gateway_cert" != "$current_gateway_cert" ]]; then
  kubectl -n "$namespace" rollout restart deployment/"$release-gateway"
  kubectl -n "$namespace" rollout status deployment/"$release-gateway" --timeout=5m
fi

echo "Keycloak and the local gateway are ready."
echo "If needed, run: sudo ./scripts/configure_workstation_dns.sh"
echo "Run ./scripts/forward.sh --trust --open to trust the CA and open the local UIs."
