#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${OPENSHELL_NAMESPACE:-openshell}"
release="${OPENSHELL_RELEASE:-openshell}"
gateway_name="${OPENSHELL_GATEWAY_NAME:-k8s}"
gateway_endpoint="${OPENSHELL_GATEWAY_ENDPOINT:-https://openshell.mizuumi.test}"
sandbox_name="${OPENSHELL_SANDBOX_NAME:-minimal-agent}"
sandbox_image="${OPENSHELL_SANDBOX_IMAGE:-nvcr.io/nvidia/base/ubuntu:24.04}"
example_dir="$repo_root/packages/openshell/examples/minimal-agent"
run_example=true

usage() {
  cat <<'EOF'
Usage: ./scripts/openshell/bootstrap.sh [--skip-example]

Registers the Kubernetes gateway with the local OpenShell CLI and runs the
dependency-free minimal sandbox example.

Environment overrides:
  OPENSHELL_NAMESPACE      Namespace (default: openshell)
  OPENSHELL_RELEASE        Service/release name (default: openshell)
  OPENSHELL_GATEWAY_NAME   Local CLI gateway name (default: k8s)
  OPENSHELL_GATEWAY_ENDPOINT
                           Gateway URL (default: https://openshell.mizuumi.test)
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

trap 'exit 130' INT
trap 'exit 143' TERM

"$repo_root/scripts/openshell/expose.sh"
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

decode_secret_key tls.crt "$tls_dir/tls.crt"
decode_secret_key tls.key "$tls_dir/tls.key"
cp "$repo_root/k8s/auth/tls/ca.crt" "$tls_dir/ca.crt"
chmod 600 "$tls_dir/ca.crt" "$tls_dir/tls.crt" "$tls_dir/tls.key"

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
# Restore the client identity and shared-gateway CA after registration.
decode_secret_key tls.crt "$tls_dir/tls.crt"
decode_secret_key tls.key "$tls_dir/tls.key"
cp "$repo_root/k8s/auth/tls/ca.crt" "$tls_dir/ca.crt"
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
The CLI will reconnect through $gateway_endpoint; no port-forward is required.
EOF
