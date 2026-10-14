#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${CONTROLPLANE_NAMESPACE:-tower}"
secret_name="${CONTROLPLANE_SIGNOZ_SECRET:-controlplane-signoz}"
secret_key="${CONTROLPLANE_SIGNOZ_SECRET_KEY:-api-key}"
api_key="${SIGNOZ_API_KEY:-}"
api_key_file="${SIGNOZ_API_KEY_FILE:-$repo_root/packages/controlplane/signoz-api-key}"

usage() {
  cat <<'EOF'
Usage: ./scripts/controlplane/bootstrap-signoz.sh

Creates or updates the Kubernetes Secret used by the control plane to query
SigNoz. Generate a key for a read-only SigNoz service account first, then pass
the one-time value in packages/controlplane/signoz-api-key, or pass it through
SIGNOZ_API_KEY_FILE or SIGNOZ_API_KEY.

Environment overrides:
  SIGNOZ_API_KEY_FILE              File containing the SigNoz API key
                                   (default: packages/controlplane/signoz-api-key)
  SIGNOZ_API_KEY                   SigNoz API key value
  CONTROLPLANE_NAMESPACE           Kubernetes namespace (default: tower)
  CONTROLPLANE_SIGNOZ_SECRET       Secret name (default: controlplane-signoz)
  CONTROLPLANE_SIGNOZ_SECRET_KEY   Secret data key (default: api-key)

If the default file is absent but the Secret already contains a non-empty key,
the script leaves the Secret unchanged.
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
  "") ;;
  *) usage >&2; exit 2 ;;
esac

for command in kubectl base64 jq; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local SigNoz secret bootstrap on non-local context: $context" >&2; exit 1 ;;
esac

if [[ -n "$api_key" && -n "${SIGNOZ_API_KEY_FILE:-}" ]]; then
  echo "Set only one of SIGNOZ_API_KEY_FILE or SIGNOZ_API_KEY." >&2
  exit 2
fi
if [[ -z "$api_key" && -f "$api_key_file" ]]; then
  api_key="$(<"$api_key_file")"
fi

if [[ -z "${api_key//[[:space:]]/}" ]]; then
  existing_key="$(kubectl -n "$namespace" get secret "$secret_name" -o json 2>/dev/null \
    | jq -r --arg key "$secret_key" '.data[$key] // empty' || true)"
  if [[ -n "$existing_key" ]] && decoded_key="$(printf '%s' "$existing_key" | base64 --decode 2>/dev/null)" \
    && [[ -n "${decoded_key//[[:space:]]/}" ]]; then
    unset decoded_key existing_key
    echo "SigNoz API key Secret already exists: $namespace/$secret_name"
    exit 0
  fi
  unset existing_key
  echo "Missing a SigNoz API key and $namespace/$secret_name does not contain one." >&2
  echo "Create a read-only SigNoz service-account key at $api_key_file," >&2
  echo "or set SIGNOZ_API_KEY_FILE or SIGNOZ_API_KEY." >&2
  exit 1
fi

secret_dir="$(mktemp -d "${TMPDIR:-/tmp}/controlplane-signoz.XXXXXX")"
cleanup() {
  rm -rf "$secret_dir"
}
trap cleanup EXIT
umask 077
printf '%s' "$api_key" > "$secret_dir/api-key"
unset api_key

kubectl get namespace "$namespace" >/dev/null 2>&1 || kubectl create namespace "$namespace"
kubectl -n "$namespace" create secret generic "$secret_name" \
  --from-file="$secret_key=$secret_dir/api-key" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

echo "SigNoz API key Secret is ready: $namespace/$secret_name"
