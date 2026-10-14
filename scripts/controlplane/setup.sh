#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${CONTROLPLANE_NAMESPACE:-tower}"
release="${CONTROLPLANE_RELEASE:-controlplane}"
image_repository="${CONTROLPLANE_IMAGE_REPOSITORY:-mizuumi/controlplane-server}"
default_image_tag="dev-$(date -u +%Y%m%d%H%M%S)"
image_tag="${CONTROLPLANE_IMAGE_TAG:-$default_image_tag}"
skip_image_build="${CONTROLPLANE_SKIP_IMAGE_BUILD:-0}"
unitycatalog_credentials_secret="${UNITYCATALOG_CREDENTIALS_SECRET:-unitycatalog-credentials}"
trust_secret="${CONTROLPLANE_TRUST_SECRET:-controlplane-trust}"
signoz_api_key_secret="${CONTROLPLANE_SIGNOZ_SECRET:-controlplane-signoz}"
blast_radius_secret="${CONTROLPLANE_BLAST_RADIUS_SECRET:-controlplane-blast-radius}"
vault_kv_mount="${VAULT_KV_MOUNT:-pi-custom-openai}"
vault_kv_secret_path="${VAULT_KV_SECRET_PATH:-config}"
blast_radius_enabled=false
chart="$repo_root/k8s/controlplane"
server_dir="$repo_root/packages/controlplane/server"

