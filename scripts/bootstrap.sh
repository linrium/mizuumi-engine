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

if [[ -f "$repo_root/k8s/vault/init.json" ]]; then
  run_step unseal_vault.sh
else
  run_step init_vault.sh
fi

run_step bootstrap_sovico.sh
run_step bootstrap_storage.sh
run_step setup_storage.sh

if [[ "$forward" == true ]]; then
  echo "==> forward.sh (press Ctrl-C to stop port-forwards and Caddy)"
  exec "$repo_root/scripts/forward.sh"
fi

echo "Bootstrap complete. Run ./scripts/forward.sh when you need local access."
