#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace="${UNITYCATALOG_NAMESPACE:-tower}"
release="${UNITYCATALOG_RELEASE:-unitycatalog}"
jwt_secret="${UNITYCATALOG_JWT_SECRET:-unitycatalog-jwt}"
credentials_secret="${UNITYCATALOG_CREDENTIALS_SECRET:-unitycatalog-credentials}"
trust_secret="${UNITYCATALOG_TRUST_SECRET:-unitycatalog-trust}"
uc_email="${UNITYCATALOG_USER_EMAIL:-khaopad@mizuumi.test}"
rustfs_role_arn="${UNITYCATALOG_RUSTFS_ROLE_ARN:-arn:aws:iam::000000000000:role/unitycatalog}"
chart="$repo_root/k8s/unitycatalog"
key_dir=""
vault_pid=""

cleanup() {
  [[ -z "$vault_pid" ]] || { kill "$vault_pid" 2>/dev/null || true; wait "$vault_pid" 2>/dev/null || true; }
  [[ -z "$key_dir" ]] || rm -rf "$key_dir"
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage: ./scripts/setup_unitycatalog.sh [helm upgrade options]

Installs or upgrades Unity Catalog OSS in the current Kubernetes context.
Additional arguments are forwarded to `helm upgrade --install`, for example:
  ./scripts/setup_unitycatalog.sh --set server.persistence.size=10Gi
  ./scripts/setup_unitycatalog.sh --values /path/to/values.yaml

Environment overrides:
  UNITYCATALOG_NAMESPACE   Kubernetes namespace (default: tower)
  UNITYCATALOG_RELEASE     Helm release name (default: unitycatalog)
  UNITYCATALOG_JWT_SECRET  Signing-key Secret name (default: unitycatalog-jwt)
  UNITYCATALOG_USER_EMAIL  Keycloak/UC user email (default: khaopad@mizuumi.test)
  UNITYCATALOG_RUSTFS_ROLE_ARN  RustFS STS role ARN
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

for command in kubectl helm openssl curl jq; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local Unity Catalog values on non-local context: $context" >&2; exit 1 ;;
esac
echo "Using Kubernetes context: $context"

for required in \
  "$repo_root/k8s/auth/tls/ca.crt" \
  "$repo_root/k8s/storage/tls/ca.crt" \
  "$repo_root/k8s/vault/tls/ca.crt"; do
  [[ -f "$required" ]] || { echo "Missing $required; bootstrap auth, Vault, and storage first." >&2; exit 1; }
done

if ! kubectl -n auth get configmap keycloak-gateway -o jsonpath='{.data.Caddyfile}' 2>/dev/null | grep -q 'uc.mizuumi.test'; then
  echo "Updating the shared gateway and certificate for uc.mizuumi.test."
  "$repo_root/scripts/setup_auth.sh"
fi

if ! kubectl get namespace "$namespace" >/dev/null 2>&1; then
  kubectl create namespace "$namespace"
fi

if kubectl -n "$namespace" get secret "$jwt_secret" >/dev/null 2>&1; then
  echo "Using existing Secret: $jwt_secret"
else
  key_dir="$(mktemp -d "${TMPDIR:-/tmp}/unitycatalog-keys.XXXXXX")"

  openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 \
    -out "$key_dir/private_key.pem" 2>/dev/null
  openssl pkcs8 -topk8 -inform PEM -outform DER -nocrypt \
    -in "$key_dir/private_key.pem" -out "$key_dir/private_key.der"
  openssl pkey -in "$key_dir/private_key.pem" -pubout -outform DER \
    -out "$key_dir/public_key.der"
  openssl rand -hex 32 > "$key_dir/key_id.txt"

  kubectl -n "$namespace" create secret generic "$jwt_secret" \
    --from-file=private_key.der="$key_dir/private_key.der" \
    --from-file=public_key.der="$key_dir/public_key.der" \
    --from-file=key_id.txt="$key_dir/key_id.txt"
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

credentials_file="$(mktemp "${TMPDIR:-/tmp}/unitycatalog-credentials.XXXXXX")"
key_dir="${key_dir:-$(mktemp -d "${TMPDIR:-/tmp}/unitycatalog-cleanup.XXXXXX")}"
mv "$credentials_file" "$key_dir/credentials.env"
jq -r '.data.data | to_entries[] | "\(.key)=\(.value)"' <<<"$vault_secret" > "$key_dir/credentials.env"
unset vault_secret
kubectl -n "$namespace" create secret generic "$credentials_secret" \
  --from-env-file="$key_dir/credentials.env" --dry-run=client -o yaml | kubectl apply -f - >/dev/null
kubectl -n "$namespace" create secret generic "$trust_secret" \
  --from-file=keycloak-ca.crt="$repo_root/k8s/auth/tls/ca.crt" \
  --from-file=rustfs-ca.crt="$repo_root/k8s/storage/tls/ca.crt" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

helm upgrade --install "$release" "$chart" \
  --namespace "$namespace" \
  --set-string "server.jwt.secretName=$jwt_secret" \
  --set-string "server.credentialsSecretName=$credentials_secret" \
  --set-string "server.trustSecretName=$trust_secret" \
  --wait --timeout 10m "$@"

server_deployment="$(kubectl -n "$namespace" get deployment \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}')"
server_service="$(kubectl -n "$namespace" get service \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}')"
kubectl -n "$namespace" rollout status deployment/"$server_deployment" --timeout=5m

run_uc_admin() {
  local output
  if output="$(kubectl -n "$namespace" exec deployment/"$server_deployment" -c server -- \
    /bin/bash -ec 'token="$(< /home/unitycatalog/etc/conf/token.txt)"; exec bin/uc --auth_token "$token" "$@"' -- "$@" 2>&1)"; then
    return 0
  fi
  if [[ "$output" == *ALREADY_EXISTS* || "$output" == *"already exists"* ]]; then
    return 0
  fi
  printf '%s\n' "$output" >&2
  return 1
}

run_uc_admin credential create --name rustfs_unitycatalog --aws_iam_role_arn "$rustfs_role_arn"
run_uc_admin external_location create --name rustfs_unitycatalog --url s3://unitycatalog --credential_name rustfs_unitycatalog
run_uc_admin catalog create --name unity --storage_root s3://unitycatalog
run_uc_admin schema create --catalog unity --name default
run_uc_admin user create --name khaopad --email "$uc_email"
run_uc_admin permission create --securable_type catalog --name unity --privilege 'USE CATALOG' --principal "$uc_email"
run_uc_admin permission create --securable_type schema --name unity.default --privilege 'USE SCHEMA' --principal "$uc_email"

"$repo_root/scripts/configure_cluster_dns.sh"

echo "Unity Catalog is ready."
echo "API: https://uc.mizuumi.test"
echo "Local fallback: kubectl -n $namespace port-forward service/$server_service 8080:8080"
echo "Use the Unity Catalog CLI to authenticate with the sovico Keycloak realm."
