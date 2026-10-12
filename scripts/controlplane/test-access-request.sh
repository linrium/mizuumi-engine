#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
api_url="${CONTROLPLANE_URL:-https://api.controlplane.mizuumi.test}"
approval_team="${CONTROLPLANE_TEST_APPROVAL_TEAM:-vietjetGrow}"
target_full_name="${CONTROLPLANE_TEST_TARGET:-mizuumi.demo.orders}"
requested_by="${CONTROLPLANE_TEST_REQUESTED_BY:-test:access-request}"
duration_seconds="${CONTROLPLANE_TEST_DURATION_SECONDS:-3600}"

for command in curl jq; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done

if ! [[ "$duration_seconds" =~ ^[1-9][0-9]*$ ]] || (( duration_seconds % 3600 != 0 )); then
  echo "CONTROLPLANE_TEST_DURATION_SECONDS must be a positive whole number of hours." >&2
  exit 1
fi

api_url="${api_url%/}"
CONTROLPLANE_URL="$api_url" "$repo_root/scripts/controlplane/seed.sh" >/dev/null

payload="$(jq -n \
  --arg requested_by "$requested_by" \
  --arg routed_for_team "$approval_team" \
  --arg target_full_name "$target_full_name" \
  --arg reason "Automated access-request smoke test for $target_full_name" \
  --argjson duration "$duration_seconds" \
  '{
    requested_by: $requested_by,
    requested_for: $requested_by,
    routed_for_team: $routed_for_team,
    target_securable_type: "table",
    target_full_name: $target_full_name,
    requested_privilege: "SELECT",
    reason: $reason,
    requested_duration_seconds: $duration,
    idempotency_key: null
  }')"

created="$(curl --fail-with-body --silent --show-error \
  --request POST \
  --header 'Content-Type: application/json' \
  --data "$payload" \
  "$api_url/api/access-requests")"
request_id="$(jq -er '.id' <<<"$created")"
request_status="$(jq -er '.status' <<<"$created")"

if [[ "$request_status" != "AWAITING_APPROVAL" ]]; then
  echo "Expected AWAITING_APPROVAL, received $request_status." >&2
  exit 1
fi

detail="$(curl --fail-with-body --silent --show-error \
  "$api_url/api/access-requests/$request_id")"
jq -e \
  --arg id "$request_id" \
  --arg reason "Automated access-request smoke test for $target_full_name" \
  --argjson duration "$duration_seconds" \
  '.id == $id
    and .status == "AWAITING_APPROVAL"
    and .reason == $reason
    and .requested_duration_seconds == $duration' \
  >/dev/null <<<"$detail"

queue="$(curl --fail-with-body --silent --show-error --get \
  --data-urlencode 'status=AWAITING_APPROVAL' \
  --data-urlencode "approval_team=$approval_team" \
  "$api_url/api/access-requests")"
jq -e --arg id "$request_id" \
  'any(.access_requests[]; .id == $id)' \
  >/dev/null <<<"$queue"

echo "Access request smoke test passed."
echo "Request: $request_id"
echo "Status: $request_status"
echo "Approval team: $approval_team"
echo "Target: $target_full_name"
