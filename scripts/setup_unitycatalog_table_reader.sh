#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace="${UNITYCATALOG_NAMESPACE:-tower}"
release="${UNITYCATALOG_RELEASE:-unitycatalog}"
realm="${KEYCLOAK_REALM:-sovico}"
keycloak_url="${KEYCLOAK_URL:-https://auth.mizuumi.test}"
unitycatalog_url="${UNITYCATALOG_URL:-https://unitycatalog.mizuumi.test}"
keycloak_ca="${KEYCLOAK_CA:-$repo_root/k8s/auth/tls/ca.crt}"
unitycatalog_ca="${UNITYCATALOG_CA:-$repo_root/k8s/auth/tls/ca.crt}"
username="${UNITYCATALOG_READER_USERNAME:-uc-table-reader}"
email="${UNITYCATALOG_READER_EMAIL:-$username@example.test}"
client_id="${UNITYCATALOG_READER_CLIENT_ID:-unitycatalog-table-reader}"
catalog="${UNITYCATALOG_CATALOG:-unity}"
schema="${UNITYCATALOG_SCHEMA:-gold}"
table="${UNITYCATALOG_TABLE:-daily_product_sales}"
full_table_name="$catalog.$schema.$table"
response_dir="$(mktemp -d "${TMPDIR:-/tmp}/unitycatalog-table-reader.XXXXXX")"
admin_token=""

