#!/usr/bin/env bash
set -euo pipefail

namespace="${MCP_NAMESPACE:-mcp}"
credentials_secret="${MCP_CREDENTIALS_SECRET:-mcp-credentials}"
duck_namespace="${DUCK_SERVER_NAMESPACE:-duck-server}"
duck_credentials_secret="${DUCK_SERVER_CREDENTIALS_SECRET:-duck-server-credentials}"

for command in kubectl; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local MCP bootstrap on non-local context: $context" >&2; exit 1 ;;
esac

kubectl -n "$duck_namespace" get secret "$duck_credentials_secret" >/dev/null 2>&1 || {
  echo "Missing $duck_namespace/$duck_credentials_secret. Run ./scripts/duck-server/bootstrap.sh first." >&2
  exit 1
}

quack_token="$(kubectl -n "$duck_namespace" get secret "$duck_credentials_secret" \
  -o jsonpath='{.data.QUACK_TOKEN}' | base64 --decode)"
[[ -n "$quack_token" ]] || {
  echo "Missing QUACK_TOKEN in $duck_namespace/$duck_credentials_secret." >&2
  exit 1
}

kubectl get namespace "$namespace" >/dev/null 2>&1 || kubectl create namespace "$namespace"

kubectl -n "$namespace" create secret generic "$credentials_secret" \
  --from-literal=QUACK_TOKEN="$quack_token" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

unset quack_token
echo "MCP bootstrap complete: Quack token mirrored into $namespace/$credentials_secret."
echo "Run ./scripts/mcp/setup.sh."
