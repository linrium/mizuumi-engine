#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${DUCK_SERVER_NAMESPACE:-duck-server}"
release="${DUCK_SERVER_RELEASE:-duck-server}"
image_repository="${DUCK_SERVER_IMAGE_REPOSITORY:-mizuumi/duck-server}"
skip_image_build="${DUCK_SERVER_SKIP_IMAGE_BUILD:-0}"
credentials_secret="${DUCK_SERVER_CREDENTIALS_SECRET:-duck-server-credentials}"
trust_secret="${DUCK_SERVER_TRUST_SECRET:-duck-server-trust}"
uc_catalog="${DUCK_SERVER_UNITY_CATALOG:-unity}"
uc_schema="${DUCK_SERVER_UNITY_SCHEMA:-demo}"
if [[ -n "${DUCK_SERVER_IMAGE_TAG:-}" ]]; then
  image_tag="$DUCK_SERVER_IMAGE_TAG"
elif [[ "$skip_image_build" == 1 ]]; then
  echo "Set DUCK_SERVER_IMAGE_TAG when DUCK_SERVER_SKIP_IMAGE_BUILD=1." >&2
  exit 2
else
  image_tag="local-$(date +%Y%m%d%H%M%S)"
fi

usage() {
  cat <<'EOF'
Usage: ./scripts/duck-server/setup.sh [helm upgrade options]

Builds and installs the DuckDB Quack server. Extra arguments are passed to
helm upgrade --install.

Environment overrides:
  DUCK_SERVER_NAMESPACE         Kubernetes namespace (default: duck-server)
  DUCK_SERVER_RELEASE           Helm release name (default: duck-server)
  DUCK_SERVER_IMAGE_REPOSITORY  Image repository (default: mizuumi/duck-server)
  DUCK_SERVER_IMAGE_TAG         Image tag (default: unique local build tag)
  DUCK_SERVER_SKIP_IMAGE_BUILD  Set to 1 to skip docker build/load
  DUCK_SERVER_CREDENTIALS_SECRET
                                 Credentials Secret (default: duck-server-credentials)
  DUCK_SERVER_TRUST_SECRET      CA Secret (default: duck-server-trust)
  DUCK_SERVER_UNITY_CATALOG     Unity Catalog name (default: unity)
  DUCK_SERVER_UNITY_SCHEMA      Unity Catalog schema (default: demo)
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
  *) echo "Refusing local Duck server values on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

if ! kubectl -n "$namespace" get secret "$credentials_secret" >/dev/null 2>&1; then
  echo "Missing $namespace/$credentials_secret. Run ./scripts/duck-server/bootstrap.sh first." >&2
  exit 1
fi
SPARK_UNITY_CATALOG="$uc_catalog" SPARK_UNITY_SCHEMAS="$uc_schema" \
  "$repo_root/scripts/spark/init-catalog.sh"
for ca in "$repo_root/k8s/storage/tls/ca.crt" "$repo_root/k8s/auth/tls/ca.crt"; do
  [[ -f "$ca" ]] || { echo "Missing local CA: $ca" >&2; exit 1; }
done

if ! kubectl -n auth get configmap keycloak-gateway -o jsonpath='{.data.Caddyfile}' 2>/dev/null | grep -q 'query.mizuumi.test'; then
  echo "Updating the shared gateway and certificate for query.mizuumi.test."
  "$repo_root/scripts/auth/setup.sh"
fi

image="$image_repository:$image_tag"
if [[ "$skip_image_build" != 1 ]]; then
  echo "Building Duck server image: $image"
  docker build -f "$repo_root/packages/duck-server/Dockerfile" -t "$image" \
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
  --from-file=rustfs-ca.crt="$repo_root/k8s/storage/tls/ca.crt" \
  --from-file=gateway-ca.crt="$repo_root/k8s/auth/tls/ca.crt" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

helm upgrade --install "$release" "$repo_root/k8s/duck-server" \
  --namespace "$namespace" \
  --set-string "image.repository=$image_repository" \
  --set-string "image.tag=$image_tag" \
  --set-string "credentialsSecretName=$credentials_secret" \
  --set-string "trustSecretName=$trust_secret" \
  --set-string "unityCatalog.catalog=$uc_catalog" \
  --set-string "unityCatalog.defaultSchema=$uc_schema" \
  --wait --timeout 10m "$@"

if [[ "$skip_image_build" != 1 ]]; then
  kubectl -n "$namespace" rollout restart deployment/"$release"
fi
kubectl -n "$namespace" rollout status deployment/"$release" --timeout=5m
"$repo_root/scripts/duck-server/test.sh"

echo "DuckDB Quack is ready at quack:query.mizuumi.test:443 (HTTPS via the shared gateway)."
