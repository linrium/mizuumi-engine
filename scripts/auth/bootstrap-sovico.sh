#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
default_user_password=123
khaopad_password_file="$repo_root/k8s/auth/sovico-khaopad-password"
khaopad_email="${SOVICO_USER_EMAIL:-khaopad@mizuumi.test}"
khaopad_first_name="${SOVICO_USER_FIRST_NAME:-Khao}"
khaopad_last_name="${SOVICO_USER_LAST_NAME:-Pad}"
khaosoi_password_file="$repo_root/k8s/auth/sovico-khaosoi-password"
khaosoi_email="${SOVICO_KHAOSOI_EMAIL:-khaosoi@mizuumi.test}"
khaosoi_first_name="${SOVICO_KHAOSOI_FIRST_NAME:-Khao}"
khaosoi_last_name="${SOVICO_KHAOSOI_LAST_NAME:-Soi}"
vault_init_file="$repo_root/k8s/vault/init.json"
vault_ca="$repo_root/k8s/vault/tls/ca.crt"
keycloak_ca="$repo_root/k8s/auth/tls/ca.crt"

for command in kubectl curl jq; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing to bootstrap local-only OIDC on non-local context: $context" >&2; exit 1 ;;
esac

kubectl -n auth get deployment keycloak >/dev/null
kubectl -n vault get pod vault-0 >/dev/null

hostname="$(kubectl -n auth get deployment keycloak -o json | jq -r '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_HOSTNAME") | .value')"
if [[ "$hostname" != "https://auth.mizuumi.test" ]]; then
  echo "Keycloak must use the local https://auth.mizuumi.test hostname for this bootstrap." >&2
  exit 1
fi
if ! kubectl -n auth get service keycloak-gateway >/dev/null 2>&1; then
  echo "Run ./scripts/auth/setup.sh to install the Keycloak gateway first." >&2
  exit 1
fi

admin_user="$(kubectl -n auth get deployment keycloak -o json | jq -er '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_BOOTSTRAP_ADMIN_USERNAME") | .value')"
admin_password="$(kubectl -n auth get secret keycloak-credentials -o json | jq -er '.data.ADMIN_PASSWORD | @base64d')"

keycloak_url=https://127.0.0.1:18080
vault_url=https://127.0.0.1:18200
issuer=https://auth.mizuumi.test/realms/sovico
discovery_url=$issuer
vault_browser_origin=https://vault.mizuumi.test
ui_callback=$vault_browser_origin/ui/vault/auth/oidc/oidc/callback
cli_callback=http://localhost:8250/oidc/callback
if [[ ! -f "$vault_ca" ]]; then
  echo "Missing Vault CA $vault_ca; run ./scripts/vault/setup.sh first." >&2
  exit 1
fi
if [[ ! -f "$keycloak_ca" ]]; then
  echo "Missing Keycloak CA $keycloak_ca; run ./scripts/auth/setup.sh first." >&2
  exit 1
fi
keycloak_curl() { curl --cacert "$keycloak_ca" "$@"; }
if [[ -n "${VAULT_TOKEN:-}" ]]; then
  vault_token="$VAULT_TOKEN"
elif [[ -f "$vault_init_file" ]]; then
  vault_token="$(jq -er '.root_token' "$vault_init_file")"
else
  echo "Set VAULT_TOKEN or run ./scripts/vault/init.sh first." >&2
  exit 1
fi
keycloak_pid=""
vault_pid=""