usage() {
  cat <<'EOF'
Usage: ./scripts/controlplane/setup.sh [helm upgrade options]

Builds and installs or upgrades the control plane in the current local Kubernetes context.
Additional arguments are forwarded to `helm upgrade --install`.

Environment overrides:
  CONTROLPLANE_NAMESPACE          Kubernetes namespace (default: tower)
  CONTROLPLANE_RELEASE            Helm release name (default: controlplane)
  CONTROLPLANE_IMAGE_REPOSITORY   Image repository (default: mizuumi/controlplane-server)
  CONTROLPLANE_IMAGE_TAG          Image tag (default: unique UTC dev timestamp)
  CONTROLPLANE_SKIP_IMAGE_BUILD   Set to 1 to skip docker build/load
  UNITYCATALOG_CREDENTIALS_SECRET Secret containing CONTROLPLANE_CLIENT_ID and
                                  CONTROLPLANE_CLIENT_SECRET (default: unitycatalog-credentials)
  CONTROLPLANE_TRUST_SECRET       Secret containing the Keycloak CA (default: controlplane-trust)
  CONTROLPLANE_SIGNOZ_SECRET      Secret containing a read-only SigNoz API key
                                  under api-key (default: controlplane-signoz)
  CONTROLPLANE_BLAST_RADIUS_SECRET Kubernetes secret populated from the Pi KV
                                  (default: controlplane-blast-radius)
  VAULT_KV_MOUNT                  OpenAI configuration KV mount
                                  (default: pi-custom-openai)
  VAULT_KV_SECRET_PATH            OpenAI configuration KV path (default: config)
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

for command in kubectl helm jq; do
  command -v "$command" >/dev/null 2>&1 || { echo "Missing required command: $command" >&2; exit 1; }
done

if [[ ! "$vault_kv_mount" =~ ^[A-Za-z0-9_-]+$ ]]; then
  echo "VAULT_KV_MOUNT must contain only letters, numbers, underscores, or hyphens." >&2
  exit 2
fi
if [[ ! "$vault_kv_secret_path" =~ ^[A-Za-z0-9._/-]+$ ]] ||
   [[ "$vault_kv_secret_path" == /* ]] ||
   [[ "/$vault_kv_secret_path/" == *"/../"* ]]; then
  echo "VAULT_KV_SECRET_PATH must be a relative Vault path without '..' segments." >&2
  exit 2
fi
if [[ "$skip_image_build" != 1 ]]; then
  command -v docker >/dev/null 2>&1 || { echo "Missing required command: docker" >&2; exit 1; }
fi

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local control plane values on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

image="$image_repository:$image_tag"
if [[ "$skip_image_build" != 1 ]]; then
  echo "Building control plane server image: $image"
  docker build -t "$image" "$server_dir"

  case "$context" in
    kind-*)
      command -v kind >/dev/null 2>&1 || { echo "Missing required command: kind" >&2; exit 1; }
      kind load docker-image "$image" --name "${context#kind-}"
      ;;
    k3d-*)
      command -v k3d >/dev/null 2>&1 || { echo "Missing required command: k3d" >&2; exit 1; }
      k3d image import "$image" --cluster "${context#k3d-}"
      ;;
    minikube|minikube-*)
      command -v minikube >/dev/null 2>&1 || { echo "Missing required command: minikube" >&2; exit 1; }
      minikube image load "$image"
      ;;
    docker-desktop|rancher-desktop|orbstack|colima)
      echo "Using Docker-local image for context: $context"
      ;;
    microk8s|k3s)
      echo "Ensure $context can access $image before the rollout starts." >&2
      ;;
  esac
fi

kubectl get namespace "$namespace" >/dev/null 2>&1 || kubectl create namespace "$namespace"

for key in CONTROLPLANE_CLIENT_ID CONTROLPLANE_CLIENT_SECRET; do
  if [[ -z "$(kubectl -n "$namespace" get secret "$unitycatalog_credentials_secret" \
    -o "jsonpath={.data.$key}" 2>/dev/null)" ]]; then
    echo "Missing $key in $namespace/$unitycatalog_credentials_secret. Rerun scripts/unitycatalog/bootstrap.sh and scripts/unitycatalog/setup.sh first." >&2
    exit 1
  fi
done

keycloak_ca="$repo_root/k8s/auth/tls/ca.crt"
[[ -f "$keycloak_ca" ]] || {
  echo "Missing $keycloak_ca. Run scripts/auth/setup.sh first." >&2
  exit 1
}
kubectl -n "$namespace" create secret generic "$trust_secret" \
  --from-file=keycloak-ca.crt="$keycloak_ca" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

if command -v vault >/dev/null 2>&1 && vault token lookup >/dev/null 2>&1; then
  if vault_config="$(vault kv get -mount="$vault_kv_mount" -format=json "$vault_kv_secret_path" 2>/dev/null)" &&
     jq -e '.data.data | (.api_key | type == "string" and length > 0) and (.base_url | type == "string" and length > 0) and (.model | type == "string" and length > 0)' <<<"$vault_config" >/dev/null; then
    jq \
      --arg namespace "$namespace" \
      --arg name "$blast_radius_secret" \
      '{apiVersion:"v1",kind:"Secret",metadata:{namespace:$namespace,name:$name},type:"Opaque",stringData:{"api-key":.data.data.api_key,"base-url":.data.data.base_url,model:.data.data.model}}' \
      <<<"$vault_config" |
      kubectl apply -f - >/dev/null
    blast_radius_enabled=true
    unset vault_config
  else
    echo "Warning: Vault KV $vault_kv_mount/$vault_kv_secret_path is unavailable or incomplete; LLM blast-radius evaluation is disabled." >&2
  fi
else
  echo "Warning: Vault is not authenticated; LLM blast-radius evaluation is disabled." >&2
fi

if ! kubectl -n "$namespace" get secret "$signoz_api_key_secret" >/dev/null 2>&1; then
  echo "Warning: $namespace/$signoz_api_key_secret was not found; /api/duck-server/metrics will return 503." >&2
fi

helm upgrade --install "$release" "$chart" \
  --namespace "$namespace" \
  --set-string "server.image.repository=$image_repository" \
  --set-string "server.image.tag=$image_tag" \
  --set-string "unityCatalog.credentialsSecretName=$unitycatalog_credentials_secret" \
  --set-string "unityCatalog.trustSecretName=$trust_secret" \
  --set-string "signoz.apiKeySecretName=$signoz_api_key_secret" \
  --set-string "blastRadius.credentialsSecretName=$blast_radius_secret" \
  --set "blastRadius.enabled=$blast_radius_enabled" \
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

if ! kubectl -n auth get configmap keycloak-gateway -o jsonpath='{.data.Caddyfile}' 2>/dev/null | grep -q 'api.controlplane.mizuumi.test'; then
  echo "Updating the shared gateway and certificate for api.controlplane.mizuumi.test."
  "$repo_root/scripts/auth/setup.sh"
fi

echo "Control plane is ready."
echo "API: https://api.controlplane.mizuumi.test"
echo "Local access: kubectl -n $namespace port-forward service/$server_service 8080:8080"

