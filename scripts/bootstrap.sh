#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
forward=true

case "${1:-}" in
  "") ;;
  --no-forward) forward=false ;;
  *) echo "Usage: $0 [--no-forward]" >&2; exit 2 ;;
esac
if [[ $# -gt 1 ]]; then
  echo "Usage: $0 [--no-forward]" >&2
  exit 2
fi

run_step() {
  local script="$1"
  echo "==> $script"
  "$repo_root/scripts/$script"
}

run_step setup_auth.sh
run_step setup_vault.sh

vault_status="$(kubectl -n vault exec vault-0 -c vault -- vault status -format=json 2>/dev/null || true)"
if ! printf '%s' "$vault_status" | jq -e '.initialized | type == "boolean"' >/dev/null 2>&1; then
  echo "Could not determine whether Vault is initialized; inspect kubectl -n vault logs vault-0." >&2
  exit 1
fi
if [[ "$(printf '%s' "$vault_status" | jq -r '.initialized')" == true ]]; then
  run_step unseal_vault.sh
else
  run_step init_vault.sh
fi

run_step bootstrap_sovico.sh
run_step bootstrap_storage.sh
run_step setup_storage.sh
run_step bootstrap_unitycatalog.sh
run_step setup_unitycatalog.sh

if [[ "$forward" == true ]]; then
  echo "==> forward.sh (verify the local gateway)"
  exec "$repo_root/scripts/forward.sh"
fi

echo "Bootstrap complete. Run ./scripts/forward.sh to verify local access."
