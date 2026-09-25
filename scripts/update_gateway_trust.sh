#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace=auth
secret=local-gateway-upstream-cas
deployment=keycloak-gateway

for command in kubectl openssl; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

bundle="$(mktemp)"
trap 'rm -f "$bundle"' EXIT
: > "$bundle"
found=0
for ca in \
  "$repo_root/k8s/auth/tls/ca.crt" \
  "$repo_root/k8s/vault/tls/ca.crt" \
  "$repo_root/k8s/storage/tls/ca.crt"; do
  if [[ -f "$ca" ]]; then
    openssl x509 -in "$ca" -noout >/dev/null
    cat "$ca" >> "$bundle"
    found=$((found + 1))
  fi
done
if (( found == 0 )); then
  echo "No local service CAs are available." >&2
  exit 1
fi

before="$(kubectl -n "$namespace" get secret "$secret" -o jsonpath='{.data.ca\.crt}' 2>/dev/null || true)"
manifest="$(kubectl -n "$namespace" create secret generic "$secret" \
  --from-file=ca.crt="$bundle" --dry-run=client -o yaml)"
printf '%s\n' "$manifest" | kubectl apply -f - >/dev/null
after="$(kubectl -n "$namespace" get secret "$secret" -o jsonpath='{.data.ca\.crt}')"

if [[ -n "$before" && "$before" != "$after" ]] && \
   kubectl -n "$namespace" get deployment "$deployment" >/dev/null 2>&1; then
  kubectl -n "$namespace" rollout restart deployment/"$deployment" >/dev/null
  kubectl -n "$namespace" rollout status deployment/"$deployment" --timeout=5m
fi

echo "Gateway trust bundle contains $found local service CA(s)."
