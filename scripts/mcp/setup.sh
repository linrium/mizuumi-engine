#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${MCP_NAMESPACE:-mcp}"
release="${MCP_RELEASE:-mcp}"
image_repository="${MCP_IMAGE_REPOSITORY:-mizuumi/mcp}"
skip_image_build="${MCP_SKIP_IMAGE_BUILD:-0}"
credentials_secret="${MCP_CREDENTIALS_SECRET:-mcp-credentials}"
trust_secret="${MCP_TRUST_SECRET:-mcp-trust}"
quack_uri="${MCP_QUACK_URI:-quack:query.mizuumi.test:443}"

if [[ -n "${MCP_IMAGE_TAG:-}" ]]; then
  image_tag="$MCP_IMAGE_TAG"
elif [[ "$skip_image_build" == 1 ]]; then
  echo "Set MCP_IMAGE_TAG when MCP_SKIP_IMAGE_BUILD=1." >&2
  exit 2
else
  image_tag="local-$(date +%Y%m%d%H%M%S)"
fi

usage() {
  cat <<'EOF'
Usage: ./scripts/mcp/setup.sh [helm upgrade options]

Builds and installs the Mizuumi MCP server. Extra arguments are passed to
helm upgrade --install.

Environment overrides:
  MCP_NAMESPACE           Kubernetes namespace (default: mcp)
  MCP_RELEASE             Helm release name (default: mcp)
  MCP_IMAGE_REPOSITORY    Image repository (default: mizuumi/mcp)
  MCP_IMAGE_TAG           Image tag (default: unique local build tag)
  MCP_SKIP_IMAGE_BUILD    Set to 1 to skip docker build/load
  MCP_CREDENTIALS_SECRET  Credentials Secret (default: mcp-credentials)
  MCP_TRUST_SECRET        CA Secret (default: mcp-trust)
  MCP_QUACK_URI           Quack URI (default: quack:query.mizuumi.test:443)
EOF
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

for command in kubectl helm; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done
if [[ "$skip_image_build" != 1 ]]; then
  command -v docker >/dev/null || { echo "Missing required command: docker" >&2; exit 1; }
fi

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local MCP setup on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

if ! kubectl -n "$namespace" get secret "$credentials_secret" >/dev/null 2>&1; then
  echo "Missing $namespace/$credentials_secret. Run ./scripts/mcp/bootstrap.sh first." >&2
  exit 1
fi

gateway_ca="$repo_root/k8s/auth/tls/ca.crt"
[[ -f "$gateway_ca" ]] || { echo "Missing gateway CA: $gateway_ca" >&2; exit 1; }

if ! kubectl -n auth get configmap keycloak-gateway -o jsonpath='{.data.Caddyfile}' 2>/dev/null | grep -q 'mcp.mizuumi.test'; then
  echo "Updating the shared gateway and certificate for mcp.mizuumi.test."
  "$repo_root/scripts/auth/setup.sh"
fi

image="$image_repository:$image_tag"
if [[ "$skip_image_build" != 1 ]]; then
  echo "Building MCP image: $image"
  docker build -f "$repo_root/packages/mcp/Dockerfile" -t "$image" \
    "$repo_root/packages"
  case "$context" in
    kind-*)
      command -v kind >/dev/null || { echo "Missing required command: kind" >&2; exit 1; }
      kind load docker-image "$image" --name "${context#kind-}"
      ;;
    k3d-*)
      command -v k3d >/dev/null || { echo "Missing required command: k3d" >&2; exit 1; }
      k3d image import "$image" --cluster "${context#k3d-}"
      ;;
    minikube|minikube-*)
      command -v minikube >/dev/null || { echo "Missing required command: minikube" >&2; exit 1; }
      minikube image load "$image"
      ;;
    docker-desktop|rancher-desktop|orbstack|colima)
      echo "Using Docker-local image for context: $context"
      ;;
    microk8s|k3s)
      echo "Built $image; ensure the cluster can access the local image." >&2
      ;;
  esac
fi

kubectl -n "$namespace" create secret generic "$trust_secret" \
  --from-file=gateway-ca.crt="$gateway_ca" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

helm upgrade --install "$release" "$repo_root/k8s/mcp" \
  --namespace "$namespace" \
  --set-string "image.repository=$image_repository" \
  --set-string "image.tag=$image_tag" \
  --set-string "credentialsSecretName=$credentials_secret" \
  --set-string "trustSecretName=$trust_secret" \
  --set-string "quack.uri=$quack_uri" \
  --wait --timeout 5m "$@"

kubectl -n "$namespace" rollout restart deployment/"$release"
kubectl -n "$namespace" rollout status deployment/"$release" --timeout=3m
"$repo_root/scripts/mcp/test.sh"

echo "Mizuumi MCP server is ready at https://mcp.mizuumi.test/mcp (via the shared gateway)."
