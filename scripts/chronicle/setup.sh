#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${CHRONICLE_NAMESPACE:-chronicle}"
release="${CHRONICLE_RELEASE:-chronicle}"
chart="$repo_root/k8s/chronicle"
package_dir="$repo_root/packages/chronicle"
image_repository="${CHRONICLE_IMAGE_REPOSITORY:-mizuumi/chronicle}"
image_tag="${CHRONICLE_IMAGE_TAG:-latest}"
skip_image_build="${CHRONICLE_SKIP_IMAGE_BUILD:-0}"
credentials_secret="${CHRONICLE_CREDENTIALS_SECRET:-chronicle-credentials}"
signer_secret="${CHRONICLE_SIGNER_SECRET:-chronicle-signer}"
demo_keys_secret="${CHRONICLE_WITNESS_DEMO_KEYS_SECRET:-chronicle-witness-demo-keys}"
trust_secret="${CHRONICLE_TRUST_SECRET:-chronicle-trust}"
vault_pid=""
temp_dir=""

cleanup() {
  [[ -z "$vault_pid" ]] || { kill "$vault_pid" 2>/dev/null || true; wait "$vault_pid" 2>/dev/null || true; }
  [[ -z "$temp_dir" ]] || rm -rf "$temp_dir"
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage: ./scripts/chronicle/setup.sh [helm upgrade options]

Installs or upgrades Chronicle in the current Kubernetes context. Extra args
are forwarded to `helm upgrade --install`.

Environment overrides:
  CHRONICLE_NAMESPACE          Kubernetes namespace (default: chronicle)
  CHRONICLE_RELEASE            Helm release name (default: chronicle)
  CHRONICLE_IMAGE_REPOSITORY   Docker image repository (default: mizuumi/chronicle)
  CHRONICLE_IMAGE_TAG          Docker image tag (default: latest)
  CHRONICLE_SKIP_IMAGE_BUILD   Set to 1 to skip docker build/load
EOF
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

for command in kubectl helm curl jq openssl; do
  command -v "$command" >/dev/null 2>&1 || { echo "Missing required command: $command" >&2; exit 1; }
done
if [[ "$skip_image_build" != 1 ]]; then
  command -v docker >/dev/null 2>&1 || { echo "Missing required command: docker" >&2; exit 1; }
fi

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local Chronicle values on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

image="$image_repository:$image_tag"
if [[ "$skip_image_build" != 1 ]]; then
  echo "Building Chronicle image: $image"
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

for required in \
  "$repo_root/k8s/vault/tls/ca.crt" \
  "$repo_root/k8s/storage/tls/ca.crt"; do
  [[ -f "$required" ]] || { echo "Missing $required; bootstrap Vault and storage first." >&2; exit 1; }
done

kubectl get namespace "$namespace" >/dev/null 2>&1 || kubectl create namespace "$namespace"

if ! kubectl -n auth get configmap keycloak-gateway -o jsonpath='{.data.Caddyfile}' 2>/dev/null | grep -q 'api.chronicle.mizuumi.test'; then
  echo "Updating the shared gateway and certificate for api.chronicle.mizuumi.test."
  "$repo_root/scripts/auth/setup.sh"
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
  if curl -fsS --cacert "$repo_root/k8s/vault/tls/ca.crt" https://127.0.0.1:18200/v1/sys/health >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then echo "Vault is unavailable or sealed." >&2; exit 1; fi
  sleep 1
done

vault_secret="$(curl -fsS --cacert "$repo_root/k8s/vault/tls/ca.crt" \
  -H "X-Vault-Token: $vault_token" https://127.0.0.1:18200/v1/secret/data/chronicle)" || {
    echo "Missing Vault KV secret secret/chronicle. Run ./scripts/chronicle/bootstrap.sh first." >&2
    exit 1
  }
unset vault_token

temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/chronicle-setup.XXXXXX")"
credentials_file="$temp_dir/credentials.env"
jq -r '.data.data | to_entries[] | "\(.key)=\(.value)"' <<<"$vault_secret" > "$credentials_file"
unset vault_secret
kubectl -n "$namespace" create secret generic "$credentials_secret" \
  --from-env-file="$credentials_file" --dry-run=client -o yaml | kubectl apply -f - >/dev/null

# Chronicle signer: reuse existing in-cluster secret if present; otherwise seed
# demo keys from the package manifests so the chart installs cleanly.
if ! kubectl -n "$namespace" get secret "$signer_secret" >/dev/null 2>&1; then
  kubectl -n "$namespace" create secret generic "$signer_secret" \
    --from-literal=chronicle-signer.key='PRIVATE+KEY+chronicle+b59447d9+AWa3mRH8LpAqPYER7xo1r17eF+OOwn78YCeE/C43UErX' \
    --from-literal=chronicle-signer.key.pub='chronicle+b59447d9+AQrolSg0fHgLthI/SMPGLWKSxjh68AD1nFC1lfuFRJci' \
    --dry-run=client -o yaml | kubectl apply -f - >/dev/null
fi

# Demo witness keys for the simulated witness quorum.
if ! kubectl -n "$namespace" get secret "$demo_keys_secret" >/dev/null 2>&1; then
  kubectl apply -n "$namespace" -f "$package_dir/manifests/witness-demo-keys.yaml" >/dev/null
fi

# Trust bundle for RustFS TLS, mounted at /etc/chronicle/trust.
kubectl -n "$namespace" create secret generic "$trust_secret" \
  --from-file=rustfs-ca.crt="$repo_root/k8s/storage/tls/ca.crt" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

helm upgrade --install "$release" "$chart" \
  --namespace "$namespace" \
  --set-string "chronicle.image.repository=$image_repository" \
  --set-string "chronicle.image.tag=$image_tag" \
  --set-string "chronicle.credentialsSecretName=$credentials_secret" \
  --set-string "chronicle.signerSecretName=$signer_secret" \
  --set-string "chronicle.trustSecretName=$trust_secret" \
  --set-string "witness.demoKeysSecretName=$demo_keys_secret" \
  --wait --timeout 10m "$@"

if [[ "$skip_image_build" != 1 ]]; then
  kubectl -n "$namespace" rollout restart deployment/"$release"
fi
kubectl -n "$namespace" rollout status deployment/"$release" --timeout=5m

echo "Chronicle is ready."
echo "Service: svc/$release.$namespace (port 80)"
echo "Local port-forward: kubectl -n $namespace port-forward service/$release 3000:80"
echo "Traces will stream to the SigNoz OTel Collector at the endpoint set in values.yaml."