cleanup() {
  unset admin_token
  rm -rf "$response_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

usage() {
  cat <<'EOF'
Usage: UNITYCATALOG_READER_PASSWORD='...' ./scripts/setup_unitycatalog_table_reader.sh

Creates/updates a dedicated non-admin Keycloak reader account and grants it
SELECT on the minimal Spark example's gold.daily_product_sales table. The
catalog and schema receive only USE CATALOG and USE SCHEMA respectively.
The table must already exist (run packages/spark/examples/minimal/main.py first).

Environment overrides:
  UNITYCATALOG_READER_USERNAME       (default: uc-table-reader)
  UNITYCATALOG_READER_EMAIL          (default: <username>@example.test)
  UNITYCATALOG_READER_PASSWORD       Required for first-time account creation;
                                     on reruns the existing password is unchanged.
  UNITYCATALOG_READER_CLIENT_ID      (default: unitycatalog-table-reader)
  UNITYCATALOG_CATALOG               (default: unity)
  UNITYCATALOG_SCHEMA                (default: gold)
  UNITYCATALOG_TABLE                 (default: daily_product_sales)
  KEYCLOAK_URL, KEYCLOAK_CA, KEYCLOAK_REALM
  UNITYCATALOG_URL, UNITYCATALOG_CA, UNITYCATALOG_NAMESPACE, UNITYCATALOG_RELEASE

The newly generated OAuth client secret is printed once at the end. Treat it
as a credential. This script only supports local Kubernetes contexts.
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
[[ -f "$keycloak_ca" ]] || { echo "Missing Keycloak CA: $keycloak_ca" >&2; exit 1; }
[[ -f "$unitycatalog_ca" ]] || { echo "Missing Unity Catalog CA: $unitycatalog_ca" >&2; exit 1; }
keycloak_url="${keycloak_url%/}"
unitycatalog_url="${unitycatalog_url%/}"

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing to modify Keycloak/Unity Catalog on non-local context: $context" >&2; exit 1 ;;
esac

server_deployment="$(kubectl -n "$namespace" get deployment \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}')"
[[ -n "$server_deployment" ]] || { echo "Unity Catalog server deployment not found." >&2; exit 1; }
admin_user="$(kubectl -n auth get deployment keycloak -o json | jq -er '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_BOOTSTRAP_ADMIN_USERNAME") | .value')"
admin_password="$(kubectl -n auth get secret keycloak-credentials -o json | jq -er '.data.ADMIN_PASSWORD | @base64d')"

if ! curl -fsS --cacert "$keycloak_ca" "$keycloak_url/realms/$realm/.well-known/openid-configuration" >/dev/null; then
  echo "Keycloak is unavailable at $keycloak_url." >&2; exit 1
fi
if ! curl -fsS --cacert "$unitycatalog_ca" "$unitycatalog_url/health/livez" >/dev/null; then
  echo "Unity Catalog is unavailable at $unitycatalog_url." >&2; exit 1
fi

status="$(printf '%s' "$admin_password" | curl -sS --cacert "$keycloak_ca" -o "$response_dir/admin-token.json" -w '%{http_code}' -X POST \
  --data-urlencode 'password@-' --data-urlencode "username=$admin_user" \
  --data-urlencode 'client_id=admin-cli' --data-urlencode 'grant_type=password' \
  "$keycloak_url/realms/master/protocol/openid-connect/token")"
[[ "$status" == 200 ]] || { echo "Keycloak admin token request failed (HTTP $status)." >&2; exit 1; }
admin_token="$(jq -er '.access_token' "$response_dir/admin-token.json")"
unset admin_password

kc() {
  local method="$1" path="$2" status
  if [[ $# -eq 3 ]]; then
    status="$(printf '%s' "$3" | curl -sS --cacert "$keycloak_ca" -o "$response_dir/keycloak-response.json" -w '%{http_code}' -X "$method" \
      -H "Authorization: Bearer $admin_token" -H 'Content-Type: application/json' --data-binary @- "$keycloak_url$path")"
  else
    status="$(curl -sS --cacert "$keycloak_ca" -o "$response_dir/keycloak-response.json" -w '%{http_code}' -X "$method" \
      -H "Authorization: Bearer $admin_token" "$keycloak_url$path")"
  fi
  [[ "$status" =~ ^2[0-9][0-9]$ ]] || { echo "Keycloak $method $path failed (HTTP $status)." >&2; return 1; }
  cat "$response_dir/keycloak-response.json"
}

upsert_client() {
  local id="$1" payload="$2" clients uuid current
  clients="$(kc GET "/admin/realms/$realm/clients?clientId=$id")"
  uuid="$(printf '%s' "$clients" | jq -r --arg id "$id" '[.[] | select(.clientId == $id)][0].id // empty')"
  if [[ -z "$uuid" ]]; then
    kc POST "/admin/realms/$realm/clients" "$payload" >/dev/null
    uuid="$(kc GET "/admin/realms/$realm/clients?clientId=$id" | jq -er --arg id "$id" '[.[] | select(.clientId == $id)][0].id')"
  else
    current="$(kc GET "/admin/realms/$realm/clients/$uuid")"
    payload="$(printf '%s' "$payload" | jq -c --argjson current "$current" '$current * .')"
    kc PUT "/admin/realms/$realm/clients/$uuid" "$payload" >/dev/null
  fi
  printf '%s' "$uuid"
}

resource_clients="$(kc GET "/admin/realms/$realm/clients?clientId=unitycatalog")"
resource_uuid="$(printf '%s' "$resource_clients" | jq -er '[.[] | select(.clientId == "unitycatalog")][0].id')"
reader_role="$(kc GET "/admin/realms/$realm/clients/$resource_uuid/roles/unitycatalog-reader")"

client_payload="$(jq -cn --arg id "$client_id" \
  '{clientId:$id,enabled:true,protocol:"openid-connect",publicClient:false,clientAuthenticatorType:"client-secret",standardFlowEnabled:false,directAccessGrantsEnabled:true,serviceAccountsEnabled:false}')"
client_uuid="$(upsert_client "$client_id" "$client_payload")"
client_secret="$(kc GET "/admin/realms/$realm/clients/$client_uuid/client-secret" | jq -er '.value')"

audience_payload='{"name":"unitycatalog-audience","protocol":"openid-connect","protocolMapper":"oidc-audience-mapper","consentRequired":false,"config":{"included.client.audience":"unitycatalog","access.token.claim":"true","id.token.claim":"false"}}'
mappers="$(kc GET "/admin/realms/$realm/clients/$client_uuid/protocol-mappers/models")"
if ! printf '%s' "$mappers" | jq -e 'any(.[]; .name == "unitycatalog-audience")' >/dev/null; then
  kc POST "/admin/realms/$realm/clients/$client_uuid/protocol-mappers/models" "$audience_payload" >/dev/null
fi

users="$(kc GET "/admin/realms/$realm/users?username=$username&exact=true")"
user_id="$(printf '%s' "$users" | jq -r --arg username "$username" '[.[] | select(.username == $username)][0].id // empty')"
if [[ -z "$user_id" ]]; then
  [[ -n "${UNITYCATALOG_READER_PASSWORD:-}" ]] || {
    echo "Set UNITYCATALOG_READER_PASSWORD to create the new account." >&2; exit 2;
  }
  user_payload="$(jq -cn --arg username "$username" --arg email "$email" --arg password "$UNITYCATALOG_READER_PASSWORD" \
    '{username:$username,firstName:"Unity Catalog",lastName:"Table Reader",email:$email,emailVerified:true,enabled:true,requiredActions:[],credentials:[{type:"password",value:$password,temporary:false}]}')"
  kc POST "/admin/realms/$realm/users" "$user_payload" >/dev/null
  user_id="$(kc GET "/admin/realms/$realm/users?username=$username&exact=true" | jq -er --arg username "$username" '[.[] | select(.username == $username)][0].id')"
fi

# Ensure the user is a reader, and remove only conflicting Unity Catalog client roles.
kc DELETE "/admin/realms/$realm/users/$user_id/role-mappings/clients/$resource_uuid" \
  "[$(kc GET "/admin/realms/$realm/clients/$resource_uuid/roles/unitycatalog-admin"),$(kc GET "/admin/realms/$realm/clients/$resource_uuid/roles/unitycatalog-writer")]" >/dev/null
kc POST "/admin/realms/$realm/users/$user_id/role-mappings/clients/$resource_uuid" "[$reader_role]" >/dev/null

# The privileged bootstrap token is used only to confirm the target and apply the exact grants.
bootstrap_token="$(kubectl -n "$namespace" exec deployment/"$server_deployment" -c server -- \
  sh -c 'cat /var/run/unitycatalog/bootstrap-token')"
api() {
  local method="$1" path="$2" body="${3:-}" status
  if [[ -n "$body" ]]; then
    status="$(printf '%s' "$body" | curl -sS --cacert "$unitycatalog_ca" -o "$response_dir/unitycatalog-response.json" -w '%{http_code}' -X "$method" \
      -H "Authorization: Bearer $bootstrap_token" -H 'Content-Type: application/json' --data-binary @- "$unitycatalog_url$path")"
  else
    status="$(curl -sS --cacert "$unitycatalog_ca" -o "$response_dir/unitycatalog-response.json" -w '%{http_code}' -X "$method" \
      -H "Authorization: Bearer $bootstrap_token" "$unitycatalog_url$path")"
  fi
  [[ "$status" =~ ^2[0-9][0-9]$ ]] || {
    echo "Unity Catalog $method $path failed (HTTP $status): $(<"$response_dir/unitycatalog-response.json")" >&2
    return 1
  }
  cat "$response_dir/unitycatalog-response.json"
}

api GET "/api/2.1/unity-catalog/tables/$full_table_name" >/dev/null || {
  echo "Table $full_table_name does not exist. Run packages/spark/examples/minimal/main.py first." >&2
  exit 1
}

grant() {
  local securable="$1" name="$2" privilege="$3" body
  body="$(jq -cn --arg principal "$email" --arg privilege "$privilege" \
    '{changes:[{principal:$principal,add:[$privilege],remove:[]}]}')"
  api PATCH "/api/2.1/unity-catalog/permissions/$securable/$name" "$body" >/dev/null
}

grant catalog "$catalog" 'USE CATALOG'
grant schema "$catalog.$schema" 'USE SCHEMA'
grant table "$full_table_name" 'SELECT'

unset bootstrap_token
printf 'Created/verified non-admin reader: %s (%s)\n' "$username" "$email"
printf 'Granted: USE CATALOG on %s; USE SCHEMA on %s.%s; SELECT on %s\n' "$catalog" "$catalog" "$schema" "$full_table_name"
printf 'OAuth client ID: %s\nOAuth client secret (store securely): %s\n' "$client_id" "$client_secret"
printf 'Password-grant token endpoint: %s/realms/%s/protocol/openid-connect/token\n' "$keycloak_url" "$realm"
