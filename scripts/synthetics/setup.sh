#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${SYNTHETICS_NAMESPACE:-synthetics}"
release="${SYNTHETICS_RELEASE:-synthetics}"
chart="$repo_root/k8s/synthetics"
package_dir="$repo_root/packages/synthetics"
image_repository="${SYNTHETICS_IMAGE_REPOSITORY:-mizuumi/synthetics}"
skip_image_build="${SYNTHETICS_SKIP_IMAGE_BUILD:-0}"
redpanda_brokers="${REDPANDA_BROKERS:-redpanda.redpanda.svc.cluster.local:9092}"
topic_vn_hdbank="${SYNTHETICS_TOPIC_VN_HDBANK:-vn.synthetics.hdbank}"
topic_vn_vikki="${SYNTHETICS_TOPIC_VN_VIKKI:-vn.synthetics.vikki}"
topic_vn_vietjetair="${SYNTHETICS_TOPIC_VN_VIETJETAIR:-vn.synthetics.vietjetair}"
topic_th_hdbank="${SYNTHETICS_TOPIC_TH_HDBANK:-th.synthetics.hdbank}"
topic_th_vikki="${SYNTHETICS_TOPIC_TH_VIKKI:-th.synthetics.vikki}"
topic_th_vietjetair="${SYNTHETICS_TOPIC_TH_VIETJETAIR:-th.synthetics.vietjetair}"

if [[ -n "${SYNTHETICS_IMAGE_TAG:-}" ]]; then
  image_tag="$SYNTHETICS_IMAGE_TAG"
elif [[ "$skip_image_build" == 1 ]]; then
  echo "Set SYNTHETICS_IMAGE_TAG when SYNTHETICS_SKIP_IMAGE_BUILD=1." >&2
  exit 2
else
  image_tag="local-$(date +%Y%m%d%H%M%S)"
fi

usage() {
  cat <<'EOF'
Usage: ./scripts/synthetics/setup.sh [helm upgrade options]

Builds and installs the synthetic-data API in the synthetics namespace.
Run ./scripts/synthetics/bootstrap.sh first. Extra arguments are passed to
helm upgrade --install.

Environment overrides:
  SYNTHETICS_NAMESPACE          Kubernetes namespace (default: synthetics)
  SYNTHETICS_RELEASE            Helm release name (default: synthetics)
  SYNTHETICS_IMAGE_REPOSITORY   Image repository (default: mizuumi/synthetics)
  SYNTHETICS_IMAGE_TAG          Image tag (default: unique local build tag)
  SYNTHETICS_SKIP_IMAGE_BUILD   Set to 1 to skip docker build/load
  REDPANDA_BROKERS              Broker list (default: in-cluster Redpanda service)
  SYNTHETICS_TOPIC_VN_HDBANK       Vietnam HDBank topic
  SYNTHETICS_TOPIC_VN_VIKKI        Vietnam Vikki topic
  SYNTHETICS_TOPIC_VN_VIETJETAIR   Vietnam VietJet Air topic
  SYNTHETICS_TOPIC_TH_HDBANK       Thailand HDBank topic
  SYNTHETICS_TOPIC_TH_VIKKI        Thailand Vikki topic
  SYNTHETICS_TOPIC_TH_VIETJETAIR   Thailand VietJet Air topic
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
  *) echo "Refusing local synthetics setup on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

if ! kubectl get namespace "$namespace" >/dev/null 2>&1; then
  echo "Missing namespace '$namespace'. Run ./scripts/synthetics/bootstrap.sh first." >&2
  exit 1
fi

image="$image_repository:$image_tag"
helm_redpanda_brokers="${redpanda_brokers//,/\\,}"
if [[ "$skip_image_build" != 1 ]]; then
  echo "Building synthetics image: $image"
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

helm upgrade --install "$release" "$chart" \
  --namespace "$namespace" \
  --set-string "image.repository=$image_repository" \
  --set-string "image.tag=$image_tag" \
  --set-string "redpanda.brokers=$helm_redpanda_brokers" \
  --set-string "redpanda.topics.vn.hdbank=$topic_vn_hdbank" \
  --set-string "redpanda.topics.vn.vikki=$topic_vn_vikki" \
  --set-string "redpanda.topics.vn.vietjetAir=$topic_vn_vietjetair" \
  --set-string "redpanda.topics.th.hdbank=$topic_th_hdbank" \
  --set-string "redpanda.topics.th.vikki=$topic_th_vikki" \
  --set-string "redpanda.topics.th.vietjetAir=$topic_th_vietjetair" \
  --wait --timeout 5m "$@"

if [[ "$skip_image_build" != 1 ]]; then
  kubectl -n "$namespace" rollout restart deployment/"$release"
fi
kubectl -n "$namespace" rollout status deployment/"$release" --timeout=3m

if ! kubectl -n auth get configmap keycloak-gateway -o jsonpath='{.data.Caddyfile}' 2>/dev/null | grep -q 'api.synthetics.mizuumi.test'; then
  echo "Updating the shared gateway and certificate for api.synthetics.mizuumi.test."
  "$repo_root/scripts/auth/setup.sh"
fi

for attempt in $(seq 1 30); do
  ready_addresses="$(kubectl -n "$namespace" get endpoints "$release" \
    -o jsonpath='{.subsets[0].addresses[*].ip}' 2>/dev/null || true)"
  if [[ -n "$ready_addresses" ]]; then
    echo "Synthetics service endpoint is ready: $ready_addresses"
    break
  fi
  if (( attempt == 30 )); then
    echo "Timed out waiting for service/$release endpoints." >&2
    exit 1
  fi
  sleep 2
done

echo "Synthetics API is ready in namespace '$namespace'."
echo "API: https://api.synthetics.mizuumi.test"
echo "Port-forward: kubectl -n $namespace port-forward service/$release 4020:4020"
echo "Generate: curl -X POST https://api.synthetics.mizuumi.test/api/v1/generate -H 'content-type: application/json' -d '{\"region\":\"vn\",\"count\":10,\"seed\":42}'"
