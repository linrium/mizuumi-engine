#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
forward=true
fresh_vault=false

usage() {
  echo "Usage: $0 [--no-forward] [--fresh-vault]" >&2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-forward) forward=false ;;
    --fresh-vault) fresh_vault=true ;;
    -h|--help) usage; exit 0 ;;
    *) usage; exit 2 ;;
  esac
  shift
done

run_step() {
  local script="$1"
  echo "==> $script"
  "$repo_root/scripts/$script"
}

run_step auth/setup.sh
run_step vault/setup.sh

vault_status="$(kubectl -n vault exec vault-0 -c vault -- vault status -format=json 2>/dev/null || true)"
if ! printf '%s' "$vault_status" | jq -e '.initialized | type == "boolean"' >/dev/null 2>&1; then
  echo "Could not determine whether Vault is initialized; inspect kubectl -n vault logs vault-0." >&2
  exit 1
fi
if [[ "$(printf '%s' "$vault_status" | jq -r '.initialized')" == true ]]; then
  run_step vault/unseal.sh
else
  init_file="$repo_root/k8s/vault/init.json"
  if [[ -e "$init_file" ]]; then
    if [[ "$fresh_vault" != true ]]; then
      echo "Vault is uninitialized, but existing unseal material is present at $init_file." >&2
      echo "Restore the original Vault volume if you need its data." >&2
      echo "For an intentional fresh local Vault, rerun with --fresh-vault to back up init.json and initialize the new empty volume." >&2
      exit 1
    fi
    backup_file="$repo_root/k8s/vault/init.$(date +%Y%m%d%H%M%S).json"
    mv "$init_file" "$backup_file"
    chmod 600 "$backup_file"
    echo "Backed up existing Vault unseal material to $backup_file."
  fi
  run_step vault/init.sh
fi

run_step auth/bootstrap-sovico.sh
run_step storage/bootstrap.sh
run_step storage/setup.sh
run_step duck-server/bootstrap.sh
run_step duck-server/setup.sh
run_step unitycatalog/bootstrap.sh
run_step unitycatalog/setup.sh
run_step chronicle/bootstrap.sh
run_step chronicle/setup.sh

if [[ "$forward" == true ]]; then
  echo "==> forward.sh (verify the local gateway)"
  exec "$repo_root/scripts/forward.sh"
fi

echo "Bootstrap complete. Run ./scripts/forward.sh to verify local access."
