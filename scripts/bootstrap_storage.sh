#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
credentials_file="$repo_root/k8s/storage/credentials.env"
vault_ca="$repo_root/k8s/vault/tls/ca.crt"
keycloak_ca="$repo_root/k8s/auth/tls/ca.crt"
for command in kubectl curl jq openssl; do
  command -v "$command" >/dev/null || { echo "Missing $command" >&2; exit 1; }
done
context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local bootstrap on non-local context: $context" >&2; exit 1 ;;
esac
[[ -f "$vault_ca" ]] || { echo "Run ./scripts/setup_vault.sh first." >&2; exit 1; }
[[ -f "$keycloak_ca" ]] || { echo "Run ./scripts/setup_auth.sh first." >&2; exit 1; }
keycloak_curl() { curl --cacert "$keycloak_ca" "$@"; }
if [[ -n "${VAULT_TOKEN:-}" ]]; then
  vault_token="$VAULT_TOKEN"
elif [[ -f "$repo_root/k8s/vault/init.json" ]]; then
  vault_token="$(jq -er '.root_token' "$repo_root/k8s/vault/init.json")"
else
  echo "Set VAULT_TOKEN or initialize Vault with ./scripts/init_vault.sh." >&2
  exit 1
fi

admin_user="$(kubectl -n auth get deployment keycloak -o json | jq -er '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_BOOTSTRAP_ADMIN_USERNAME") | .value')"
admin_password="$(kubectl -n auth get secret keycloak-credentials -o json | jq -er '.data.ADMIN_PASSWORD | @base64d')"
keycloak_url=https://127.0.0.1:18080
vault_url=https://127.0.0.1:18200
keycloak_pid=""
vault_pid=""
cleanup() {
  [[ -z "$keycloak_pid" ]] || { kill "$keycloak_pid" 2>/dev/null || true; wait "$keycloak_pid" 2>/dev/null || true; }
  [[ -z "$vault_pid" ]] || { kill "$vault_pid" 2>/dev/null || true; wait "$vault_pid" 2>/dev/null || true; }
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
kubectl -n auth port-forward --address 127.0.0.1 service/keycloak 18080:8080 >/dev/null 2>&1 &
keycloak_pid=$!
kubectl -n vault port-forward --address 127.0.0.1 pod/vault-0 18200:8200 >/dev/null 2>&1 &
vault_pid=$!
for attempt in {1..60}; do
  if keycloak_curl -fsS "$keycloak_url/realms/sovico/.well-known/openid-configuration" >/dev/null 2>&1 &&
     curl -fsS --cacert "$vault_ca" "$vault_url/v1/sys/health" >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then echo "Keycloak or Vault unavailable/sealed." >&2; exit 1; fi
  sleep 1
done

admin_token="$(printf '%s' "$admin_password" | keycloak_curl -fsS -X POST \
  --data-urlencode 'password@-' --data-urlencode "username=$admin_user" \
  --data-urlencode 'client_id=admin-cli' --data-urlencode 'grant_type=password' \
  "$keycloak_url/realms/master/protocol/openid-connect/token" | jq -er '.access_token')"
unset admin_password
kc() {
  local method="$1" path="$2"
  if [[ $# -eq 3 ]]; then
    printf '%s' "$3" | keycloak_curl -fsS -X "$method" -H "Authorization: Bearer $admin_token" \
      -H 'Content-Type: application/json' --data-binary @- "$keycloak_url$path"
  else
    keycloak_curl -fsS -X "$method" -H "Authorization: Bearer $admin_token" "$keycloak_url$path"
  fi
}
vault() {
  local method="$1" path="$2"
  if [[ $# -eq 3 ]]; then
    printf '%s' "$3" | curl -fsS --cacert "$vault_ca" -X "$method" \
      -H "X-Vault-Token: $vault_token" -H 'Content-Type: application/json' \
      --data-binary @- "$vault_url$path"
  else
    curl -fsS --cacert "$vault_ca" -X "$method" \
      -H "X-Vault-Token: $vault_token" "$vault_url$path"
  fi
}

clients="$(kc GET '/admin/realms/sovico/clients?clientId=rustfs-console')"
client_uuid="$(printf '%s' "$clients" | jq -r '[.[] | select(.clientId == "rustfs-console")][0].id // empty')"
callback=https://localhost:9000/rustfs/admin/v3/oidc/callback/default
client_payload="$(jq -cn --arg callback "$callback" '{clientId:"rustfs-console",enabled:true,protocol:"openid-connect",publicClient:false,clientAuthenticatorType:"client-secret",standardFlowEnabled:true,directAccessGrantsEnabled:false,serviceAccountsEnabled:false,redirectUris:[$callback],webOrigins:["https://localhost:9001"],attributes:{"pkce.code.challenge.method":"S256"}}')"
if [[ -z "$client_uuid" ]]; then
  kc POST /admin/realms/sovico/clients "$client_payload" >/dev/null
  client_uuid="$(kc GET '/admin/realms/sovico/clients?clientId=rustfs-console' | jq -er '[.[] | select(.clientId == "rustfs-console")][0].id')"
else
  current_client="$(kc GET "/admin/realms/sovico/clients/$client_uuid")"
  client_payload="$(printf '%s' "$current_client" | jq -c --arg callback "$callback" '.enabled=true | .publicClient=false | .clientAuthenticatorType="client-secret" | .standardFlowEnabled=true | .directAccessGrantsEnabled=false | .serviceAccountsEnabled=false | .redirectUris=[$callback] | .webOrigins=["https://localhost:9001"] | .attributes["pkce.code.challenge.method"]="S256"')"
  kc PUT "/admin/realms/sovico/clients/$client_uuid" "$client_payload" >/dev/null
fi
mapper_list="$(kc GET "/admin/realms/sovico/clients/$client_uuid/protocol-mappers/models")"
if ! printf '%s' "$mapper_list" | jq -e 'any(.[]; .name == "groups")' >/dev/null; then
  kc POST "/admin/realms/sovico/clients/$client_uuid/protocol-mappers/models" \
    '{"name":"groups","protocol":"openid-connect","protocolMapper":"oidc-group-membership-mapper","consentRequired":false,"config":{"full.path":"false","id.token.claim":"true","access.token.claim":"true","userinfo.token.claim":"true","claim.name":"groups"}}' >/dev/null
fi
client_secret="$(kc GET "/admin/realms/sovico/clients/$client_uuid/client-secret" | jq -er '.value')"
groups="$(kc GET '/admin/realms/sovico/groups?search=readonly')"
group_id="$(printf '%s' "$groups" | jq -r '[.[] | select(.name == "readonly")][0].id // empty')"
if [[ -z "$group_id" ]]; then
  kc POST /admin/realms/sovico/groups '{"name":"readonly"}' >/dev/null
  group_id="$(kc GET '/admin/realms/sovico/groups?search=readonly' | jq -er '[.[] | select(.name == "readonly")][0].id')"
fi
users="$(kc GET '/admin/realms/sovico/users?username=vault-user&exact=true')"
user_id="$(printf '%s' "$users" | jq -er '[.[] | select(.username == "vault-user")][0].id')"
kc PUT "/admin/realms/sovico/users/$user_id/groups/$group_id" >/dev/null

mounts="$(vault GET /v1/sys/mounts)"
if ! printf '%s' "$mounts" | jq -e 'has("transit/")' >/dev/null; then
  vault POST /v1/sys/mounts/transit '{"type":"transit"}' >/dev/null
fi
key_status="$(curl -sS --cacert "$vault_ca" -o /dev/null -w '%{http_code}' -H "X-Vault-Token: $vault_token" "$vault_url/v1/transit/keys/rustfs")"
if [[ "$key_status" == 404 ]]; then
  vault POST /v1/transit/keys/rustfs '{"type":"aes256-gcm96","exportable":false,"deletion_allowed":false}' >/dev/null
elif [[ "$key_status" != 200 ]]; then
  echo "Cannot inspect Vault Transit key (HTTP $key_status)." >&2; exit 1
fi
policy='path "transit/encrypt/rustfs" { capabilities = ["update"] }
path "transit/decrypt/rustfs" { capabilities = ["update"] }
path "transit/keys" { capabilities = ["list"] }
path "transit/keys/rustfs" { capabilities = ["read"] }'
policy_payload="$(jq -cn --arg policy "$policy" '{policy:$policy}')"
vault PUT /v1/sys/policies/acl/rustfs-transit "$policy_payload" >/dev/null

existing_secret="$(kubectl -n rustfs get secret rustfs-credentials -o json 2>/dev/null || true)"
if [[ -n "$existing_secret" ]]; then
  access_key="$(printf '%s' "$existing_secret" | jq -er '.data.RUSTFS_ACCESS_KEY | @base64d')"
  secret_key="$(printf '%s' "$existing_secret" | jq -er '.data.RUSTFS_SECRET_KEY | @base64d')"
  kms_token="$(printf '%s' "$existing_secret" | jq -r '.data.RUSTFS_KMS_VAULT_TOKEN // empty | @base64d')"
else
  access_key="$(openssl rand -hex 16 | tr '[:lower:]' '[:upper:]')"
  secret_key="$(openssl rand -hex 32)"
  kms_token=""
fi
if [[ -n "$kms_token" ]]; then
  token_status="$(curl -sS --cacert "$vault_ca" -o /dev/null -w '%{http_code}' -H "X-Vault-Token: $kms_token" "$vault_url/v1/auth/token/lookup-self")"
  [[ "$token_status" == 200 ]] || kms_token=""
fi
if [[ -z "$kms_token" ]]; then
  kms_token="$(vault POST /v1/auth/token/create-orphan '{"policies":["rustfs-transit"],"no_default_policy":true,"ttl":"720h","renewable":true,"display_name":"rustfs-kms"}' | jq -er '.auth.client_token')"
fi
umask 077
mkdir -p "$repo_root/k8s/storage"
printf 'RUSTFS_ACCESS_KEY=%s\nRUSTFS_SECRET_KEY=%s\nRUSTFS_IDENTITY_OPENID_CLIENT_SECRET=%s\nRUSTFS_KMS_ENABLE=true\nRUSTFS_KMS_BACKEND=vault-transit\nRUSTFS_KMS_VAULT_ADDRESS=https://vault.vault.svc.cluster.local:8200\nRUSTFS_KMS_VAULT_TOKEN=%s\nRUSTFS_KMS_VAULT_MOUNT_PATH=transit\nRUSTFS_KMS_DEFAULT_KEY_ID=rustfs\n' \
  "$access_key" "$secret_key" "$client_secret" "$kms_token" > "$credentials_file"
chmod 600 "$credentials_file"
unset vault_token admin_token client_secret kms_token secret_key
kubectl create namespace rustfs --dry-run=client -o yaml | kubectl apply -f -
kubectl -n rustfs create secret generic rustfs-credentials --from-env-file="$credentials_file" --dry-run=client -o yaml | kubectl apply -f -
ca_bundle="$(mktemp)"
trap 'rm -f "$ca_bundle"; cleanup' EXIT
cat "$vault_ca" "$keycloak_ca" > "$ca_bundle"
kubectl -n rustfs create secret generic rustfs-vault-ca --from-file=ca.crt="$ca_bundle" --dry-run=client -o yaml | kubectl apply -f -
echo "RustFS bootstrap complete: sovico OIDC client, readonly group, Vault Transit key and restricted token."
echo "Credentials are in $credentials_file (mode 600); back them up securely."
echo "The Vault KMS token has a 720-hour TTL; renew or rotate it before expiry."
echo "Run ./scripts/setup_storage.sh."
