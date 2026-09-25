#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
trust_local_ca=0
open_browser=0

usage() {
  cat <<'EOF'
Usage: ./scripts/forward.sh [--trust] [--open]

  --trust  Trust the generated local gateway CA on macOS.
  --open   Open the Keycloak, Vault, and RustFS browser UIs.
  --force, --force-stop
            Accepted for compatibility; no local forwarding process remains.
  -h, --help
            Show this help.

The in-cluster gateway is exposed directly by a LoadBalancer Service, so this
script only verifies access, optionally trusts the CA, and opens the UIs.
Before the first run, configure the workstation DNS entry with:
  sudo ./scripts/configure_workstation_dns.sh
EOF
}

while (( $# > 0 )); do
  case "$1" in
    --trust) trust_local_ca=1 ;;
    --open) open_browser=1 ;;
    --force|--force-stop)
      echo "$1 is no longer needed; the gateway has no local forwarding process." >&2
      ;;
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

for command in kubectl curl; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

public_hosts=(auth.mizuumi.test vault.mizuumi.test storage.mizuumi.test api.storage.mizuumi.test)
for hostname in "${public_hosts[@]}"; do
  if ! awk -v hostname="$hostname" '$1 == "127.0.0.1" { for (i = 2; i <= NF; i++) if ($i == hostname) found = 1 } END { exit !found }' /etc/hosts; then
    echo "$hostname is not mapped to 127.0.0.1." >&2
    echo "Run: sudo ./scripts/configure_workstation_dns.sh" >&2
    exit 1
  fi
done

if (( trust_local_ca )); then
  if [[ "$(uname -s)" != Darwin ]]; then
    echo "--trust currently supports the macOS user keychain only." >&2
    exit 1
  fi
  command -v security >/dev/null || { echo "Missing required command: security" >&2; exit 1; }
  login_keychain="$(security default-keychain -d user | sed -E 's/^[[:space:]]*"//; s/"[[:space:]]*$//')"
  ca="$repo_root/k8s/auth/tls/ca.crt"
  if [[ -z "$login_keychain" || ! -f "$login_keychain" ]]; then
    echo "Could not find the default macOS user keychain." >&2
    exit 1
  fi
  if [[ ! -f "$ca" ]]; then
    echo "Missing local gateway CA: $ca" >&2
    echo "Run ./scripts/setup_auth.sh first." >&2
    exit 1
  fi
  security add-trusted-cert -r trustRoot -p ssl -k "$login_keychain" "$ca"
  echo "Trusted the local gateway CA in $login_keychain."
fi

kubectl -n auth get service keycloak-gateway >/dev/null
external_address="$(kubectl -n auth get service keycloak-gateway -o jsonpath='{.status.loadBalancer.ingress[0].ip}{.status.loadBalancer.ingress[0].hostname}')"
if [[ -z "$external_address" ]]; then
  echo "The keycloak-gateway LoadBalancer has no external address." >&2
  echo "Enable your local cluster's LoadBalancer integration (for example, minikube tunnel)." >&2
  exit 1
fi

for attempt in {1..30}; do
  if curl --fail --silent --show-error --cacert "$repo_root/k8s/auth/tls/ca.crt" \
    https://auth.mizuumi.test/realms/sovico/.well-known/openid-configuration >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 30 ]]; then
    echo "The local gateway is not reachable at https://auth.mizuumi.test." >&2
    exit 1
  fi
  sleep 1
done

echo "Gateway:          $external_address"
echo "Keycloak API/UI: https://auth.mizuumi.test"
echo "Vault API:       https://vault.mizuumi.test/v1"
echo "Vault UI:        https://vault.mizuumi.test/ui/"
echo "RustFS S3/API:   https://api.storage.mizuumi.test"
echo "RustFS Console:  https://storage.mizuumi.test"

if (( open_browser )); then
  if [[ "$(uname -s)" != Darwin ]]; then
    echo "--open currently supports macOS only." >&2
    exit 1
  fi
  open \
    "https://auth.mizuumi.test" \
    "https://vault.mizuumi.test/ui/" \
    "https://storage.mizuumi.test"
fi
