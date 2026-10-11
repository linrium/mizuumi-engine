#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
example_dir="$repo_root/packages/openshell/examples/pi-custom-openai"

profile_name="${CUSTOM_OPENAI_PROFILE:-pi-custom-openai}"
provider_name="${CUSTOM_OPENAI_PROVIDER:-pi-custom-openai}"
image="${CUSTOM_OPENAI_IMAGE:-pi-custom-openai:local-1}"
container_engine="${CONTAINER_ENGINE:-docker}"
skip_build="${CUSTOM_OPENAI_SKIP_BUILD:-false}"
force_recreate="${CUSTOM_OPENAI_RECREATE:-false}"

mcp_profile_name="${MIZUUMI_MCP_PROFILE:-mizuumi-mcp}"
mcp_provider_name="${MIZUUMI_MCP_PROVIDER:-mizuumi-mcp}"
mcp_url="${MIZUUMI_MCP_URL:-http://mcp.mcp.svc.cluster.local:8080/mcp}"

vault_mount="${VAULT_KV_MOUNT:-pi-custom-openai}"
vault_secret_path="${VAULT_KV_SECRET_PATH:-config}"
keycloak_ca="${KEYCLOAK_CA:-$repo_root/k8s/auth/tls/ca.crt}"
studio_env="$repo_root/packages/studio/.env.local"
gateway_name="${OPENSHELL_GATEWAY_NAME:-k8s}"

studio_env_value() {
  local name="$1"
  [[ -f "$studio_env" ]] || return 0
  awk -v name="$name" 'index($0, name "=") == 1 {sub(/^[^=]*=/, ""); print; exit}' "$studio_env"
}

oidc_client_secret="${OPENSHELL_OIDC_CLIENT_SECRET:-$(studio_env_value OPENSHELL_OIDC_CLIENT_SECRET)}"

openshell_oidc() {
  SSL_CERT_FILE="$keycloak_ca" \
    OPENSHELL_OIDC_CLIENT_SECRET="$oidc_client_secret" \
    openshell "$@"
}

usage() {
  cat <<'EOF'
Usage: ./scripts/openshell/init-pi-provider.sh [--skip-build] [--recreate]

Idempotently prepares the pi-custom-openai OpenShell profile, provider, and
container image so Studio can spawn pi agents from /chat. Reads the inference
endpoint and API key from Vault (mount: pi-custom-openai, path: config).

Options:
  --skip-build   Reuse the existing CUSTOM_OPENAI_IMAGE; do not rebuild it.
  --recreate     Delete and recreate the OpenShell profile and provider.

Environment overrides:
  CUSTOM_OPENAI_PROFILE   OpenShell profile id   (default: pi-custom-openai)
  CUSTOM_OPENAI_PROVIDER  OpenShell provider name (default: pi-custom-openai)
  CUSTOM_OPENAI_IMAGE     Container image tag    (default: pi-custom-openai:local-1)
  CONTAINER_ENGINE        Build tool             (default: docker)
  VAULT_KV_MOUNT          Vault KV mount         (default: pi-custom-openai)
  VAULT_KV_SECRET_PATH    Vault secret path      (default: config)
  KEYCLOAK_CA             OIDC issuer CA         (default: k8s/auth/tls/ca.crt)
EOF
}

while (( $# > 0 )); do
  case "$1" in
    --skip-build) skip_build=true ;;
    --recreate) force_recreate=true ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
  esac
  shift
done

for command in vault openshell jq; do
  command -v "$command" >/dev/null || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done
if [[ "$skip_build" != true ]]; then
  command -v "$container_engine" >/dev/null || {
    echo "Missing required command: $container_engine (set CONTAINER_ENGINE or pass --skip-build)" >&2
    exit 1
  }
fi

[[ -f "$example_dir/Dockerfile.pi" ]] || {
  echo "Missing example directory: $example_dir" >&2
  exit 1
}

: "${VAULT_ADDR:?Set VAULT_ADDR to the HashiCorp Vault server URL}"
export VAULT_CACERT="${VAULT_CACERT:-$keycloak_ca}"

