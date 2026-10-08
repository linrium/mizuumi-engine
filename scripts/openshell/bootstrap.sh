#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${OPENSHELL_NAMESPACE:-openshell}"
release="${OPENSHELL_RELEASE:-openshell}"
gateway_name="${OPENSHELL_GATEWAY_NAME:-k8s}"
local_port="${OPENSHELL_LOCAL_PORT:-8080}"
sandbox_name="${OPENSHELL_SANDBOX_NAME:-minimal-agent}"
sandbox_image="${OPENSHELL_SANDBOX_IMAGE:-nvcr.io/nvidia/base/ubuntu:24.04}"
example_dir="$repo_root/packages/openshell/examples/minimal-agent"
run_example=true
port_forward_pid=""

usage() {
  cat <<'EOF'
Usage: ./scripts/openshell/bootstrap.sh [--skip-example]

Registers the Kubernetes gateway with the local OpenShell CLI and runs the
dependency-free minimal sandbox example.

Environment overrides:
  OPENSHELL_NAMESPACE      Namespace (default: openshell)
  OPENSHELL_RELEASE        Service/release name (default: openshell)
  OPENSHELL_GATEWAY_NAME   Local CLI gateway name (default: k8s)
  OPENSHELL_LOCAL_PORT     Local port-forward port (default: 8080)
  OPENSHELL_SANDBOX_NAME   Example sandbox name (default: minimal-agent)
  OPENSHELL_SANDBOX_IMAGE  Example image (default: NVIDIA Ubuntu 24.04 base)
EOF
}

case "${1:-}" in
  "") ;;
  --skip-example) run_example=false ;;
  -h|--help)
    usage
    exit 0
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
[[ $# -le 1 ]] || { usage >&2; exit 2; }

for command in kubectl openshell openssl jq; do
  command -v "$command" >/dev/null || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done
[[ -f "$example_dir/agent.sh" ]] || {
  echo "Missing minimal agent example: $example_dir/agent.sh" >&2
  exit 1
}

cleanup() {
  if [[ -n "$port_forward_pid" ]]; then
    kill "$port_forward_pid" 2>/dev/null || true
    wait "$port_forward_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

kubectl -n "$namespace" get service "$release" >/dev/null
kubectl -n "$namespace" get secret "${release}-client-tls" >/dev/null

config_home="${XDG_CONFIG_HOME:-${HOME}/.config}"
tls_dir="$config_home/openshell/gateways/$gateway_name/mtls"
umask 077
mkdir -p "$tls_dir"

decode_secret_key() {
  local key="$1" destination="$2"
  kubectl -n "$namespace" get secret "${release}-client-tls" \
    -o "jsonpath={.data.${key//./\\.}}" | openssl base64 -d -A > "$destination"
  [[ -s "$destination" ]] || {
    echo "Secret ${release}-client-tls has no $key data." >&2
    exit 1
  }
}

decode_secret_key ca.crt "$tls_dir/ca.crt"
decode_secret_key tls.crt "$tls_dir/tls.crt"
decode_secret_key tls.key "$tls_dir/tls.key"
chmod 600 "$tls_dir/ca.crt" "$tls_dir/tls.crt" "$tls_dir/tls.key"

kubectl -n "$namespace" port-forward \
  "service/$release" "$local_port:8080" >/dev/null 2>&1 &
port_forward_pid=$!
for attempt in {1..30}; do
  if ! kill -0 "$port_forward_pid" 2>/dev/null; then
    echo "OpenShell port-forward exited before the gateway became reachable." >&2
    exit 1
  fi
  tls_probe="$(openssl s_client -connect "127.0.0.1:$local_port" \
    -servername localhost \
    -CAfile "$tls_dir/ca.crt" -cert "$tls_dir/tls.crt" \
    -key "$tls_dir/tls.key" </dev/null 2>&1 || true)"
  if [[ "$tls_probe" == *"Verify return code: 0 (ok)"* ]]; then
    break
  fi
  if [[ "$attempt" -eq 30 ]]; then
    echo "Timed out waiting for the OpenShell gateway port-forward." >&2
    exit 1
  fi
  sleep 1
done

gateway_endpoint="https://127.0.0.1:$local_port"
gateway_metadata="$config_home/openshell/gateways/$gateway_name/metadata.json"
if [[ -f "$gateway_metadata" ]] &&
   jq -e --arg endpoint "$gateway_endpoint" \
     '.gateway_endpoint == $endpoint and .auth_mode == "mtls"' \
     "$gateway_metadata" >/dev/null; then
  echo "Gateway $gateway_name is already registered at $gateway_endpoint."
else
  if [[ -f "$gateway_metadata" ]]; then
    openshell gateway remove "$gateway_name"
  fi
  openshell gateway add "$gateway_endpoint" --local --name "$gateway_name"
fi
# OpenShell 0.1.2 imports the package-managed local gateway bundle during
# `gateway add --local`, even when this named gateway already has its own bundle.
# Restore the Kubernetes-issued material after registration.
decode_secret_key ca.crt "$tls_dir/ca.crt"
decode_secret_key tls.crt "$tls_dir/tls.crt"
decode_secret_key tls.key "$tls_dir/tls.key"
chmod 600 "$tls_dir/ca.crt" "$tls_dir/tls.crt" "$tls_dir/tls.key"
openshell gateway select "$gateway_name"
openshell status

if [[ "$run_example" == true ]]; then
  if openshell sandbox get "$sandbox_name" >/dev/null 2>&1; then
    echo "Sandbox $sandbox_name already exists; reusing it."
  else
    openshell sandbox create \
      --name "$sandbox_name" \
      --from "$sandbox_image" \
      --cpu 250m \
      --memory 256Mi \
      --detach \
      -- sh -c 'while :; do sleep 3600; done'
  fi
  openshell sandbox upload "$sandbox_name" "$example_dir"
  openshell sandbox exec -n "$sandbox_name" \
    --workdir "/sandbox/$(basename "$example_dir")" \
    -- sh ./agent.sh "verify Kubernetes sandbox isolation"
fi

cat <<EOF
OpenShell Kubernetes bootstrap complete.
The temporary port-forward has stopped. Reconnect with:
  kubectl -n $namespace port-forward service/$release $local_port:8080
EOF
