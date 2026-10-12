#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${DUCK_SERVER_NAMESPACE:-duck-server}"
credentials_secret="${DUCK_SERVER_CREDENTIALS_SECRET:-duck-server-credentials}"
bucket="${DUCK_SERVER_S3_BUCKET:-duck-server}"
realm="${KEYCLOAK_REALM:-sovico}"
keycloak_client_id="${DUCK_SERVER_KEYCLOAK_CLIENT_ID:-duck-server}"
rustfs_ca="$repo_root/k8s/storage/tls/ca.crt"
keycloak_ca="$repo_root/k8s/auth/tls/ca.crt"

usage() {
  cat <<'EOF'
Usage: ./scripts/duck-server/bootstrap.sh

Create or update the Duck server's RustFS identity and its least-privilege
Keycloak token-introspection client.

Environment overrides:
  DUCK_SERVER_NAMESPACE           Default: duck-server
  DUCK_SERVER_CREDENTIALS_SECRET  Default: duck-server-credentials
  DUCK_SERVER_S3_BUCKET           Default: duck-server
  DUCK_SERVER_KEYCLOAK_CLIENT_ID  Default: duck-server
  KEYCLOAK_REALM                  Default: sovico
EOF
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
  "") ;;
  *) usage >&2; exit 2 ;;
esac

for command in kubectl curl jq openssl; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local Duck server bootstrap on non-local context: $context" >&2; exit 1 ;;
esac

[[ -f "$rustfs_ca" ]] || { echo "Missing RustFS CA: $rustfs_ca" >&2; exit 1; }
[[ -f "$keycloak_ca" ]] || { echo "Missing Keycloak CA: $keycloak_ca" >&2; exit 1; }
kubectl get namespace "$namespace" >/dev/null 2>&1 || kubectl create namespace "$namespace"
case "$keycloak_client_id" in
  *[!A-Za-z0-9._-]*|'')
    echo "DUCK_SERVER_KEYCLOAK_CLIENT_ID contains invalid characters." >&2
    exit 1
    ;;
esac

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
keycloak_url=https://127.0.0.1:18080
keycloak_pid=""
response_dir="$(mktemp -d "${TMPDIR:-/tmp}/duck-server-bootstrap.XXXXXX")"
cleanup() {
  [[ -z "$rustfs_pid" ]] || { kill "$rustfs_pid" 2>/dev/null || true; wait "$rustfs_pid" 2>/dev/null || true; }
  [[ -z "$keycloak_pid" ]] || { kill "$keycloak_pid" 2>/dev/null || true; wait "$keycloak_pid" 2>/dev/null || true; }
  rm -rf "$response_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

kubectl -n storage port-forward --address 127.0.0.1 service/rustfs-svc 19000:9000 >/dev/null 2>&1 &
rustfs_pid=$!
kubectl -n auth port-forward --address 127.0.0.1 service/keycloak 18080:8080 >/dev/null 2>&1 &
keycloak_pid=$!
for attempt in {1..60}; do
  if curl -fsS --cacert "$rustfs_ca" "$rustfs_url/minio/health/live" >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then echo "RustFS is unavailable." >&2; exit 1; fi
  sleep 1
done

for attempt in {1..60}; do
  if curl -fsS --cacert "$keycloak_ca" \
    "$keycloak_url/realms/$realm/.well-known/openid-configuration" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$keycloak_pid" 2>/dev/null; then
    echo "Keycloak port-forward stopped unexpectedly; is port 18080 already in use?" >&2
    exit 1
  fi
  if [[ "$attempt" -eq 60 ]]; then echo "Keycloak realm $realm is unavailable." >&2; exit 1; fi
  sleep 1
done

admin_user="$(kubectl -n auth get deployment keycloak -o json | jq -er '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_BOOTSTRAP_ADMIN_USERNAME") | .value')"
admin_password="$(kubectl -n auth get secret keycloak-credentials -o json | jq -er '.data.ADMIN_PASSWORD | @base64d')"
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
    curl -fsS --cacert "$keycloak_ca" -X "$method" \
      -H "Authorization: Bearer $admin_token" "$keycloak_url$path"
  fi
}

clients="$(kc GET "/admin/realms/$realm/clients?clientId=$keycloak_client_id")"
client_uuid="$(printf '%s' "$clients" | jq -r --arg id "$keycloak_client_id" '[.[] | select(.clientId == $id)][0].id // empty')"
client_payload="$(jq -cn --arg id "$keycloak_client_id" \
  '{clientId:$id,name:"Mizuumi Duck query server",enabled:true,protocol:"openid-connect",publicClient:false,clientAuthenticatorType:"client-secret",standardFlowEnabled:false,directAccessGrantsEnabled:false,serviceAccountsEnabled:false}')"
if [[ -z "$client_uuid" ]]; then
  kc POST "/admin/realms/$realm/clients" "$client_payload" >/dev/null
  client_uuid="$(kc GET "/admin/realms/$realm/clients?clientId=$keycloak_client_id" | jq -er --arg id "$keycloak_client_id" '[.[] | select(.clientId == $id)][0].id')"
  echo "Created Keycloak client $realm/$keycloak_client_id."
else
  current_client="$(kc GET "/admin/realms/$realm/clients/$client_uuid")"
  client_payload="$(printf '%s' "$current_client" | jq -c --arg id "$keycloak_client_id" \
    '.clientId=$id | .name="Mizuumi Duck query server" | .enabled=true | .protocol="openid-connect" | .publicClient=false | .clientAuthenticatorType="client-secret" | .standardFlowEnabled=false | .directAccessGrantsEnabled=false | .serviceAccountsEnabled=false')"
  kc PUT "/admin/realms/$realm/clients/$client_uuid" "$client_payload" >/dev/null
  echo "Updated Keycloak client $realm/$keycloak_client_id."
fi
keycloak_client_secret="$(kc GET "/admin/realms/$realm/clients/$client_uuid/client-secret" | jq -r '.value // empty')"
if [[ -z "$keycloak_client_secret" ]]; then
  keycloak_client_secret="$(kc POST "/admin/realms/$realm/clients/$client_uuid/client-secret" '{}' | jq -er '.value')"
fi

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
  --from-literal=KEYCLOAK_CLIENT_ID="$keycloak_client_id" \
  --from-literal=KEYCLOAK_CLIENT_SECRET="$keycloak_client_secret" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

unset admin_token rustfs_root_access rustfs_root_secret s3_secret_key quack_token keycloak_client_secret
echo "Duck server bootstrap complete: RustFS, Quack, and Keycloak introspection credentials are ready."
echo "Run ./scripts/duck-server/setup.sh."
