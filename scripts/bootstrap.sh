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
  run_step vault/init.sh
fi

run_step auth/bootstrap-sovico.sh
run_step storage/bootstrap.sh
run_step storage/setup.sh
run_step unitycatalog/bootstrap.sh
run_step unitycatalog/setup.sh

if [[ "$forward" == true ]]; then
  echo "==> forward.sh (verify the local gateway)"
  exec "$repo_root/scripts/forward.sh"
fi

echo "Bootstrap complete. Run ./scripts/forward.sh to verify local access."
