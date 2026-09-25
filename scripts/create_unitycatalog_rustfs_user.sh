#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace="${UNITYCATALOG_NAMESPACE:-tower}"
credentials_secret="${UNITYCATALOG_CREDENTIALS_SECRET:-unitycatalog-credentials}"
rustfs_namespace="${RUSTFS_NAMESPACE:-storage}"
rustfs_service="${RUSTFS_SERVICE:-rustfs-svc}"
rustfs_credentials_secret="${RUSTFS_CREDENTIALS_SECRET:-rustfs-credentials}"
bucket="${UNITYCATALOG_BUCKET:-unitycatalog}"
policy="${UNITYCATALOG_RUSTFS_POLICY:-readwrite}"
access_key="${UNITYCATALOG_S3_ACCESS_KEY:-}"
secret_key="${UNITYCATALOG_S3_SECRET_KEY:-}"
sync_vault=1
rustfs_pid=""
vault_pid=""
response_dir=""

cleanup() {
  [[ -z "$rustfs_pid" ]] || { kill "$rustfs_pid" 2>/dev/null || true; wait "$rustfs_pid" 2>/dev/null || true; }
  [[ -z "$vault_pid" ]] || { kill "$vault_pid" 2>/dev/null || true; wait "$vault_pid" 2>/dev/null || true; }
  [[ -z "$response_dir" ]] || rm -rf "$response_dir"
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage: ./scripts/create_unitycatalog_rustfs_user.sh [--skip-vault]

Creates a non-root RustFS IAM user for the Rust Unity Catalog server, attaches
the configured RustFS policy, creates the Unity Catalog bucket, validates STS,
and stores the user key in the Kubernetes Secret consumed by the Helm chart.

The RustFS root key is read from the storage namespace only to call RustFS admin
APIs during provisioning. It is never stored in the Unity Catalog namespace.

Environment overrides:
  UNITYCATALOG_NAMESPACE             Target namespace (default: tower)
  UNITYCATALOG_CREDENTIALS_SECRET    Target Secret (default: unitycatalog-credentials)
  UNITYCATALOG_BUCKET                Bucket to create (default: unitycatalog)
  UNITYCATALOG_RUSTFS_POLICY         Policy to attach (default: readwrite)
  UNITYCATALOG_S3_ACCESS_KEY         Optional fixed non-root access key
  UNITYCATALOG_S3_SECRET_KEY         Optional fixed non-root secret key
  RUSTFS_NAMESPACE                   RustFS namespace (default: storage)
  RUSTFS_SERVICE                     RustFS service name (default: rustfs-svc)
  RUSTFS_CREDENTIALS_SECRET          RustFS root Secret (default: rustfs-credentials)

By default the script also writes S3_ACCESS_KEY and S3_SECRET_KEY to Vault KV
secret/data/unitycatalog when Vault is initialized. Pass --skip-vault to skip it.
EOF
}

case "${1:-}" in
  --skip-vault)
    sync_vault=0
    shift
    ;;
  -h|--help)
    usage
    exit 0
    ;;
esac
if [[ $# -ne 0 ]]; then
  usage >&2
  exit 2
fi

for command in kubectl curl jq openssl; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

rustfs_ca="$repo_root/k8s/storage/tls/ca.crt"
vault_ca="$repo_root/k8s/vault/tls/ca.crt"
[[ -f "$rustfs_ca" ]] || { echo "Missing RustFS CA: $rustfs_ca" >&2; exit 1; }

if ! kubectl get namespace "$namespace" >/dev/null 2>&1; then
  kubectl create namespace "$namespace"
fi

if kubectl -n "$namespace" get secret "$credentials_secret" >/dev/null 2>&1; then
  existing_secret="$(kubectl -n "$namespace" get secret "$credentials_secret" -o json)"
  [[ -n "$access_key" ]] || access_key="$(printf '%s' "$existing_secret" | jq -r '.data.S3_ACCESS_KEY // empty | @base64d')"
  [[ -n "$secret_key" ]] || secret_key="$(printf '%s' "$existing_secret" | jq -r '.data.S3_SECRET_KEY // empty | @base64d')"
  unset existing_secret
fi

[[ -n "$access_key" ]] || access_key="UC$(openssl rand -hex 10 | tr '[:lower:]' '[:upper:]')"
[[ -n "$secret_key" ]] || secret_key="$(openssl rand -hex 32)"

rustfs_secret="$(kubectl -n "$rustfs_namespace" get secret "$rustfs_credentials_secret" -o json)"
rustfs_root_access="$(printf '%s' "$rustfs_secret" | jq -er '.data.RUSTFS_ACCESS_KEY | @base64d')"
rustfs_root_secret="$(printf '%s' "$rustfs_secret" | jq -er '.data.RUSTFS_SECRET_KEY | @base64d')"
unset rustfs_secret

response_dir="$(mktemp -d "${TMPDIR:-/tmp}/unitycatalog-rustfs-user.XXXXXX")"
rustfs_url=https://127.0.0.1:19000
kubectl -n "$rustfs_namespace" port-forward --address 127.0.0.1 "service/$rustfs_service" 19000:9000 >/dev/null 2>&1 &
rustfs_pid=$!

for attempt in {1..60}; do
  if curl -fsS --cacert "$rustfs_ca" "$rustfs_url/minio/health/live" >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then
    echo "RustFS is unavailable through service/$rustfs_service." >&2
    exit 1
  fi
  sleep 1
done

rustfs_admin_request() {
  local method="$1" path="$2" body="${3:-}" output="$4"
  local args=(--silent --show-error --cacert "$rustfs_ca" -o "$output" -w '%{http_code}' -X "$method"
    --aws-sigv4 "aws:amz:us-east-1:s3" --user "$rustfs_root_access:$rustfs_root_secret")
  if [[ -n "$body" ]]; then args+=(-H 'Content-Type: application/json' --data-binary "$body"); fi
  curl "${args[@]}" "$rustfs_url$path"
}

user_body="$(jq -cn --arg secret "$secret_key" '{secretKey:$secret,status:"enabled"}')"
status="$(rustfs_admin_request PUT "/rustfs/admin/v3/add-user?accessKey=$access_key" "$user_body" "$response_dir/rustfs-user")"
case "$status" in
  200|204|409) ;;
  *) echo "RustFS user creation failed (HTTP $status): $(<"$response_dir/rustfs-user")" >&2; exit 1 ;;