if [[ ! "$profile_name" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
  echo "CUSTOM_OPENAI_PROFILE must use lowercase kebab-case." >&2
  exit 2
fi
if [[ ! "$vault_mount" =~ ^[A-Za-z0-9_-]+$ ]]; then
  echo "VAULT_KV_MOUNT must contain only letters, numbers, underscores, or hyphens." >&2
  exit 2
fi

vault status >/dev/null
vault token lookup >/dev/null || {
  echo "The Vault CLI is not authenticated. Run 'vault login' first." >&2
  exit 1
}

if ! vault_base_url="$(vault kv get -mount="$vault_mount" -field=base_url "$vault_secret_path")" ||
   ! vault_api_key="$(vault kv get -mount="$vault_mount" -field=api_key "$vault_secret_path")"; then
  echo "Could not read $vault_mount/$vault_secret_path from Vault." >&2
  echo "Seed it with: $example_dir/init-vault-kv.sh" >&2
  exit 1
fi

if [[ "$vault_base_url" =~ ^(https?)://([^/:]+)(:([0-9]+))?(/[^?#]*)?$ ]]; then
  scheme="${BASH_REMATCH[1]}"
  endpoint_host="${BASH_REMATCH[2]}"
  endpoint_port="${BASH_REMATCH[4]}"
  endpoint_path="${BASH_REMATCH[5]}"
else
  echo "base_url in Vault must be an http(s) URL without query or fragment." >&2
  exit 2
fi
if [[ -z "$endpoint_port" ]]; then
  [[ "$scheme" == https ]] && endpoint_port=443 || endpoint_port=80
fi
endpoint_path="${endpoint_path%/}"
[[ -n "$endpoint_path" ]] || endpoint_path=""
endpoint_path="${endpoint_path}/**"

[[ -n "$oidc_client_secret" ]] || {
  echo "Missing OPENSHELL_OIDC_CLIENT_SECRET." >&2
  echo "Run ./scripts/studio/bootstrap.sh or export it in the environment." >&2
  exit 1
}

openshell_oidc gateway login "$gateway_name" >/dev/null
openshell_oidc status >/dev/null || {
  echo "Cannot reach the selected OpenShell gateway. Run ./scripts/openshell/bootstrap.sh first." >&2
  exit 1
}

if [[ "$skip_build" != true ]]; then
  echo "Building $image via $container_engine..."
  "$container_engine" build -t "$image" -f "$example_dir/Dockerfile.pi" "$example_dir"
else
  echo "Skipping image build; expecting $image to already exist."
fi

profile_file="$(mktemp "${TMPDIR:-/tmp}/pi-custom-openai-profile.XXXXXX.yaml")"
trap 'rm -f "$profile_file"' EXIT

cat >"$profile_file" <<EOF
id: $profile_name
display_name: Pi custom OpenAI-compatible API
description: Endpoint-bound OpenAI-compatible inference access for Pi
category: inference
inference_capable: true
credentials:
  - name: api_key
    description: OpenAI-compatible API key
    env_vars:
      - OPENAI_API_KEY
    required: true
    auth_style: bearer
    header_name: authorization
discovery:
  credentials:
    - api_key
endpoints:
  - host: $endpoint_host
    port: $endpoint_port
    path: $endpoint_path
    protocol: rest
    access: read-write
    enforcement: enforce
binaries:
  - /usr/local/bin/node
EOF

if [[ "$force_recreate" == true ]]; then
  if openshell_oidc provider get "$provider_name" >/dev/null 2>&1; then
    echo "Deleting existing provider $provider_name..."
    openshell_oidc provider delete "$provider_name" --yes >/dev/null 2>&1 \
      || openshell_oidc provider delete "$provider_name" >/dev/null
  fi
  if openshell_oidc profile describe "$profile_name" >/dev/null 2>&1; then
    echo "Deleting existing profile $profile_name..."
    openshell_oidc profile delete "$profile_name" --yes >/dev/null 2>&1 \
      || openshell_oidc profile delete "$profile_name" >/dev/null
  fi
fi

if openshell_oidc profile describe "$profile_name" >/dev/null 2>&1; then
  echo "Profile $profile_name exists; reusing it."
else
  openshell_oidc profile lint -f "$profile_file"
  openshell_oidc profile import -f "$profile_file"
fi

if openshell_oidc provider get "$provider_name" >/dev/null 2>&1; then
  echo "Provider $provider_name exists; reusing it."
else
  SSL_CERT_FILE="$keycloak_ca" \
    OPENSHELL_OIDC_CLIENT_SECRET="$oidc_client_secret" \
    OPENAI_API_KEY="$vault_api_key" \
    openshell provider create \
      --name "$provider_name" \
      --type "$profile_name" \
      --from-existing
fi

if [[ "$mcp_url" =~ ^(https?)://([^/:]+)(:([0-9]+))?(/[^?#]*)?$ ]]; then
  mcp_scheme="${BASH_REMATCH[1]}"
  mcp_host="${BASH_REMATCH[2]}"
  mcp_port="${BASH_REMATCH[4]}"
  mcp_path="${BASH_REMATCH[5]}"
else
  echo "MIZUUMI_MCP_URL must be an http(s) URL without query or fragment." >&2
  exit 2
fi
if [[ -z "$mcp_port" ]]; then
  [[ "$mcp_scheme" == https ]] && mcp_port=443 || mcp_port=80
fi
# Allow both the exact base path and anything under the MCP host. Using "/**"
# avoids the "/mcp" vs "/mcp/**" mismatch that trips openshell's L7 policy.
mcp_path="/**"

mcp_profile_file="$(mktemp "${TMPDIR:-/tmp}/mizuumi-mcp-profile.XXXXXX.yaml")"
trap 'rm -f "$profile_file" "$mcp_profile_file"' EXIT

cat >"$mcp_profile_file" <<EOF
id: $mcp_profile_name
display_name: Mizuumi MCP
description: Streamable-HTTP egress to the Mizuumi MCP endpoint for pi agents
category: data
inference_capable: false
endpoints:
  - host: $mcp_host
    port: $mcp_port
    path: $mcp_path
    protocol: rest
    access: read-write
    enforcement: enforce
binaries:
  - /usr/local/bin/node
EOF

if [[ "$force_recreate" == true ]]; then
  if openshell_oidc provider get "$mcp_provider_name" >/dev/null 2>&1; then
    openshell_oidc provider delete "$mcp_provider_name" --yes >/dev/null 2>&1 \
      || openshell_oidc provider delete "$mcp_provider_name" >/dev/null
  fi
  if openshell_oidc profile describe "$mcp_profile_name" >/dev/null 2>&1; then
    openshell_oidc profile delete "$mcp_profile_name" --yes >/dev/null 2>&1 \
      || openshell_oidc profile delete "$mcp_profile_name" >/dev/null
  fi
fi

if openshell_oidc profile describe "$mcp_profile_name" >/dev/null 2>&1; then
  echo "Profile $mcp_profile_name exists; reusing it."
else
  openshell_oidc profile lint -f "$mcp_profile_file"
  openshell_oidc profile import -f "$mcp_profile_file"
fi

if openshell_oidc provider get "$mcp_provider_name" >/dev/null 2>&1; then
  echo "Provider $mcp_provider_name exists; reusing it."
else
  openshell_oidc provider create \
    --name "$mcp_provider_name" \
    --type "$mcp_profile_name"
fi

cat <<EOF
Pi + MCP providers are ready.
  pi profile/provider:  $profile_name / $provider_name
  mcp profile/provider: $mcp_profile_name / $mcp_provider_name
  pi image:             $image
  pi endpoint:          $vault_base_url
  mcp endpoint:         $mcp_url

Studio needs both providers attached to each sandbox. Set:
  OPENSHELL_PI_PROVIDER=$provider_name
  OPENSHELL_MCP_PROVIDER=$mcp_provider_name
EOF
