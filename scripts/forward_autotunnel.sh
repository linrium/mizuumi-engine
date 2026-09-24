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
  # autotunnel requires an HTTP listener even when only TCP routes are used.
  listen: "127.0.0.1:0"
  idle_timeout: 60m
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
echo "Keycloak API/UI: https://localhost:8080"
echo "Vault API:       https://localhost:8200/v1"
echo "Vault UI:        https://localhost:8200/ui"
echo "RustFS S3/API:   http://localhost:9000"
echo "RustFS Console:  http://localhost:9001"
echo "Press Ctrl-C to stop autotunnel."

autotunnel -config "$config_file"
