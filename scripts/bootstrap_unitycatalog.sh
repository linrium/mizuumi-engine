#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
vault_ca="$repo_root/k8s/vault/tls/ca.crt"
keycloak_ca="$repo_root/k8s/auth/tls/ca.crt"
rustfs_ca="$repo_root/k8s/storage/tls/ca.crt"
uc_email="${UNITYCATALOG_USER_EMAIL:-khaopad@mizuumi.test}"

for command in kubectl curl jq openssl; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local bootstrap on non-local context: $context" >&2; exit 1 ;;
esac

for ca in "$vault_ca" "$keycloak_ca" "$rustfs_ca"; do
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

admin_user="$(kubectl -n auth get deployment keycloak -o json | jq -er '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_BOOTSTRAP_ADMIN_USERNAME") | .value')"
admin_password="$(kubectl -n auth get secret keycloak-credentials -o json | jq -er '.data.ADMIN_PASSWORD | @base64d')"
rustfs_secret="$(kubectl -n storage get secret rustfs-credentials -o json)"
rustfs_root_access="$(printf '%s' "$rustfs_secret" | jq -er '.data.RUSTFS_ACCESS_KEY | @base64d')"
rustfs_root_secret="$(printf '%s' "$rustfs_secret" | jq -er '.data.RUSTFS_SECRET_KEY | @base64d')"
unset rustfs_secret

keycloak_url=https://127.0.0.1:18080
vault_url=https://127.0.0.1:18200
rustfs_url=https://127.0.0.1:19000
keycloak_pid=""
vault_pid=""
rustfs_pid=""
response_dir="$(mktemp -d "${TMPDIR:-/tmp}/unitycatalog-bootstrap.XXXXXX")"
cleanup() {
  for pid in "$keycloak_pid" "$vault_pid" "$rustfs_pid"; do
    [[ -z "$pid" ]] || { kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; }
  done
  rm -rf "$response_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

kubectl -n auth port-forward --address 127.0.0.1 service/keycloak 18080:8080 >/dev/null 2>&1 & keycloak_pid=$!
kubectl -n vault port-forward --address 127.0.0.1 pod/vault-0 18200:8200 >/dev/null 2>&1 & vault_pid=$!
kubectl -n storage port-forward --address 127.0.0.1 service/rustfs-svc 19000:9000 >/dev/null 2>&1 & rustfs_pid=$!

for attempt in {1..60}; do
  if curl -fsS --cacert "$keycloak_ca" "$keycloak_url/realms/sovico/.well-known/openid-configuration" >/dev/null 2>&1 &&
     curl -fsS --cacert "$vault_ca" "$vault_url/v1/sys/health" >/dev/null 2>&1 &&
     curl -fsS --cacert "$rustfs_ca" "$rustfs_url/minio/health/live" >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then echo "Keycloak, Vault, or RustFS is unavailable." >&2; exit 1; fi
  sleep 1
done

admin_token="$(printf '%s' "$admin_password" | curl -fsS --cacert "$keycloak_ca" -X POST \
  --data-urlencode 'password@-' --data-urlencode "username=$admin_user" \
  --data-urlencode 'client_id=admin-cli' --data-urlencode 'grant_type=password' \
  "$keycloak_url/realms/master/protocol/openid-connect/token" | jq -er '.access_token')"
unset admin_password

kc() {
  local method="$1" path="$2"
  if [[ $# -eq 3 ]]; then
    printf '%s' "$3" | curl -fsS --cacert "$keycloak_ca" -X "$method" \
      -H "Authorization: Bearer $admin_token" -H 'Content-Type: application/json' \
      --data-binary @- "$keycloak_url$path"
  else
    curl -fsS --cacert "$keycloak_ca" -X "$method" -H "Authorization: Bearer $admin_token" "$keycloak_url$path"
  fi
}

upsert_client() {
  local client_id="$1" payload="$2" clients client_uuid current
  clients="$(kc GET "/admin/realms/sovico/clients?clientId=$client_id")"
  client_uuid="$(printf '%s' "$clients" | jq -r --arg id "$client_id" '[.[] | select(.clientId == $id)][0].id // empty')"
  if [[ -z "$client_uuid" ]]; then
    kc POST /admin/realms/sovico/clients "$payload" >/dev/null
    client_uuid="$(kc GET "/admin/realms/sovico/clients?clientId=$client_id" | jq -er --arg id "$client_id" '[.[] | select(.clientId == $id)][0].id')"
  else
    current="$(kc GET "/admin/realms/sovico/clients/$client_uuid")"
    payload="$(printf '%s' "$payload" | jq -c --argjson current "$current" '$current * .')"
    kc PUT "/admin/realms/sovico/clients/$client_uuid" "$payload" >/dev/null
  fi
  printf '%s' "$client_uuid"
}

server_payload='{"clientId":"unitycatalog","enabled":true,"protocol":"openid-connect","publicClient":false,"clientAuthenticatorType":"client-secret","standardFlowEnabled":true,"directAccessGrantsEnabled":false,"serviceAccountsEnabled":false,"redirectUris":["http://localhost:*"],"webOrigins":[]}'
server_uuid="$(upsert_client unitycatalog "$server_payload")"
oauth_client_secret="$(kc GET "/admin/realms/sovico/clients/$server_uuid/client-secret" | jq -er '.value')"

# Remove the obsolete browser client from earlier UI-enabled installations.
old_ui_clients="$(kc GET '/admin/realms/sovico/clients?clientId=unitycatalog-ui')"
while IFS= read -r old_ui_uuid; do
  [[ -z "$old_ui_uuid" ]] || kc DELETE "/admin/realms/sovico/clients/$old_ui_uuid" >/dev/null
done < <(printf '%s' "$old_ui_clients" | jq -r '.[] | select(.clientId == "unitycatalog-ui") | .id')

users="$(kc GET '/admin/realms/sovico/users?username=khaopad&exact=true')"
user_id="$(printf '%s' "$users" | jq -er '[.[] | select(.username == "khaopad")][0].id')"
current_user="$(kc GET "/admin/realms/sovico/users/$user_id")"
updated_user="$(printf '%s' "$current_user" | jq -c --arg email "$uc_email" '.email=$email | .emailVerified=true')"
kc PUT "/admin/realms/sovico/users/$user_id" "$updated_user" >/dev/null

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
  -H "X-Vault-Token: $vault_token" "$vault_url/v1/secret/data/unitycatalog")"
case "$secret_status" in
  200)
    s3_access_key="$(jq -er '.data.data.S3_ACCESS_KEY' "$response_dir/vault-secret.json")"
    s3_secret_key="$(jq -er '.data.data.S3_SECRET_KEY' "$response_dir/vault-secret.json")"
    ;;
  404)
    s3_access_key="UC$(openssl rand -hex 10 | tr '[:lower:]' '[:upper:]')"
    s3_secret_key="$(openssl rand -hex 32)"
    ;;
  *) echo "Cannot read Unity Catalog secret from Vault (HTTP $secret_status)." >&2; exit 1 ;;
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
  --aws-sigv4 "aws:amz:us-east-1:s3" --user "$s3_access_key:$s3_secret_key" "$rustfs_url/unitycatalog")"
