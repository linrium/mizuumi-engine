#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
console_url=https://console.signoz.mizuumi.test
otlp_url=https://otlp.signoz.mizuumi.test
ca="$repo_root/k8s/auth/tls/ca.crt"

for command in curl; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

if [[ ! -f "$ca" ]]; then
  echo "Missing local gateway CA: $ca" >&2
  echo "Run ./scripts/auth/setup.sh first." >&2
  exit 1
fi

for attempt in $(seq 1 30); do
  if response="$(curl -fsS --cacert "$ca" "$console_url/api/v1/health" 2>/dev/null)"; then
    break
  fi
  if (( attempt == 30 )); then
    echo "Timed out waiting for $console_url." >&2
    echo "Run: sudo ./scripts/configure_workstation_dns.sh" >&2
    echo "Then apply the gateway with: ./scripts/auth/setup.sh" >&2
    exit 1
  fi
  sleep 1
done

if ! curl -fsS --cacert "$ca" \
  -H 'Content-Type: application/x-protobuf' \
  --data-binary '' \
  "$otlp_url/v1/traces" >/dev/null; then
  echo "The console is healthy, but OTLP ingestion is not ready." >&2
  echo "Complete the initial user and organization setup at $console_url, then retry." >&2
  exit 1
fi

echo "SigNoz console health check passed: $response"
echo "SigNoz OTLP/HTTP check passed: $otlp_url"
