#!/usr/bin/env bash
set -euo pipefail

if ! command -v kubectl >/dev/null 2>&1; then
  echo "Missing required command: kubectl" >&2
  exit 1
fi

keycloak_only=false
case "${1:-}" in
  "") ;;
  --keycloak-only) keycloak_only=true ;;
  *) echo "Usage: $0 [--keycloak-only]" >&2; exit 2 ;;
esac
if [[ $# -gt 1 ]]; then
  echo "Usage: $0 [--keycloak-only]" >&2
  exit 2
fi

kubectl -n auth get service keycloak >/dev/null
storage_available=false
if [[ "$keycloak_only" == false ]]; then
  kubectl -n vault get pod vault-0 >/dev/null
fi
if [[ "$keycloak_only" == false ]] && kubectl -n rustfs get service rustfs-svc >/dev/null 2>&1; then
  storage_available=true
fi

auth_pid=""
vault_pid=""
storage_pid=""
auth_started=0
vault_started=0
storage_started=0
auth_failures=0
vault_failures=0
storage_failures=0
cleanup() {
  if [[ -n "$auth_pid" ]]; then
    kill "$auth_pid" 2>/dev/null || true
    wait "$auth_pid" 2>/dev/null || true
  fi
  if [[ -n "$vault_pid" ]]; then
    kill "$vault_pid" 2>/dev/null || true
    wait "$vault_pid" 2>/dev/null || true
  fi
  if [[ -n "$storage_pid" ]]; then
    kill "$storage_pid" 2>/dev/null || true
    wait "$storage_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

start_forward() {
  case "$1" in
    auth)
      kubectl -n auth port-forward --address 127.0.0.1 service/keycloak 8080:8080 &
      auth_pid=$!
      auth_started=$SECONDS
      ;;
    vault)
      kubectl -n vault port-forward --address 127.0.0.1 pod/vault-0 8200:8200 &
      vault_pid=$!
      vault_started=$SECONDS
      ;;
    storage)
      kubectl -n rustfs port-forward --address 127.0.0.1 service/rustfs-svc 9000:9000 9001:9001 &
      storage_pid=$!
      storage_started=$SECONDS
      ;;
  esac
}

check_forward() {
  local name="$1" pid started failures
  case "$name" in
    auth) pid="$auth_pid"; started="$auth_started"; failures="$auth_failures" ;;
    vault) pid="$vault_pid"; started="$vault_started"; failures="$vault_failures" ;;
    storage) pid="$storage_pid"; started="$storage_started"; failures="$storage_failures" ;;
  esac
  if kill -0 "$pid" 2>/dev/null; then
    return
  fi
  wait "$pid" || true
  # A healthy tunnel may be dropped after serving a request. Only count
  # failures that happen immediately (for example, a local port conflict).
  if (( SECONDS - started < 3 )); then
    failures=$((failures + 1))
  else
    failures=1
  fi
  if (( failures >= 5 )); then
    echo "$name port-forward failed repeatedly; stopping. Check the pod and local port." >&2
    exit 1
  fi
  echo "$name port-forward dropped; reconnecting in 2 seconds ($failures/5)." >&2
  sleep 2
  case "$name" in
    auth) auth_failures="$failures" ;;
    vault) vault_failures="$failures" ;;
    storage) storage_failures="$failures" ;;
  esac
  start_forward "$name"
}

start_forward auth
if [[ "$keycloak_only" == false ]]; then
  start_forward vault
  if [[ "$storage_available" == true ]]; then
    start_forward storage
  fi
fi

echo "Keycloak API/UI: https://localhost:8080"
if [[ "$keycloak_only" == false ]]; then
  echo "Vault API:       https://localhost:8200/v1"
  echo "Vault UI:        https://localhost:8200/ui"
fi
if [[ "$storage_available" == true ]]; then
  echo "RustFS S3/API:   http://localhost:9000"
  echo "RustFS Console:  http://localhost:9001"
fi
echo "Press Ctrl-C to stop the forwards."

while true; do
  check_forward auth
  if [[ "$keycloak_only" == false ]]; then
    check_forward vault
    if [[ "$storage_available" == true ]]; then
      check_forward storage
    fi
  fi
  sleep 1
done
