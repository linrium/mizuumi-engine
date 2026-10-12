#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

namespace="${DAGSTER_NAMESPACE:-dagster}"
release="${DAGSTER_RELEASE:-dagster}"
chart_version="${DAGSTER_CHART_VERSION:-1.9.5}"
image_repository="${DAGSTER_USER_CODE_IMAGE_REPOSITORY:-mizuumi/dagster-user-code}"
image_tag="${DAGSTER_USER_CODE_IMAGE_TAG:-0.1.3}"
build_image="${DAGSTER_BUILD_IMAGE:-true}"
spark_namespace="${SPARK_NAMESPACE:-spark}"
spark_application_name="${SPARK_APPLICATION_NAME:-unity-catalog-managed}"

usage() {
  cat <<'EOF'
Usage: ./scripts/dagster/setup.sh [extra helm args]

Builds the Mizuumi Dagster user-code image, installs the upstream Dagster
Helm chart (via the umbrella chart in k8s/dagster/), and configures a schedule
that re-triggers the unity-catalog-managed SparkApplication every 10 minutes.

Environment overrides:
  DAGSTER_NAMESPACE                     Namespace (default: dagster)
  DAGSTER_RELEASE                       Helm release name (default: dagster)
  DAGSTER_CHART_VERSION                 Upstream chart version (default: 1.9.5)
  DAGSTER_USER_CODE_IMAGE_REPOSITORY    User-code image repo
  DAGSTER_USER_CODE_IMAGE_TAG           User-code image tag
  DAGSTER_BUILD_IMAGE                   Build/import the image (default: true)
  SPARK_NAMESPACE                       SparkApplication namespace (default: spark)
  SPARK_APPLICATION_NAME                SparkApplication name
                                        (default: unity-catalog-managed)
EOF
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

for command in kubectl helm; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done
if [[ "$build_image" == true ]]; then
  command -v docker >/dev/null || { echo "Missing required command: docker" >&2; exit 1; }
fi

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local Dagster values on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

if [[ "$build_image" == true ]]; then
  image="$image_repository:$image_tag"
  docker build -t "$image" "$repo_root/packages/dagster"
  case "$context" in
    kind-*) kind load docker-image "$image" --name "${context#kind-}" ;;
    k3d-*) k3d image import "$image" --cluster "${context#k3d-}" ;;
    minikube|minikube-*) minikube image load "$image" ;;
    docker-desktop|rancher-desktop|orbstack|colima) ;;
    *)
      echo "Image built, but $context requires a registry or manual image import." >&2
      echo "Set DAGSTER_BUILD_IMAGE=false after making $image available to the cluster." >&2
      exit 1
      ;;
  esac
fi

kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f - >/dev/null
kubectl create namespace "$spark_namespace" --dry-run=client -o yaml | kubectl apply -f - >/dev/null

helm repo add dagster https://dagster-io.github.io/helm --force-update
helm repo update dagster

helm dependency update "$repo_root/k8s/dagster"

helm upgrade --install "$release" "$repo_root/k8s/dagster" \
  --namespace "$namespace" \
  --values "$repo_root/k8s/dagster/values.yaml" \
  --skip-schema-validation \
  --set "sparkJobNamespace=$spark_namespace" \
  --set "sparkApplicationName=$spark_application_name" \
  --set-string "dagster.dagster-user-deployments.deployments[0].image.repository=$image_repository" \
  --set-string "dagster.dagster-user-deployments.deployments[0].image.tag=$image_tag" \
  --set-string "dagster.dagster-user-deployments.deployments[0].env[0].value=$spark_namespace" \
  --set-string "dagster.dagster-user-deployments.deployments[0].env[1].value=$spark_application_name" \
  --wait --timeout 15m \
  "$@"

echo
kubectl -n "$namespace" get pods
echo
echo "Dagster is installed."
echo "Webserver: kubectl -n $namespace port-forward svc/${release}-dagster-webserver 3000:80"
echo "Schedule:  unity_catalog_managed_every_10_minutes (every */10 * * * *)"
echo "After port-forward, enable the schedule in the UI if auto-enable is off."
