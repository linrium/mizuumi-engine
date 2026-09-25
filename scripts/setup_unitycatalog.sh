#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace="${UNITYCATALOG_NAMESPACE:-tower}"
release="${UNITYCATALOG_RELEASE:-unitycatalog}"
credentials_secret="${UNITYCATALOG_CREDENTIALS_SECRET:-unitycatalog-credentials}"
trust_secret="${UNITYCATALOG_TRUST_SECRET:-unitycatalog-trust}"
image_repository="${UNITYCATALOG_IMAGE_REPOSITORY:-mizuumi/unitycatalog-server}"
image_tag="${UNITYCATALOG_IMAGE_TAG:-latest}"
skip_image_build="${UNITYCATALOG_SKIP_IMAGE_BUILD:-0}"
chart="$repo_root/k8s/unitycatalog"
server_dir="$repo_root/packages/unitycatalog/server"
temp_dir=""
vault_pid=""

cleanup() {
  [[ -z "$vault_pid" ]] || { kill "$vault_pid" 2>/dev/null || true; wait "$vault_pid" 2>/dev/null || true; }
  [[ -z "$temp_dir" ]] || rm -rf "$temp_dir"
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage: ./scripts/setup_unitycatalog.sh [helm upgrade options]

Installs or upgrades the Rust Unity Catalog server in the current Kubernetes context.
Additional arguments are forwarded to `helm upgrade --install`, for example:
  ./scripts/setup_unitycatalog.sh --set postgresql.persistence.size=10Gi
  ./scripts/setup_unitycatalog.sh --values /path/to/values.yaml

Environment overrides:
  UNITYCATALOG_NAMESPACE   Kubernetes namespace (default: tower)
  UNITYCATALOG_RELEASE     Helm release name (default: unitycatalog)
  UNITYCATALOG_CREDENTIALS_SECRET  RustFS IAM Secret name (default: unitycatalog-credentials)
  UNITYCATALOG_TRUST_SECRET        CA bundle Secret name (default: unitycatalog-trust)
  UNITYCATALOG_IMAGE_REPOSITORY    Docker image repository (default: mizuumi/unitycatalog-server)
  UNITYCATALOG_IMAGE_TAG           Docker image tag (default: latest)
  UNITYCATALOG_SKIP_IMAGE_BUILD    Set to 1 to skip docker build/load
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

for command in kubectl helm curl jq; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done
if [[ "$skip_image_build" != 1 ]]; then
  command -v docker >/dev/null 2>&1 || { echo "Missing required command: docker" >&2; exit 1; }
fi

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local Unity Catalog values on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

image="$image_repository:$image_tag"
if [[ "$skip_image_build" != 1 ]]; then
  echo "Building Unity Catalog server image: $image"
  docker build -t "$image" "$server_dir"

  case "$context" in
    kind-*)
      if command -v kind >/dev/null 2>&1; then
        cluster="${context#kind-}"
        echo "Loading image into kind cluster: $cluster"
        kind load docker-image "$image" --name "$cluster"
      else
        echo "Kubernetes context is kind but the kind CLI is missing." >&2
        exit 1
      fi
      ;;
    k3d-*)
      if command -v k3d >/dev/null 2>&1; then
        cluster="${context#k3d-}"
        echo "Importing image into k3d cluster: $cluster"
        k3d image import "$image" --cluster "$cluster"
      else
        echo "Kubernetes context is k3d but the k3d CLI is missing." >&2
        exit 1
      fi
      ;;
    minikube|minikube-*)
      if command -v minikube >/dev/null 2>&1; then
        echo "Loading image into minikube."
        minikube image load "$image"
      else
        echo "Kubernetes context is minikube but the minikube CLI is missing." >&2
        exit 1
      fi
      ;;
    docker-desktop|rancher-desktop|orbstack|colima)
      echo "Using Docker-local image for context: $context"
      ;;
    microk8s)
      echo "Built $image; ensure microk8s can access the local Docker image or push it to a registry." >&2
      ;;
    k3s)
      echo "Built $image; ensure k3s can access the local Docker image or push it to a registry." >&2
      ;;
  esac
