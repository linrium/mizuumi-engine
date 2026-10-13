#!/usr/bin/env bash
set -euo pipefail

api_url="${SYNTHETICS_API_URL:-https://api.synthetics.mizuumi.test/api/v1/generate}"
stream_id="${SYNTHETICS_STREAM_ID:-single-user-vn}"
region="${SYNTHETICS_REGION:-vn}"
seed="${SYNTHETICS_SEED:-42}"
scenario="${SYNTHETICS_SCENARIO:-realistic}"
requested_source="${SYNTHETICS_SOURCE:-auto}"
interval_seconds="${SYNTHETICS_INTERVAL_SECONDS:-60}"
watch=false

usage() {
  cat <<'EOF'
Usage: ./scripts/synthetics/generate-one-user.sh [--watch]

Generates or advances one persistent synthetic user. Each successful iteration sends one source
projection for the same canonical person and never adds another person.

Environment variables:
  SYNTHETICS_API_URL           API endpoint
  SYNTHETICS_STREAM_ID         Persistent stream ID (default: single-user-vn)
  SYNTHETICS_REGION            vn or th (default: vn)
  SYNTHETICS_SEED              Non-negative integer (default: 42)
  SYNTHETICS_SCENARIO          clean, realistic, matcher-demo, or stress
  SYNTHETICS_SOURCE            auto, hdbank, vikkiBank, vietjetAir, or crm
  SYNTHETICS_INTERVAL_SECONDS  Delay between --watch iterations (default: 60)
  SYNTHETICS_INSECURE          true to pass --insecure to curl

Without --watch the script performs one successful generation and exits, which is suitable for
cron or a Kubernetes CronJob. With --watch it runs continuously and rotates sources in auto mode.
EOF
}

case "${1:-}" in
  "") ;;
  --watch) watch=true ;;
  -h|--help) usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac

command -v curl >/dev/null 2>&1 || { echo "Missing required command: curl" >&2; exit 1; }

case "$region" in vn|th) ;; *) echo "SYNTHETICS_REGION must be vn or th" >&2; exit 2 ;; esac
case "$scenario" in clean|realistic|matcher-demo|stress) ;; *) echo "Invalid SYNTHETICS_SCENARIO" >&2; exit 2 ;; esac
case "$requested_source" in auto|hdbank|vikkiBank|vietjetAir|crm) ;; *) echo "Invalid SYNTHETICS_SOURCE" >&2; exit 2 ;; esac
[[ "$stream_id" =~ ^[a-zA-Z0-9._-]+$ ]] || { echo "Invalid SYNTHETICS_STREAM_ID" >&2; exit 2; }
[[ "$seed" =~ ^[0-9]+$ ]] || { echo "SYNTHETICS_SEED must be a non-negative integer" >&2; exit 2; }
[[ "$interval_seconds" =~ ^[0-9]+$ ]] || { echo "SYNTHETICS_INTERVAL_SECONDS must be a positive integer" >&2; exit 2; }
(( interval_seconds > 0 )) || { echo "SYNTHETICS_INTERVAL_SECONDS must be greater than zero" >&2; exit 2; }

curl_args=(--silent --show-error --request POST --header "content-type: application/json")
if [[ "${SYNTHETICS_INSECURE:-false}" == "true" ]]; then
  curl_args+=(--insecure)
fi

response_file="$(mktemp "${TMPDIR:-/tmp}/mizuumi-one-user.XXXXXX")"
trap 'rm -f "$response_file"' EXIT

stream_known=false
iteration=0
sources=(hdbank vikkiBank vietjetAir crm)

choose_source() {
  if [[ "$requested_source" != "auto" ]]; then
    printf '%s' "$requested_source"
    return
  fi
  local epoch_day
  epoch_day="$(( $(date -u +%s) / 86400 ))"
  printf '%s' "${sources[$(( (epoch_day + iteration) % ${#sources[@]} ))]}"
}

post_payload() {
  local payload="$1"
  curl "${curl_args[@]}" \
    --output "$response_file" \
    --write-out '%{http_code}' \
    --data "$payload" \
    "$api_url"
}

run_once() {
  local as_of source payload status
  as_of="$(date -u +'%Y-%m-%dT%H:%M:%S.000Z')"
  source="$(choose_source)"

  if [[ "$stream_known" == "false" ]]; then
    payload="$(printf '{"mode":"incremental","operation":"initialize","streamId":"%s","region":"%s","asOf":"%s","initialPopulation":1,"newPeoplePerDay":0,"seed":%s,"scenario":"%s","sourceMode":"all","sources":["%s"],"maxEvents":100}' \
      "$stream_id" "$region" "$as_of" "$seed" "$scenario" "$source")"
    status="$(post_payload "$payload")"
    if [[ "$status" == "200" ]]; then
      stream_known=true
      echo "[$as_of] initialized stream=$stream_id source=$source"
      cat "$response_file"
      echo
      return
    fi
    if [[ "$status" != "409" ]]; then
      echo "Initialization failed with HTTP $status" >&2
      cat "$response_file" >&2
      echo >&2
      return 1
    fi
    stream_known=true
  fi

  payload="$(printf '{"mode":"incremental","operation":"advance","streamId":"%s","asOf":"%s","newPeople":0,"emitCurrentRecord":true,"sources":["%s"],"maxEvents":100}' \
    "$stream_id" "$as_of" "$source")"
  status="$(post_payload "$payload")"
  if [[ "$status" != "200" ]]; then
    echo "Advance failed with HTTP $status" >&2
    cat "$response_file" >&2
    echo >&2
    return 1
  fi
  echo "[$as_of] advanced stream=$stream_id source=$source"
  cat "$response_file"
  echo
}

while true; do
  run_once
  iteration=$((iteration + 1))
  if [[ "$watch" != "true" ]]; then
    break
  fi
  sleep "$interval_seconds"
done
