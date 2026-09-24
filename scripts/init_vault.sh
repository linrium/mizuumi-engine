#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
init_file="$repo_root/k8s/vault/init.json"
for command in kubectl jq; do
  command -v "$command" >/dev/null || { echo "Missing $command" >&2; exit 1; }
done
if [[ -e "$init_file" ]]; then
  echo "Refusing to overwrite $init_file. Use ./scripts/unseal_vault.sh instead." >&2
  exit 1
fi
kubectl -n vault get pod vault-0 >/dev/null
status="$(kubectl -n vault exec vault-0 -c vault -- vault status -format=json 2>/dev/null || true)"
if [[ -n "$status" ]] && printf '%s' "$status" | jq -e '.initialized == true' >/dev/null; then
  echo "Vault is already initialized. Recover its original unseal material; do not reinitialize." >&2
  exit 1
fi
umask 077
kubectl -n vault exec vault-0 -c vault -- vault operator init -key-shares=1 -key-threshold=1 -format=json > "$init_file"
if ! jq -e '.unseal_keys_b64[0] and .root_token' "$init_file" >/dev/null; then
  echo "Initialization output needs manual inspection: $init_file" >&2
  exit 1
fi
chmod 600 "$init_file"
echo "Vault initialized. CRITICAL: securely back up $init_file outside this machine; loss of the key makes Vault and RustFS SSE-KMS data inaccessible."
"$repo_root/scripts/unseal_vault.sh"
