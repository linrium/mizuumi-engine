#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${CONTROLPLANE_NAMESPACE:-tower}"
release="${CONTROLPLANE_RELEASE:-controlplane}"
local_port="${CONTROLPLANE_LOCAL_PORT:-18081}"
port_forward_pid=""

cleanup() {
  [[ -z "$port_forward_pid" ]] || { kill "$port_forward_pid" 2>/dev/null || true; wait "$port_forward_pid" 2>/dev/null || true; }
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

command -v curl >/dev/null 2>&1 || { echo "Missing required command: curl" >&2; exit 1; }

if [[ -z "${SIGNOZ_API_KEY:-}" ]]; then
  export SIGNOZ_API_KEY_FILE="${SIGNOZ_API_KEY_FILE:-$repo_root/packages/controlplane/signoz-api-key}"
fi
"$repo_root/scripts/controlplane/bootstrap-signoz.sh"
"$repo_root/scripts/controlplane/setup.sh" "$@"

service_name="$(kubectl -n "$namespace" get service \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}')"
kubectl -n "$namespace" port-forward --address 127.0.0.1 service/"$service_name" "$local_port:8080" >/dev/null 2>&1 &
port_forward_pid=$!

for attempt in {1..30}; do
  if curl --fail --silent --show-error "http://127.0.0.1:$local_port/health/livez" >/dev/null &&
     curl --fail --silent --show-error "http://127.0.0.1:$local_port/health/readyz" >/dev/null; then
    CONTROLPLANE_URL="http://127.0.0.1:$local_port" \
      "$repo_root/scripts/controlplane/seed.sh"
    echo "Control plane bootstrap complete; liveness and readiness checks passed."
    exit 0
  fi
  if [[ "$attempt" -eq 30 ]]; then
    echo "Control plane health checks did not become ready." >&2
    exit 1
  fi
  sleep 1
done

