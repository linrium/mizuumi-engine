#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
trust_local_cas=0
open_browser=0

usage() {
  cat <<'EOF'
Usage: ./scripts/forward.sh [--trust] [--open]

  --trust  Trust the generated local CAs in the macOS user keychain.
  --open   Open the Keycloak, Vault, and RustFS browser UIs.
  -h, --help
           Show this help.

Run with --trust once to remove browser certificate warnings. The trust
setting persists, so subsequent runs only need --open.
EOF
}

while (( $# > 0 )); do
  case "$1" in
    --trust) trust_local_cas=1 ;;
    --open) open_browser=1 ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

if ! command -v kubectl >/dev/null 2>&1; then
  echo "Missing required command: kubectl" >&2
  exit 1
fi

trust_cas() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "--trust currently supports the macOS user keychain only." >&2
    exit 1
  fi
  if ! command -v security >/dev/null 2>&1; then
    echo "Missing required command: security" >&2
    exit 1
  fi

  local login_keychain ca
  login_keychain="$(security default-keychain -d user |
    sed -E 's/^[[:space:]]*"//; s/"[[:space:]]*$//')"
  if [[ -z "$login_keychain" || ! -f "$login_keychain" ]]; then
    echo "Could not find the default macOS user keychain." >&2
    exit 1
  fi

  for ca in \
    "$repo_root/k8s/auth/tls/ca.crt" \
    "$repo_root/k8s/vault/tls/ca.crt" \
    "$repo_root/k8s/storage/tls/ca.crt"; do
    if [[ ! -f "$ca" ]]; then
      echo "Missing local CA: $ca" >&2
      echo "Run ./scripts/bootstrap.sh --no-forward first." >&2
      exit 1
    fi
    security add-trusted-cert -r trustRoot -p ssl -k "$login_keychain" "$ca"
  done
  echo "Trusted the Keycloak, Vault, and RustFS local CAs in $login_keychain."
}

if (( trust_local_cas )); then
  trust_cas
fi

if (( open_browser )) && [[ "$(uname -s)" != "Darwin" ]]; then
  echo "--open currently supports macOS only." >&2
  exit 1
fi

kubectl -n auth get service keycloak >/dev/null
kubectl -n vault get pod vault-0 >/dev/null
kubectl -n rustfs get service rustfs-svc >/dev/null

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
start_forward vault
start_forward storage

echo "Keycloak API/UI: https://localhost:8080"
echo "Vault API:       https://localhost:8200/v1"
echo "Vault UI:        https://vault.localhost:8200/ui/"
echo "RustFS S3/API:   https://localhost:9000"
echo "RustFS Console:  https://localhost:9001"
echo "Press Ctrl-C to stop the forwards."

if (( open_browser )); then
  # Give kubectl a moment to establish its listeners before launching the tabs.
  sleep 1
  open \
    "https://localhost:8080" \
    "https://vault.localhost:8200/ui/" \
    "https://localhost:9001"
fi

while true; do
  check_forward auth
  check_forward vault
  check_forward storage
  sleep 1
done