cleanup() {
  if [[ -n "$keycloak_pid" ]]; then
    kill "$keycloak_pid" 2>/dev/null || true
    wait "$keycloak_pid" 2>/dev/null || true
  fi
  if [[ -n "$vault_pid" ]]; then
    kill "$vault_pid" 2>/dev/null || true
    wait "$vault_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

kubectl -n auth port-forward --address 127.0.0.1 service/keycloak 18080:8080 >/dev/null 2>&1 &
keycloak_pid=$!
kubectl -n vault port-forward --address 127.0.0.1 pod/vault-0 18200:8200 >/dev/null 2>&1 &
vault_pid=$!

for attempt in {1..60}; do
  if keycloak_curl -fsS "$keycloak_url/realms/master/.well-known/openid-configuration" >/dev/null 2>&1 &&
     curl -fsS --cacert "$vault_ca" "$vault_url/v1/sys/health" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$keycloak_pid" 2>/dev/null || ! kill -0 "$vault_pid" 2>/dev/null; then
    echo "A temporary port-forward failed to start." >&2
    exit 1
  fi
  if [[ "$attempt" -eq 60 ]]; then
    echo "Timed out waiting for Keycloak and Vault." >&2
    exit 1
  fi
  sleep 1
done

get_admin_token() {
  printf '%s' "$admin_password" | keycloak_curl -fsS -X POST \
    --data-urlencode 'password@-' \
    --data-urlencode "username=$admin_user" \
    --data-urlencode 'client_id=admin-cli' \
    --data-urlencode 'grant_type=password' \
    "$keycloak_url/realms/master/protocol/openid-connect/token" | jq -er '.access_token'
}

admin_token="$(get_admin_token)"

keycloak_request() {
  local method="$1" path="$2"
  if [[ $# -eq 3 ]]; then
    printf '%s' "$3" | keycloak_curl -fsS -X "$method" \
      -H "Authorization: Bearer $admin_token" -H 'Content-Type: application/json' \
      --data-binary @- "$keycloak_url$path"
  else
    keycloak_curl -fsS -X "$method" -H "Authorization: Bearer $admin_token" "$keycloak_url$path"
  fi
}

vault_request() {
  local method="$1" path="$2"
  if [[ $# -eq 3 ]]; then
    printf '%s' "$3" | curl -fsS --cacert "$vault_ca" -X "$method" \
      -H "X-Vault-Token: $vault_token" -H 'Content-Type: application/json' \
      --data-binary @- "$vault_url$path"
  else
    curl -fsS --cacert "$vault_ca" -X "$method" -H "X-Vault-Token: $vault_token" "$vault_url$path"
  fi
}

realm_status="$(keycloak_curl -sS -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $admin_token" "$keycloak_url/admin/realms/sovico")"
case "$realm_status" in
  200) echo 'Keycloak realm sovico already exists.' ;;
  404)
    keycloak_request POST /admin/realms '{"realm":"sovico","enabled":true}' >/dev/null
    echo 'Created Keycloak realm sovico.'
    ;;
  *) echo "Could not inspect Keycloak realm (HTTP $realm_status)." >&2; exit 1 ;;
esac

clients="$(keycloak_request GET '/admin/realms/sovico/clients?clientId=vault')"
client_uuid="$(printf '%s' "$clients" | jq -r '[.[] | select(.clientId == "vault")][0].id // empty')"
client_payload="$(jq -cn --arg ui "$ui_callback" --arg cli "$cli_callback" --arg origin "$vault_browser_origin" \
  '{clientId:"vault",enabled:true,protocol:"openid-connect",publicClient:false,clientAuthenticatorType:"client-secret",standardFlowEnabled:true,directAccessGrantsEnabled:false,serviceAccountsEnabled:false,redirectUris:[$ui,$cli],webOrigins:[$origin]}')"
if [[ -z "$client_uuid" ]]; then
  keycloak_request POST /admin/realms/sovico/clients "$client_payload" >/dev/null
  clients="$(keycloak_request GET '/admin/realms/sovico/clients?clientId=vault')"
  client_uuid="$(printf '%s' "$clients" | jq -er '[.[] | select(.clientId == "vault")][0].id')"
  echo 'Created Keycloak client vault.'
else
  current_client="$(keycloak_request GET "/admin/realms/sovico/clients/$client_uuid")"
  client_payload="$(printf '%s' "$current_client" | jq -c --arg ui "$ui_callback" --arg cli "$cli_callback" --arg origin "$vault_browser_origin" \
    '.enabled=true | .publicClient=false | .clientAuthenticatorType="client-secret" | .standardFlowEnabled=true | .directAccessGrantsEnabled=false | .serviceAccountsEnabled=false | .redirectUris=[$ui,$cli] | .webOrigins=[$origin]')"
  keycloak_request PUT "/admin/realms/sovico/clients/$client_uuid" "$client_payload" >/dev/null
  echo 'Updated Keycloak client vault.'
fi

client_secret="$(keycloak_request GET "/admin/realms/sovico/clients/$client_uuid/client-secret" | jq -r '.value // empty')"
if [[ -z "$client_secret" ]]; then
  client_secret="$(keycloak_request POST "/admin/realms/sovico/clients/$client_uuid/client-secret" '{}' | jq -er '.value')"
fi

ensure_user() {
  local username="$1" email="$2" first_name="$3" last_name="$4" password="$5" password_file="$6"
  local users user_id current_user user_payload password_payload

  [[ -n "$password" ]] || { echo "$username password cannot be empty." >&2; exit 1; }
  umask 077
  printf '%s\n' "$password" > "$password_file"
  echo "Wrote $username password to $password_file (mode 600)."

  users="$(keycloak_request GET "/admin/realms/sovico/users?username=$username&exact=true")"
  user_id="$(printf '%s' "$users" | jq -r --arg username "$username" '[.[] | select(.username == $username)][0].id // empty')"
  if [[ -n "$user_id" ]]; then
    current_user="$(keycloak_request GET "/admin/realms/sovico/users/$user_id")"
    user_payload="$(printf '%s' "$current_user" | jq -c \
      --arg email "$email" \
      --arg first_name "$first_name" \
      --arg last_name "$last_name" \
      '.enabled=true | .email=$email | .emailVerified=true | .firstName=$first_name | .lastName=$last_name')"
    keycloak_request PUT "/admin/realms/sovico/users/$user_id" "$user_payload" >/dev/null
    password_payload="$(jq -cn --arg password "$password" '{type:"password",value:$password,temporary:false}')"
    keycloak_request PUT "/admin/realms/sovico/users/$user_id/reset-password" "$password_payload" >/dev/null
    echo "Keycloak user $username already exists; profile and password updated."
  else
    user_payload="$(jq -cn \
      --arg username "$username" \
      --arg email "$email" \
      --arg first_name "$first_name" \
      --arg last_name "$last_name" \
      --arg password "$password" \
      '{username:$username,enabled:true,email:$email,emailVerified:true,firstName:$first_name,lastName:$last_name,credentials:[{type:"password",value:$password,temporary:false}]}')"
    keycloak_request POST /admin/realms/sovico/users "$user_payload" >/dev/null
    echo "Created Keycloak user $username."
  fi
}

ensure_user \
  khaopad "$khaopad_email" "$khaopad_first_name" "$khaopad_last_name" \
  "${SOVICO_USER_PASSWORD:-$default_user_password}" "$khaopad_password_file"
ensure_user \
  khaosoi "$khaosoi_email" "$khaosoi_first_name" "$khaosoi_last_name" \
  "${SOVICO_KHAOSOI_PASSWORD:-$default_user_password}" "$khaosoi_password_file"
unset admin_password admin_token

auth_mounts="$(vault_request GET /v1/sys/auth)"
if ! printf '%s' "$auth_mounts" | jq -e 'has("oidc/")' >/dev/null; then
  vault_request POST /v1/sys/auth/oidc '{"type":"oidc"}' >/dev/null
  echo 'Enabled Vault OIDC auth.'
fi

config_payload="$(printf '%s' "$client_secret" | jq -Rs --rawfile ca "$keycloak_ca" \
  --arg discovery "$discovery_url" --arg issuer "$issuer" \
  '{oidc_discovery_url:$discovery,oidc_discovery_ca_pem:$ca,oidc_client_id:"vault",oidc_client_secret:.,bound_issuer:$issuer,default_role:"sovico"}')"
vault_request POST /v1/auth/oidc/config "$config_payload" >/dev/null
unset client_secret config_payload

role_payload="$(jq -cn --arg ui "$ui_callback" --arg cli "$cli_callback" \
  '{role_type:"oidc",user_claim:"sub",bound_audiences:["vault"],allowed_redirect_uris:[$ui,$cli],token_policies:["default"]}')"
vault_request POST /v1/auth/oidc/role/sovico "$role_payload" >/dev/null

auth_url_payload="$(jq -cn --arg redirect "$ui_callback" '{role:"sovico",redirect_uri:$redirect}')"
auth_url="$(vault_request POST /v1/auth/oidc/oidc/auth_url "$auth_url_payload" | jq -er '.data.auth_url')"
if [[ "$auth_url" != "$issuer/protocol/openid-connect/auth"* ]]; then
  echo 'Vault returned an unexpected Keycloak authorization URL.' >&2
  exit 1
fi

echo 'Vault OIDC is configured for Keycloak realm sovico.'
echo 'Run ./scripts/forward.sh, then open https://vault.mizuumi.test/ui/ and choose OIDC login.'
echo 'Sign in as khaopad or khaosoi with the password set during bootstrap (default: 123).'
