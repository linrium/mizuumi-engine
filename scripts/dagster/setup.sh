#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

namespace="${DAGSTER_NAMESPACE:-dagster}"
release="${DAGSTER_RELEASE:-dagster}"
chart_version="${DAGSTER_CHART_VERSION:-1.9.5}"
image_repository="${DAGSTER_USER_CODE_IMAGE_REPOSITORY:-mizuumi/dagster-user-code}"
image_tag="${DAGSTER_USER_CODE_IMAGE_TAG:-0.3.3}"
build_image="${DAGSTER_BUILD_IMAGE:-true}"
spark_namespace="${SPARK_NAMESPACE:-spark}"
spark_app_name="${SPARK_APP_NAME:-unity-catalog-managed}"
spark_script_source="${SPARK_SCRIPT_SOURCE:-$repo_root/packages/spark/examples/managed/main.py}"
spark_script_filename="${SPARK_SCRIPT_FILENAME:-main.py}"
spark_script_configmap="${SPARK_SCRIPT_CONFIGMAP:-managed-orders-spark-script}"

usage() {
  cat <<'EOF'
Usage: ./scripts/dagster/setup.sh [extra helm args]

Builds the Mizuumi Dagster user-code image, installs the upstream Dagster
Helm chart (via the umbrella chart in k8s/dagster/), uploads the managed-orders
Spark script as a ConfigMap in the Spark namespace, and provisions RBAC for
Dagster to launch spark-submit Jobs that materialize the Unity Catalog
managed table every 10 minutes.

Environment overrides:
  DAGSTER_NAMESPACE                     Namespace (default: dagster)
  DAGSTER_RELEASE                       Helm release name (default: dagster)
  DAGSTER_CHART_VERSION                 Upstream chart version (default: 1.9.5)
  DAGSTER_USER_CODE_IMAGE_REPOSITORY    User-code image repo
  DAGSTER_USER_CODE_IMAGE_TAG           User-code image tag
  DAGSTER_BUILD_IMAGE                   Build/import the image (default: true)
  SPARK_NAMESPACE                       Spark namespace (default: spark)
  SPARK_APP_NAME                        spark-submit --name value
                                        (default: unity-catalog-managed)
  SPARK_SCRIPT_SOURCE                   Local path to the Spark app main.py
                                        (default: packages/spark/examples/managed/main.py)
  SPARK_SCRIPT_FILENAME                 Filename inside the ConfigMap/pod
                                        (default: main.py)
  SPARK_SCRIPT_CONFIGMAP                ConfigMap name
                                        (default: managed-orders-spark-script)
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
[[ -f "$spark_script_source" ]] || {
  echo "Missing Spark application script: $spark_script_source" >&2
  exit 1
}

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

# Publish the Spark job's script as a ConfigMap that the driver pod mounts
# via `spark.kubernetes.driver.volumes.configMap.*`.
kubectl -n "$spark_namespace" create configmap "$spark_script_configmap" \
  --from-file="$spark_script_filename=$spark_script_source" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

helm repo add dagster https://dagster-io.github.io/helm --force-update
helm repo update dagster

helm dependency update "$repo_root/k8s/dagster"

helm upgrade --install "$release" "$repo_root/k8s/dagster" \
  --namespace "$namespace" \
  --values "$repo_root/k8s/dagster/values.yaml" \
  --skip-schema-validation \
  --set "sparkJobNamespace=$spark_namespace" \
  --set "sparkApplicationName=$spark_app_name" \
  --set-string "dagster.dagster-user-deployments.deployments[0].image.repository=$image_repository" \
  --set-string "dagster.dagster-user-deployments.deployments[0].image.tag=$image_tag" \
  --wait --timeout 15m \
  "$@"

echo
kubectl -n "$namespace" get pods
echo
echo "Dagster is installed."
echo "Console: https://console.dagster.mizuumi.test"
echo "GraphQL: https://api.dagster.mizuumi.test/graphql"
echo "Asset:   unity/demo/orders  (group: unity_catalog)"
echo "Schedule: unity_catalog_managed_orders_every_10_minutes"
echo "Watch:   kubectl -n $spark_namespace get pods -l 'spark-role in (driver,executor)' -w"
