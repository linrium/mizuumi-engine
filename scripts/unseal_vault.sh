#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
init_file="$repo_root/k8s/vault/init.json"
if [[ ! -f "$init_file" ]]; then
  echo "Missing $init_file; supply the original unseal key before proceeding." >&2
  exit 1
fi
status="$(kubectl -n vault exec vault-0 -c vault -- vault status -format=json 2>/dev/null || true)"
if [[ -n "$status" ]] && printf '%s' "$status" | jq -e '.initialized == true and .sealed == false' >/dev/null; then
  echo "Vault is already unsealed."
  exit 0
fi
key="$(jq -er '.unseal_keys_b64[0]' "$init_file")"
kubectl -n vault exec vault-0 -c vault -- vault operator unseal "$key" >/dev/null
unset key
kubectl -n vault exec vault-0 -c vault -- vault status
