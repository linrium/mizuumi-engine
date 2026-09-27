#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
base_url="${UNITYCATALOG_URL:-https://unitycatalog.mizuumi.test}"
ca="${UNITYCATALOG_CA:-$repo_root/k8s/auth/tls/ca.crt}"
expected_bucket="${UNITYCATALOG_BUCKET:-unitycatalog}"
duration_seconds="${UNITYCATALOG_STS_DURATION_SECONDS:-900}"
auth_token="${UNITYCATALOG_AUTH_TOKEN:-}"

usage() {
  cat <<'EOF'
Usage: ./scripts/validate_unitycatalog_vending.sh

Validates the Rust Unity Catalog vending API through the local gateway by
calling GET /api/vending/buckets and checking that the Unity Catalog bucket is
visible with credentials vended through RustFS STS.

Environment overrides:
  UNITYCATALOG_URL                 API base URL (default: https://unitycatalog.mizuumi.test)
  UNITYCATALOG_CA                  CA bundle path (default: k8s/auth/tls/ca.crt)
  UNITYCATALOG_BUCKET              Expected bucket (default: unitycatalog)
  UNITYCATALOG_STS_DURATION_SECONDS  STS duration query value (default: 900)
  UNITYCATALOG_AUTH_TOKEN          Keycloak or bootstrap bearer token (required)
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
  "")
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

for command in curl jq; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done
[[ -f "$ca" ]] || { echo "Missing CA bundle: $ca" >&2; exit 1; }
[[ -n "$auth_token" ]] || { echo "Set UNITYCATALOG_AUTH_TOKEN." >&2; exit 1; }

response="$(curl --fail --silent --show-error --cacert "$ca" \
  -H "Authorization: Bearer $auth_token" \
  "$base_url/api/vending/buckets?duration_seconds=$duration_seconds")"

if ! jq -e --arg bucket "$expected_bucket" '.buckets | any(.name == $bucket)' <<<"$response" >/dev/null; then
  echo "Vending API responded, but bucket '$expected_bucket' was not present." >&2
  jq . <<<"$response" >&2
  exit 1
fi

bucket_count="$(jq '.buckets | length' <<<"$response")"
echo "Vending API is healthy: $bucket_count bucket(s), including $expected_bucket."
