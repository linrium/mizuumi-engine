#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
init_file="$repo_root/k8s/vault/init.json"
status="$(kubectl -n vault exec vault-0 -c vault -- vault status -format=json 2>/dev/null || true)"
if ! printf '%s' "$status" | jq -e '.initialized | type == "boolean"' >/dev/null 2>&1; then
  echo "Could not determine whether Vault is initialized; inspect kubectl -n vault logs vault-0." >&2
  exit 1
fi
if [[ "$(printf '%s' "$status" | jq -r '.initialized')" == false ]]; then
  echo "Vault is not initialized. The current volume cannot be unsealed with $init_file." >&2
  echo "Restore the original Vault volume if you need its data. For a fresh start, back up and move init.json out of k8s/vault/, then run ./scripts/init_vault.sh." >&2
  exit 1
fi
if [[ ! -f "$init_file" ]]; then
  echo "Missing $init_file; supply the original unseal key before proceeding." >&2
  exit 1
fi
if printf '%s' "$status" | jq -e '.sealed == false' >/dev/null; then
  echo "Vault is already unsealed."
  exit 0
fi
key="$(jq -er '.unseal_keys_b64[0]' "$init_file")"
kubectl -n vault exec vault-0 -c vault -- vault operator unseal "$key" >/dev/null
unset key
kubectl -n vault exec vault-0 -c vault -- vault status
