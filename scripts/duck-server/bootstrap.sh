#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${DUCK_SERVER_NAMESPACE:-duck-server}"
credentials_secret="${DUCK_SERVER_CREDENTIALS_SECRET:-duck-server-credentials}"
bucket="${DUCK_SERVER_S3_BUCKET:-duck-server}"
rustfs_ca="$repo_root/k8s/storage/tls/ca.crt"

for command in kubectl curl jq openssl; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local Duck server bootstrap on non-local context: $context" >&2; exit 1 ;;
esac

[[ -f "$rustfs_ca" ]] || { echo "Missing RustFS CA: $rustfs_ca" >&2; exit 1; }
kubectl get namespace "$namespace" >/dev/null 2>&1 || kubectl create namespace "$namespace"

existing_secret="$(kubectl -n "$namespace" get secret "$credentials_secret" -o json 2>/dev/null || true)"
if [[ -n "$existing_secret" ]]; then
  s3_access_key="$(printf '%s' "$existing_secret" | jq -er '.data.S3_ACCESS_KEY | @base64d')"
  s3_secret_key="$(printf '%s' "$existing_secret" | jq -er '.data.S3_SECRET_KEY | @base64d')"
  quack_token="$(printf '%s' "$existing_secret" | jq -er '.data.QUACK_TOKEN | @base64d')"
else
  s3_access_key="DUCK$(openssl rand -hex 10 | tr '[:lower:]' '[:upper:]')"
  s3_secret_key="$(openssl rand -hex 32)"
  quack_token="$(openssl rand -hex 32)"
fi
unset existing_secret

rustfs_secret="$(kubectl -n storage get secret rustfs-credentials -o json)"
rustfs_root_access="$(printf '%s' "$rustfs_secret" | jq -er '.data.RUSTFS_ACCESS_KEY | @base64d')"
rustfs_root_secret="$(printf '%s' "$rustfs_secret" | jq -er '.data.RUSTFS_SECRET_KEY | @base64d')"
unset rustfs_secret

rustfs_url=https://127.0.0.1:19000
rustfs_pid=""
response_dir="$(mktemp -d "${TMPDIR:-/tmp}/duck-server-bootstrap.XXXXXX")"
cleanup() {
  [[ -z "$rustfs_pid" ]] || { kill "$rustfs_pid" 2>/dev/null || true; wait "$rustfs_pid" 2>/dev/null || true; }
  rm -rf "$response_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

kubectl -n storage port-forward --address 127.0.0.1 service/rustfs-svc 19000:9000 >/dev/null 2>&1 &
rustfs_pid=$!
for attempt in {1..60}; do
  if curl -fsS --cacert "$rustfs_ca" "$rustfs_url/minio/health/live" >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then echo "RustFS is unavailable." >&2; exit 1; fi
  sleep 1
done

rustfs_admin_request() {
  local method="$1" path="$2" body="${3:-}" output="$4"
  local args=(--silent --show-error --cacert "$rustfs_ca" -o "$output" -w '%{http_code}' -X "$method"
    --aws-sigv4 "aws:amz:us-east-1:s3" --user "$rustfs_root_access:$rustfs_root_secret")
  if [[ -n "$body" ]]; then args+=(-H 'Content-Type: application/json' --data-binary "$body"); fi
  curl "${args[@]}" "$rustfs_url$path"
}

user_body="$(jq -cn --arg secret "$s3_secret_key" '{secretKey:$secret,status:"enabled"}')"
status="$(rustfs_admin_request PUT "/rustfs/admin/v3/add-user?accessKey=$s3_access_key" "$user_body" "$response_dir/user")"
case "$status" in
  200|204) ;;
  *) echo "RustFS user creation failed (HTTP $status): $(<"$response_dir/user")" >&2; exit 1 ;;
esac

status="$(rustfs_admin_request PUT "/rustfs/admin/v3/set-user-or-group-policy?policyName=readwrite&userOrGroup=$s3_access_key&isGroup=false" "" "$response_dir/policy")"
case "$status" in
  200|204) ;;
  *) echo "RustFS policy attachment failed (HTTP $status): $(<"$response_dir/policy")" >&2; exit 1 ;;
esac

bucket_status="$(curl -sS --cacert "$rustfs_ca" -o "$response_dir/bucket" -w '%{http_code}' -X PUT \
  --aws-sigv4 "aws:amz:us-east-1:s3" --user "$s3_access_key:$s3_secret_key" "$rustfs_url/$bucket")"
case "$bucket_status" in
  200|409) ;;
  *) echo "RustFS bucket creation failed (HTTP $bucket_status): $(<"$response_dir/bucket")" >&2; exit 1 ;;
esac

kubectl -n "$namespace" create secret generic "$credentials_secret" \
  --from-literal=S3_ACCESS_KEY="$s3_access_key" \
  --from-literal=S3_SECRET_KEY="$s3_secret_key" \
  --from-literal=QUACK_TOKEN="$quack_token" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

unset rustfs_root_access rustfs_root_secret s3_secret_key quack_token
echo "Duck server bootstrap complete: RustFS user, bucket '$bucket', and Kubernetes credentials are ready."
echo "Run ./scripts/duck-server/setup.sh."
