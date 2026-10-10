#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${CHRONICLE_UI_NAMESPACE:-chronicle}"
release="${CHRONICLE_UI_RELEASE:-chronicle-ui}"
chart="$repo_root/k8s/chronicle-ui"
package_dir="$repo_root/packages/chronicle-ui"
image_repository="${CHRONICLE_UI_IMAGE_REPOSITORY:-mizuumi/chronicle-ui}"
image_tag="${CHRONICLE_UI_IMAGE_TAG:-latest}"
skip_image_build="${CHRONICLE_UI_SKIP_IMAGE_BUILD:-0}"

usage() {
  cat <<'EOF'
Usage: ./scripts/chronicle-ui/setup.sh [helm upgrade options]

Installs or upgrades the Chronicle UI in the current Kubernetes context and
ensures the shared gateway routes https://ui.chronicle.mizuumi.test to it.
Extra args are forwarded to `helm upgrade --install`.

Environment overrides:
  CHRONICLE_UI_NAMESPACE          Kubernetes namespace (default: chronicle-ui)
  CHRONICLE_UI_RELEASE            Helm release name (default: chronicle-ui)
  CHRONICLE_UI_IMAGE_REPOSITORY   Docker image repository (default: mizuumi/chronicle-ui)
  CHRONICLE_UI_IMAGE_TAG          Docker image tag (default: latest)
  CHRONICLE_UI_SKIP_IMAGE_BUILD   Set to 1 to skip docker build/load
EOF
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

for command in kubectl helm; do
  command -v "$command" >/dev/null 2>&1 || { echo "Missing required command: $command" >&2; exit 1; }
done
if [[ "$skip_image_build" != 1 ]]; then
  command -v docker >/dev/null 2>&1 || { echo "Missing required command: docker" >&2; exit 1; }
fi

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local Chronicle UI values on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

image="$image_repository:$image_tag"
if [[ "$skip_image_build" != 1 ]]; then
  echo "Building Chronicle UI image: $image"
  docker build -t "$image" "$package_dir"

  case "$context" in
    kind-*)
      command -v kind >/dev/null 2>&1 || { echo "Kubernetes context is kind but the kind CLI is missing." >&2; exit 1; }
      kind load docker-image "$image" --name "${context#kind-}"
      ;;
    k3d-*)
      command -v k3d >/dev/null 2>&1 || { echo "Kubernetes context is k3d but the k3d CLI is missing." >&2; exit 1; }
      k3d image import "$image" --cluster "${context#k3d-}"
      ;;
    minikube|minikube-*)
      command -v minikube >/dev/null 2>&1 || { echo "Kubernetes context is minikube but the minikube CLI is missing." >&2; exit 1; }
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

kubectl get namespace "$namespace" >/dev/null 2>&1 || kubectl create namespace "$namespace"

if ! kubectl -n auth get configmap keycloak-gateway -o jsonpath='{.data.Caddyfile}' 2>/dev/null | grep -q 'ui.chronicle.mizuumi.test'; then
  echo "Updating the shared gateway and certificate for ui.chronicle.mizuumi.test."
  "$repo_root/scripts/auth/setup.sh"
fi

helm upgrade --install "$release" "$chart" \
  --namespace "$namespace" \
  --set-string "chronicleUi.image.repository=$image_repository" \
  --set-string "chronicleUi.image.tag=$image_tag" \
  --wait --timeout 10m "$@"

if [[ "$skip_image_build" != 1 ]]; then
  kubectl -n "$namespace" rollout restart deployment/"$release"
fi
kubectl -n "$namespace" rollout status deployment/"$release" --timeout=5m

echo "Chronicle UI is ready."
echo "Public URL: https://ui.chronicle.mizuumi.test"
echo "Service: svc/$release.$namespace (port 80)"
echo "Local port-forward: kubectl -n $namespace port-forward service/$release 3000:80"