esac

status="$(rustfs_admin_request PUT "/rustfs/admin/v3/set-user-or-group-policy?policyName=$policy&userOrGroup=$access_key&isGroup=false" "" "$response_dir/rustfs-policy")"
case "$status" in
  200|204) ;;
  *) echo "RustFS policy attachment failed (HTTP $status): $(<"$response_dir/rustfs-policy")" >&2; exit 1 ;;
esac

bucket_status="$(curl -sS --cacert "$rustfs_ca" -o "$response_dir/rustfs-bucket" -w '%{http_code}' -X PUT \
  --aws-sigv4 "aws:amz:us-east-1:s3" --user "$access_key:$secret_key" "$rustfs_url/$bucket")"
case "$bucket_status" in
  200|409) ;;
  *) echo "RustFS bucket creation failed (HTTP $bucket_status): $(<"$response_dir/rustfs-bucket")" >&2; exit 1 ;;
esac

sts_status="$(curl -sS --cacert "$rustfs_ca" -o "$response_dir/rustfs-sts" -w '%{http_code}' -X POST \
  --aws-sigv4 "aws:amz:us-east-1:s3" --user "$access_key:$secret_key" \
  --data-urlencode 'Action=AssumeRole' --data-urlencode 'Version=2011-06-15' \
  --data-urlencode 'DurationSeconds=900' "$rustfs_url/")"
[[ "$sts_status" == 200 ]] || { echo "RustFS STS validation failed (HTTP $sts_status): $(<"$response_dir/rustfs-sts")" >&2; exit 1; }

kubectl -n "$namespace" create secret generic "$credentials_secret" \
  --from-literal=S3_ACCESS_KEY="$access_key" \
  --from-literal=S3_SECRET_KEY="$secret_key" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

if (( sync_vault )); then
  if [[ -n "${VAULT_TOKEN:-}" ]]; then
    vault_token="$VAULT_TOKEN"
  elif [[ -f "$repo_root/k8s/vault/init.json" ]]; then
    vault_token="$(jq -er '.root_token' "$repo_root/k8s/vault/init.json")"
  else
    vault_token=""
  fi

  if [[ -n "$vault_token" && -f "$vault_ca" ]]; then
    vault_url=https://127.0.0.1:18200
    kubectl -n vault port-forward --address 127.0.0.1 pod/vault-0 18200:8200 >/dev/null 2>&1 &
    vault_pid=$!
    for attempt in {1..60}; do
      if curl -fsS --cacert "$vault_ca" "$vault_url/v1/sys/health" >/dev/null 2>&1; then
        break
      fi
      if [[ "$attempt" -eq 60 ]]; then
        echo "Vault is unavailable; Kubernetes Secret was created but Vault was not updated." >&2
        vault_token=""
        break
      fi
      sleep 1
    done
  fi

  if [[ -n "${vault_token:-}" ]]; then
    secret_status="$(curl -sS --cacert "$vault_ca" -o "$response_dir/vault-secret.json" -w '%{http_code}' \
      -H "X-Vault-Token: $vault_token" "$vault_url/v1/secret/data/unitycatalog")"
    case "$secret_status" in
      200)
        vault_payload="$(jq -cn \
          --slurpfile existing "$response_dir/vault-secret.json" \
          --arg access "$access_key" --arg secret "$secret_key" \
          '{data:($existing[0].data.data + {S3_ACCESS_KEY:$access,S3_SECRET_KEY:$secret})}')"
        ;;
      404)
        vault_payload="$(jq -cn --arg access "$access_key" --arg secret "$secret_key" \
          '{data:{S3_ACCESS_KEY:$access,S3_SECRET_KEY:$secret}}')"
        ;;
      *) echo "Cannot read Vault secret/data/unitycatalog (HTTP $secret_status); Kubernetes Secret was created." >&2; vault_payload="" ;;
    esac
    if [[ -n "${vault_payload:-}" ]]; then
      printf '%s' "$vault_payload" | curl -fsS --cacert "$vault_ca" -X POST \
        -H "X-Vault-Token: $vault_token" -H 'Content-Type: application/json' \
        --data-binary @- "$vault_url/v1/secret/data/unitycatalog" >/dev/null
    fi
  fi
fi

unset rustfs_root_access rustfs_root_secret secret_key vault_token
echo "Unity Catalog RustFS user is ready."
echo "Kubernetes Secret: $namespace/$credentials_secret"
echo "Helm values: vending.accessKeySecretKey=S3_ACCESS_KEY, vending.secretKeySecretKey=S3_SECRET_KEY"
