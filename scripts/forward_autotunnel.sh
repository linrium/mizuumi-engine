#!/usr/bin/env bash
set -euo pipefail

for command in kubectl jq autotunnel; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

if [[ $# -ne 0 ]]; then
  echo "Usage: $0" >&2
  exit 2
fi

context="$(kubectl config current-context)"
if [[ -z "$context" ]]; then
  echo "No current Kubernetes context is selected." >&2
  exit 1
fi

kubectl -n auth get service keycloak >/dev/null
kubectl -n vault get pod vault-0 >/dev/null
kubectl -n rustfs get service rustfs-svc >/dev/null

if command -v lsof >/dev/null 2>&1 &&
   lsof -nP -iTCP:8989 -sTCP:LISTEN >/dev/null 2>&1; then
  echo "Port 8989 is already in use. If Homebrew autotunnel is running, stop it with:" >&2
  echo "  brew services stop autotunnel" >&2
  echo "Then rerun this script." >&2
  exit 1
fi

# JSON strings are valid YAML scalars, including for context names with punctuation.
context_yaml="$(jq -n --arg value "$context" '$value')"
config_file="$(mktemp "${TMPDIR:-/tmp}/mizuumi-autotunnel.XXXXXX")"
cleanup() {
  rm -f "$config_file"
}
trap cleanup EXIT

cat >"$config_file" <<EOF
apiVersion: autotunnel/v1
auto_reload_config: false
http:
  listen: "127.0.0.1:8989"
  idle_timeout: 60m
  k8s:
    routes:
      auth.localhost:
        context: $context_yaml
        namespace: auth
        service: keycloak
        port: 8080
        scheme: https
      vault.localhost:
        context: $context_yaml
        namespace: vault
        pod: vault-0
        port: 8200
        scheme: https
      storage.localhost:
        context: $context_yaml
        namespace: rustfs
        service: rustfs-svc
        port: 9000
        scheme: https
      storage-console.localhost:
        context: $context_yaml
        namespace: rustfs
        service: rustfs-svc
        port: 9001
        scheme: https
tcp:
  idle_timeout: 60m
  k8s:
    routes:
      8080:
        context: $context_yaml
        namespace: auth
        service: keycloak
        port: 8080
      8200:
        context: $context_yaml
        namespace: vault
        pod: vault-0
        port: 8200
      9000:
        context: $context_yaml
        namespace: rustfs
        service: rustfs-svc
        port: 9000
      9001:
        context: $context_yaml
        namespace: rustfs
        service: rustfs-svc
        port: 9001
EOF

echo "On-demand forwards for Kubernetes context: $context"
echo "Friendly HTTPS routes (port 8989):"
echo "Keycloak API/UI: https://auth.localhost:8989"
echo "Vault API:       https://vault.localhost:8989/v1"
echo "Vault UI:        https://vault.localhost:8989/ui"
echo "RustFS S3/API:   https://storage.localhost:8989"
echo "RustFS Console:  https://storage-console.localhost:8989"
echo "Existing localhost forwards (used by OIDC redirects):"
echo "Keycloak API/UI: https://localhost:8080"
echo "Vault API:       https://localhost:8200/v1"
echo "Vault UI:        https://localhost:8200/ui"
echo "RustFS S3/API:   https://localhost:9000"
echo "RustFS Console:  https://localhost:9001"
echo "For OIDC login, use the localhost URLs until the clients are configured for friendly callbacks."
echo "Press Ctrl-C to stop autotunnel."

autotunnel -config "$config_file"
