#!/usr/bin/env bash
set -euo pipefail

example_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
vault_mount="${VAULT_KV_MOUNT:-pi-custom-openai}"
vault_secret_path="${VAULT_KV_SECRET_PATH:-config}"

: "${VAULT_ADDR:?Set VAULT_ADDR to the HashiCorp Vault server URL}"

if [[ ! "$VAULT_ADDR" =~ ^https?:// ]]; then
  echo "VAULT_ADDR must be an http(s) URL, not a Vault token." >&2
  exit 2
fi

if [[ ! "$vault_mount" =~ ^[A-Za-z0-9_-]+$ ]]; then
  echo "VAULT_KV_MOUNT must contain only letters, numbers, underscores, or hyphens." >&2
  exit 2
fi

if [[ ! "$vault_secret_path" =~ ^[A-Za-z0-9._/-]+$ ]] ||
   [[ "$vault_secret_path" == /* ]] ||
   [[ "/$vault_secret_path/" == *"/../"* ]]; then
  echo "VAULT_KV_SECRET_PATH must be a relative Vault path without '..' segments." >&2
  exit 2
fi

profile_name="${CUSTOM_OPENAI_PROFILE:-pi-custom-openai}"
provider_name="${CUSTOM_OPENAI_PROVIDER:-pi-custom-openai}"
sandbox_name="${CUSTOM_OPENAI_SANDBOX:-pi-custom-openai}"
image="${CUSTOM_OPENAI_IMAGE:-pi-custom-openai:local}"
container_engine="${CONTAINER_ENGINE:-docker}"
skip_build="${CUSTOM_OPENAI_SKIP_BUILD:-false}"

required_commands=(vault openshell)
if [[ "$skip_build" != true ]]; then
  required_commands+=("$container_engine")
fi

for command in "${required_commands[@]}"; do
  command -v "$command" >/dev/null || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done

vault status >/dev/null
vault token lookup >/dev/null

if ! vault_base_url="$(vault kv get -mount="$vault_mount" -field=base_url "$vault_secret_path")" ||
   ! vault_api_key="$(vault kv get -mount="$vault_mount" -field=api_key "$vault_secret_path")" ||
   ! vault_model="$(vault kv get -mount="$vault_mount" -field=model "$vault_secret_path")"; then
  echo "Could not read the Pi configuration from Vault. Run ./init-vault-kv.sh first." >&2
  exit 1
fi

unset CUSTOM_OPENAI_BASE_URL CUSTOM_OPENAI_API_KEY CUSTOM_OPENAI_MODEL
CUSTOM_OPENAI_BASE_URL="$vault_base_url"
CUSTOM_OPENAI_API_KEY="$vault_api_key"
CUSTOM_OPENAI_MODEL="$vault_model"

if ! openshell status >/dev/null; then
  cat >&2 <<'EOF'
Cannot reach the selected OpenShell gateway. Start or reconnect the gateway,
then retry.
EOF
  exit 1
fi

if [[ ! "$profile_name" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
  echo "CUSTOM_OPENAI_PROFILE must use lowercase kebab-case." >&2
  exit 2
fi

if [[ "$CUSTOM_OPENAI_BASE_URL" =~ ^(https?)://([^/:]+)(:([0-9]+))?(/[^?#]*)?$ ]]; then
  scheme="${BASH_REMATCH[1]}"
  endpoint_host="${BASH_REMATCH[2]}"
  endpoint_port="${BASH_REMATCH[4]}"
  endpoint_path="${BASH_REMATCH[5]}"
else
  echo "CUSTOM_OPENAI_BASE_URL must be an http(s) URL without query or fragment." >&2
  exit 2
fi

if [[ ! "$endpoint_host" =~ ^[A-Za-z0-9.-]+$ ]] ||
   [[ -n "$endpoint_path" && ! "$endpoint_path" =~ ^/[-A-Za-z0-9._~/%]*$ ]]; then
  echo "CUSTOM_OPENAI_BASE_URL contains unsupported host or path characters." >&2
  exit 2
fi

if [[ -z "$endpoint_port" ]]; then
  if [[ "$scheme" == https ]]; then
    endpoint_port=443
  else
    endpoint_port=80
  fi
fi

endpoint_path="${endpoint_path%/}"
if [[ -z "$endpoint_path" ]]; then
  endpoint_path="/**"
else
  endpoint_path="$endpoint_path/**"
fi

if openshell profile describe "$profile_name" >/dev/null 2>&1; then
  cat >&2 <<EOF
OpenShell profile '$profile_name' already exists. To avoid silently reusing a
profile bound to another endpoint, delete it or choose a unique name with
CUSTOM_OPENAI_PROFILE.
EOF
  exit 1
fi

if openshell provider get "$provider_name" >/dev/null 2>&1; then
  echo "OpenShell provider '$provider_name' already exists; choose another CUSTOM_OPENAI_PROVIDER." >&2
  exit 1
fi

if openshell sandbox get "$sandbox_name" >/dev/null 2>&1; then
  echo "OpenShell sandbox '$sandbox_name' already exists; choose another CUSTOM_OPENAI_SANDBOX." >&2
  exit 1
fi

profile_file="$(mktemp "${TMPDIR:-/tmp}/pi-custom-openai-profile.XXXXXX.yaml")"
cleanup() {
  rm -f "$profile_file"
}
trap cleanup EXIT

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

if [[ "$skip_build" != true ]]; then
  "$container_engine" build -t "$image" -f "$example_dir/Dockerfile.pi" "$example_dir"
fi

openshell profile lint -f "$profile_file"
openshell profile import -f "$profile_file"

OPENAI_API_KEY="$CUSTOM_OPENAI_API_KEY" \
  openshell provider create \
    --name "$provider_name" \
    --type "$profile_name" \
    --from-existing

openshell sandbox create \
  --name "$sandbox_name" \
  --from "$image" \
  --provider "$provider_name" \
  --env "CUSTOM_OPENAI_BASE_URL=$CUSTOM_OPENAI_BASE_URL" \
  --env "CUSTOM_OPENAI_MODEL=$CUSTOM_OPENAI_MODEL" \
  --env "PI_CODING_AGENT_DIR=/tmp/pi-agent-runtime" \
  --label "mizuumi.studio.agent=true" \
  -- pi-custom-openai
