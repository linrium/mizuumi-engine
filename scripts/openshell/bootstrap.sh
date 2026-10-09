#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${OPENSHELL_NAMESPACE:-openshell}"
release="${OPENSHELL_RELEASE:-openshell}"
gateway_name="${OPENSHELL_GATEWAY_NAME:-k8s}"
gateway_endpoint="${OPENSHELL_GATEWAY_ENDPOINT:-https://openshell.mizuumi.test}"
keycloak_ca="${KEYCLOAK_CA:-$repo_root/k8s/auth/tls/ca.crt}"
sandbox_name="${OPENSHELL_SANDBOX_NAME:-minimal-agent}"
sandbox_image="${OPENSHELL_SANDBOX_IMAGE:-nvcr.io/nvidia/base/ubuntu:24.04}"
example_dir="$repo_root/packages/openshell/examples/minimal-agent"
studio_env="$repo_root/packages/studio/.env.local"
run_example=true

studio_env_value() {
  local name="$1"
  [[ -f "$studio_env" ]] || return 0
  awk -v name="$name" 'index($0, name "=") == 1 {sub(/^[^=]*=/, ""); print; exit}' "$studio_env"
}

oidc_issuer="${OPENSHELL_OIDC_ISSUER:-$(studio_env_value OPENSHELL_OIDC_ISSUER)}"
oidc_issuer="${oidc_issuer:-https://auth.mizuumi.test/realms/sovico}"
oidc_client_id="${OPENSHELL_OIDC_CLIENT_ID:-$(studio_env_value OPENSHELL_OIDC_CLIENT_ID)}"
oidc_client_id="${oidc_client_id:-studio}"
oidc_client_secret="${OPENSHELL_OIDC_CLIENT_SECRET:-$(studio_env_value OPENSHELL_OIDC_CLIENT_SECRET)}"
oidc_audience="${OPENSHELL_OIDC_AUDIENCE:-$(studio_env_value OPENSHELL_OIDC_AUDIENCE)}"
oidc_audience="${oidc_audience:-openshell-cli}"

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
  OPENSHELL_OIDC_ISSUER     Keycloak issuer (default: sovico realm)
  OPENSHELL_OIDC_CLIENT_ID  Keycloak service client (default: studio)
  OPENSHELL_OIDC_CLIENT_SECRET
                           Defaults to packages/studio/.env.local
  OPENSHELL_OIDC_AUDIENCE   Default: openshell-cli
  KEYCLOAK_CA               OIDC issuer CA (default: k8s/auth/tls/ca.crt)
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

for command in kubectl openshell openssl jq curl; do
  command -v "$command" >/dev/null || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done
[[ -f "$example_dir/agent.sh" ]] || {
  echo "Missing minimal agent example: $example_dir/agent.sh" >&2
  exit 1
}
[[ -f "$keycloak_ca" ]] || {
  echo "Missing Keycloak CA: $keycloak_ca" >&2
  echo "Run ./scripts/setup_auth.sh or set KEYCLOAK_CA." >&2
  exit 1
}
[[ -n "$oidc_client_secret" ]] || {
  echo "Missing OPENSHELL_OIDC_CLIENT_SECRET." >&2
  echo "Run ./scripts/bootstrap_studio.sh or set it in the environment." >&2
  exit 1
}

openshell_oidc() {
  SSL_CERT_FILE="$keycloak_ca" \
    OPENSHELL_OIDC_CLIENT_SECRET="$oidc_client_secret" \
    openshell "$@"
}

trap 'exit 130' INT
trap 'exit 143' TERM

"$repo_root/scripts/openshell/expose.sh"
kubectl -n "$namespace" get service "$release" >/dev/null
kubectl -n "$namespace" get secret "${release}-client-tls" >/dev/null

for attempt in {1..30}; do
  if curl -fsS --cacert "$keycloak_ca" \
    "$oidc_issuer/.well-known/openid-configuration" 2>/dev/null | \
    jq -e --arg issuer "$oidc_issuer" \
      '.issuer == $issuer and (.token_endpoint | type == "string")' >/dev/null; then
    break
  fi
  if [[ "$attempt" -eq 30 ]]; then
    echo "Keycloak OIDC discovery is unavailable at $oidc_issuer." >&2
    exit 1
  fi
  sleep 1
done

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
   jq -e --arg endpoint "$gateway_endpoint" --arg issuer "$oidc_issuer" \
     --arg client_id "$oidc_client_id" --arg audience "$oidc_audience" \
     '.gateway_endpoint == $endpoint and .is_remote == false and
      .auth_mode == "oidc" and
      .oidc_issuer == $issuer and .oidc_client_id == $client_id and
      (.oidc_audience // .oidc_client_id) == $audience' \
     "$gateway_metadata" >/dev/null; then
  echo "Gateway $gateway_name is already registered at $gateway_endpoint."
else
  if [[ -f "$gateway_metadata" ]]; then
    openshell_oidc gateway remove "$gateway_name"
  fi
  for attempt in {1..3}; do
    openshell_oidc gateway add "$gateway_endpoint" \
      --local \
      --name "$gateway_name" \
      --oidc-issuer "$oidc_issuer" \
      --oidc-client-id "$oidc_client_id" \
      --oidc-audience "$oidc_audience"
    [[ -f "$gateway_metadata" ]] && break
    if [[ "$attempt" -lt 3 ]]; then
      echo "OIDC registration failed; retrying after the gateway rollout..." >&2
      sleep 2
    fi
  done
  [[ -f "$gateway_metadata" ]] || {
    echo "OpenShell rolled back gateway $gateway_name after OIDC authentication failed." >&2
    echo "Verify the Keycloak issuer, service-account secret, audience, and CA." >&2
    exit 1
  }
fi
# OIDC authenticates the caller at the application layer. The shared local
# gateway still requires this client identity for the TLS connection.
decode_secret_key tls.crt "$tls_dir/tls.crt"
decode_secret_key tls.key "$tls_dir/tls.key"
cp "$repo_root/k8s/auth/tls/ca.crt" "$tls_dir/ca.crt"
chmod 600 "$tls_dir/ca.crt" "$tls_dir/tls.crt" "$tls_dir/tls.key"
openshell_oidc gateway select "$gateway_name"
openshell_oidc status

if [[ "$run_example" == true ]]; then
  if openshell_oidc sandbox get "$sandbox_name" >/dev/null 2>&1; then
    echo "Sandbox $sandbox_name already exists; reusing it."
  else
    openshell_oidc sandbox create \
      --name "$sandbox_name" \
      --from "$sandbox_image" \
      --cpu 250m \
      --memory 256Mi \
      --detach \
      -- sh -c 'while :; do sleep 3600; done'
  fi
  openshell_oidc sandbox upload "$sandbox_name" "$example_dir"
  openshell_oidc sandbox exec -n "$sandbox_name" \
    --workdir "/sandbox/$(basename "$example_dir")" \
    -- sh ./agent.sh "verify Kubernetes sandbox isolation"
fi

cat <<EOF
OpenShell Kubernetes bootstrap complete.
The CLI will reconnect through $gateway_endpoint; no port-forward is required.
EOF