[[ "$bucket_status" == 200 || "$bucket_status" == 409 ]] || { echo "RustFS bucket creation failed (HTTP $bucket_status): $(<"$response_dir/rustfs-bucket")" >&2; exit 1; }

sts_status="$(curl -sS --cacert "$rustfs_ca" -o "$response_dir/rustfs-sts" -w '%{http_code}' -X POST \
  --aws-sigv4 "aws:amz:us-east-1:s3" --user "$s3_access_key:$s3_secret_key" \
  --data-urlencode 'Action=AssumeRole' --data-urlencode 'Version=2011-06-15' \
  --data-urlencode 'DurationSeconds=900' "$rustfs_url/")"
[[ "$sts_status" == 200 ]] || { echo "RustFS STS validation failed (HTTP $sts_status): $(<"$response_dir/rustfs-sts")" >&2; exit 1; }

vault_payload="$(jq -cn \
  --arg client_id unitycatalog --arg client_secret "$oauth_client_secret" \
  --arg access "$s3_access_key" --arg secret "$s3_secret_key" \
  '{data:{OAUTH_CLIENT_ID:$client_id,OAUTH_CLIENT_SECRET:$client_secret,S3_ACCESS_KEY:$access,S3_SECRET_KEY:$secret}}')"
vault POST /v1/secret/data/unitycatalog "$vault_payload" >/dev/null

unset admin_token vault_token oauth_client_secret rustfs_root_access rustfs_root_secret s3_secret_key
echo "Unity Catalog bootstrap complete: Keycloak client, RustFS IAM/STS bucket, and Vault KV secret are ready."
echo "Keycloak user khaopad will map to Unity Catalog user $uc_email."
echo "Run ./scripts/setup_unitycatalog.sh."