fi

for required in \
  "$repo_root/k8s/auth/tls/ca.crt" \
  "$repo_root/k8s/storage/tls/ca.crt" \
  "$repo_root/k8s/vault/tls/ca.crt"; do
  [[ -f "$required" ]] || { echo "Missing $required; bootstrap auth, Vault, and storage first." >&2; exit 1; }
done

if ! kubectl -n auth get configmap keycloak-gateway -o jsonpath='{.data.Caddyfile}' 2>/dev/null | grep -q 'unitycatalog.mizuumi.test'; then
  echo "Updating the shared gateway and certificate for unitycatalog.mizuumi.test."
  "$repo_root/scripts/setup_auth.sh"
fi

if ! kubectl get namespace "$namespace" >/dev/null 2>&1; then
  kubectl create namespace "$namespace"
fi

if [[ -n "${VAULT_TOKEN:-}" ]]; then
  vault_token="$VAULT_TOKEN"
elif [[ -f "$repo_root/k8s/vault/init.json" ]]; then
  vault_token="$(jq -er '.root_token' "$repo_root/k8s/vault/init.json")"
else
  echo "Set VAULT_TOKEN or initialize Vault first." >&2
  exit 1
fi

kubectl -n vault port-forward --address 127.0.0.1 pod/vault-0 18200:8200 >/dev/null 2>&1 &
vault_pid=$!
for attempt in {1..60}; do
  if curl -fsS --cacert "$repo_root/k8s/vault/tls/ca.crt" https://127.0.0.1:18200/v1/sys/health >/dev/null 2>&1; then break; fi
  if [[ "$attempt" -eq 60 ]]; then echo "Vault is unavailable or sealed." >&2; exit 1; fi
  sleep 1
done
vault_secret="$(curl -fsS --cacert "$repo_root/k8s/vault/tls/ca.crt" \
  -H "X-Vault-Token: $vault_token" https://127.0.0.1:18200/v1/secret/data/unitycatalog)" || {
    echo "Missing Vault KV secret secret/unitycatalog. Run ./scripts/bootstrap_unitycatalog.sh first." >&2
    exit 1
  }
unset vault_token

temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/unitycatalog-setup.XXXXXX")"
credentials_file="$temp_dir/credentials.env"
jq -r '.data.data | to_entries[] | "\(.key)=\(.value)"' <<<"$vault_secret" > "$credentials_file"
unset vault_secret
kubectl -n "$namespace" create secret generic "$credentials_secret" \
  --from-env-file="$credentials_file" --dry-run=client -o yaml | kubectl apply -f - >/dev/null
kubectl -n "$namespace" create secret generic "$trust_secret" \
  --from-file=keycloak-ca.crt="$repo_root/k8s/auth/tls/ca.crt" \
  --from-file=rustfs-ca.crt="$repo_root/k8s/storage/tls/ca.crt" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

helm upgrade --install "$release" "$chart" \
  --namespace "$namespace" \
  --set-string "server.image.repository=$image_repository" \
  --set-string "server.image.tag=$image_tag" \
  --set-string "server.credentialsSecretName=$credentials_secret" \
  --set-string "server.trustSecretName=$trust_secret" \
  --wait --timeout 10m "$@"

server_deployment="$(kubectl -n "$namespace" get deployment \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}')"
server_service="$(kubectl -n "$namespace" get service \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}')"
if [[ "$skip_image_build" != 1 ]]; then
  kubectl -n "$namespace" rollout restart deployment/"$server_deployment"
fi
kubectl -n "$namespace" rollout status deployment/"$server_deployment" --timeout=5m

"$repo_root/scripts/configure_cluster_dns.sh"

echo "Rust Unity Catalog server is ready."
echo "API: https://unitycatalog.mizuumi.test"
echo "Local fallback: kubectl -n $namespace port-forward service/$server_service 8080:8080"
