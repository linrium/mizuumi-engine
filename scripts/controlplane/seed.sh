#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
api_url="${CONTROLPLANE_URL:-https://api.controlplane.mizuumi.test}"
fixture="$repo_root/scripts/controlplane/fixtures/vietjet-grow-table-read.json"
template_key="vietjet-grow-table-read"

for command in curl jq; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done

api_url="${api_url%/}"
templates="$(curl --fail-with-body --silent --show-error \
  "$api_url/api/policy-templates")"
template_id="$(jq -r --arg key "$template_key" \
  '.policy_templates[]? | select(.key == $key) | .id' \
  <<<"$templates" | head -n 1)"

if [[ -n "$template_id" ]]; then
  echo "Policy template already seeded: $template_key ($template_id)"
  exit 0
fi

created="$(curl --fail-with-body --silent --show-error \
  --request POST \
  --header 'Content-Type: application/json' \
  --data-binary "@$fixture" \
  "$api_url/api/policy-templates")"
template_id="$(jq -er '.id' <<<"$created")"

echo "Seeded policy template: $template_key ($template_id)"
