#!/usr/bin/env bash
set -euo pipefail

vault_mount="${VAULT_KV_MOUNT:-pi-custom-openai}"
vault_secret_path="${VAULT_KV_SECRET_PATH:-config}"

: "${VAULT_ADDR:?Set VAULT_ADDR to the HashiCorp Vault server URL}"
: "${CUSTOM_OPENAI_BASE_URL:?Set CUSTOM_OPENAI_BASE_URL to seed Vault}"
: "${CUSTOM_OPENAI_API_KEY:?Set CUSTOM_OPENAI_API_KEY to seed Vault}"
: "${CUSTOM_OPENAI_MODEL:?Set CUSTOM_OPENAI_MODEL to seed Vault}"

if [[ ! "$VAULT_ADDR" =~ ^https?:// ]]; then
  echo "VAULT_ADDR must be an http(s) URL, not a Vault token." >&2
  exit 2
fi

for command in vault jq; do
  command -v "$command" >/dev/null || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done

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

vault status >/dev/null
vault token lookup >/dev/null

mounts_json="$(vault secrets list -format=json)"
if jq -e --arg mount "$vault_mount/" 'has($mount)' <<<"$mounts_json" >/dev/null; then
  if ! jq -e --arg mount "$vault_mount/" \
    '.[$mount].type == "kv" and .[$mount].options.version == "2"' \
    <<<"$mounts_json" >/dev/null; then
    echo "Vault mount '$vault_mount/' exists but is not KV v2." >&2
    exit 1
  fi
  echo "Using existing Vault KV v2 mount: $vault_mount/"
else
  vault secrets enable \
    -path="$vault_mount" \
    -description="Pi custom OpenAI-compatible endpoint" \
    kv-v2
fi

# Feed JSON on stdin so the API key is not placed in a command-line argument.
jq -n '{
  base_url: env.CUSTOM_OPENAI_BASE_URL,
  api_key: env.CUSTOM_OPENAI_API_KEY,
  model: env.CUSTOM_OPENAI_MODEL
}' | vault kv put -mount="$vault_mount" "$vault_secret_path" -

echo "Seeded Vault KV v2 secret: $vault_mount/$vault_secret_path"
