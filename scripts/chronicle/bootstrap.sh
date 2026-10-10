#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${CHRONICLE_NAMESPACE:-chronicle}"
bucket="${CHRONICLE_S3_BUCKET:-chronicle}"
vault_ca="$repo_root/k8s/vault/tls/ca.crt"
rustfs_ca="$repo_root/k8s/storage/tls/ca.crt"

for command in kubectl curl jq openssl; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local Chronicle bootstrap on non-local context: $context" >&2; exit 1 ;;
esac

for ca in "$vault_ca" "$rustfs_ca"; do
  [[ -f "$ca" ]] || { echo "Missing local CA: $ca" >&2; exit 1; }
done

if [[ -n "${VAULT_TOKEN:-}" ]]; then
  vault_token="$VAULT_TOKEN"
elif [[ -f "$repo_root/k8s/vault/init.json" ]]; then
  vault_token="$(jq -er '.root_token' "$repo_root/k8s/vault/init.json")"
else
  echo "Set VAULT_TOKEN or initialize Vault first." >&2
  exit 1
fi

rustfs_secret="$(kubectl -n storage get secret rustfs-credentials -o json)"
rustfs_root_access="$(printf '%s' "$rustfs_secret" | jq -er '.data.RUSTFS_ACCESS_KEY | @base64d')"
rustfs_root_secret="$(printf '%s' "$rustfs_secret" | jq -er '.data.RUSTFS_SECRET_KEY | @base64d')"
unset rustfs_secret

vault_url=https://127.0.0.1:18200
rustfs_url=https://127.0.0.1:19000
vault_pid=""
rustfs_pid=""
response_dir="$(mktemp -d "${TMPDIR:-/tmp}/chronicle-bootstrap.XXXXXX")"
cleanup() {
  for pid in "$vault_pid" "$rustfs_pid"; do
    [[ -z "$pid" ]] || { kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; }
  done
  rm -rf "$response_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

kubectl -n vault port-forward --address 127.0.0.1 pod/vault-0 18200:8200 >/dev/null 2>&1 & vault_pid=$!
kubectl -n storage port-forward --address 127.0.0.1 service/rustfs-svc 19000:9000 >/dev/null 2>&1 & rustfs_pid=$!

for attempt in {1..60}; do
  if curl -fsS --cacert "$vault_ca" "$vault_url/v1/sys/health" >/dev/null 2>&1 &&
     curl -fsS --cacert "$rustfs_ca" "$rustfs_url/minio/health/live" >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then echo "Vault or RustFS is unavailable." >&2; exit 1; fi
  sleep 1
done

vault() {
  local method="$1" path="$2"
  if [[ $# -eq 3 ]]; then
    printf '%s' "$3" | curl -fsS --cacert "$vault_ca" -X "$method" \
      -H "X-Vault-Token: $vault_token" -H 'Content-Type: application/json' \
      --data-binary @- "$vault_url$path"
  else
    curl -fsS --cacert "$vault_ca" -X "$method" -H "X-Vault-Token: $vault_token" "$vault_url$path"
  fi
}

mounts="$(vault GET /v1/sys/mounts)"
if ! printf '%s' "$mounts" | jq -e 'has("secret/")' >/dev/null; then
  vault POST /v1/sys/mounts/secret '{"type":"kv","options":{"version":"2"}}' >/dev/null
elif [[ "$(printf '%s' "$mounts" | jq -r '."secret/".options.version // "1"')" != 2 ]]; then
  echo "Vault secret/ exists but is not KV v2." >&2
  exit 1
fi

secret_status="$(curl -sS --cacert "$vault_ca" -o "$response_dir/vault-secret.json" -w '%{http_code}' \
  -H "X-Vault-Token: $vault_token" "$vault_url/v1/secret/data/chronicle")"
case "$secret_status" in
  200)
    s3_access_key="$(jq -er '.data.data.S3_ACCESS_KEY' "$response_dir/vault-secret.json")"
    s3_secret_key="$(jq -er '.data.data.S3_SECRET_KEY' "$response_dir/vault-secret.json")"
    ;;
  404)
    s3_access_key="CH$(openssl rand -hex 10 | tr '[:lower:]' '[:upper:]')"
    s3_secret_key="$(openssl rand -hex 32)"
    ;;
  *) echo "Cannot read Chronicle secret from Vault (HTTP $secret_status)." >&2; exit 1 ;;
esac

rustfs_request() {
  local method="$1" path="$2" body="${3:-}" output="$4"
  local args=(--silent --show-error --cacert "$rustfs_ca" -o "$output" -w '%{http_code}' -X "$method"
    --aws-sigv4 "aws:amz:us-east-1:s3" --user "$rustfs_root_access:$rustfs_root_secret")
  if [[ -n "$body" ]]; then args+=(-H 'Content-Type: application/json' --data-binary "$body"); fi
  curl "${args[@]}" "$rustfs_url$path"
}

user_body="$(jq -cn --arg secret "$s3_secret_key" '{secretKey:$secret,status:"enabled"}')"
status="$(rustfs_request PUT "/rustfs/admin/v3/add-user?accessKey=$s3_access_key" "$user_body" "$response_dir/rustfs-user")"
[[ "$status" == 200 || "$status" == 204 ]] || { echo "RustFS user creation failed (HTTP $status): $(<"$response_dir/rustfs-user")" >&2; exit 1; }
status="$(rustfs_request PUT "/rustfs/admin/v3/set-user-or-group-policy?policyName=readwrite&userOrGroup=$s3_access_key&isGroup=false" "" "$response_dir/rustfs-policy")"
[[ "$status" == 200 || "$status" == 204 ]] || { echo "RustFS policy attachment failed (HTTP $status): $(<"$response_dir/rustfs-policy")" >&2; exit 1; }

bucket_status="$(curl -sS --cacert "$rustfs_ca" -o "$response_dir/rustfs-bucket" -w '%{http_code}' -X PUT \
  --aws-sigv4 "aws:amz:us-east-1:s3" --user "$s3_access_key:$s3_secret_key" "$rustfs_url/$bucket")"
[[ "$bucket_status" == 200 || "$bucket_status" == 409 ]] || { echo "RustFS bucket creation failed (HTTP $bucket_status): $(<"$response_dir/rustfs-bucket")" >&2; exit 1; }

vault_payload="$(jq -cn --arg access "$s3_access_key" --arg secret "$s3_secret_key" --arg bucket "$bucket" \
  '{data:{S3_ACCESS_KEY:$access,S3_SECRET_KEY:$secret,S3_BUCKET:$bucket}}')"
vault POST /v1/secret/data/chronicle "$vault_payload" >/dev/null

unset vault_token rustfs_root_access rustfs_root_secret s3_secret_key

echo "Chronicle bootstrap complete: RustFS user, bucket '$bucket', and Vault secret/chronicle are ready."
echo "Run ./scripts/chronicle/setup.sh."
