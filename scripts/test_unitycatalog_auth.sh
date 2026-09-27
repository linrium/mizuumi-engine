#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace="${UNITYCATALOG_NAMESPACE:-tower}"
release="${UNITYCATALOG_RELEASE:-unitycatalog}"
keycloak_ca="${KEYCLOAK_CA:-$repo_root/k8s/auth/tls/ca.crt}"
unitycatalog_ca="${UNITYCATALOG_CA:-$repo_root/k8s/auth/tls/ca.crt}"
keycloak_url="${KEYCLOAK_URL:-https://auth.mizuumi.test}"
unitycatalog_url="${UNITYCATALOG_URL:-https://unitycatalog.mizuumi.test}"
suffix="$(openssl rand -hex 4)"
human_client="uc-auth-human-$suffix"
service_client="uc-auth-service-$suffix"
username="uc-auth-user-$suffix"
email="$username@example.test"
password="$(openssl rand -base64 24 | tr -d '\n')"
admin_token=""
human_client_uuid=""
service_client_uuid=""
user_id=""
response_dir="$(mktemp -d "${TMPDIR:-/tmp}/unitycatalog-auth-test.XXXXXX")"

cleanup() {
  if [[ -n "$admin_token" ]]; then
    [[ -z "$user_id" ]] || kc DELETE "/admin/realms/sovico/users/$user_id" >/dev/null 2>&1 || true
    [[ -z "$human_client_uuid" ]] || kc DELETE "/admin/realms/sovico/clients/$human_client_uuid" >/dev/null 2>&1 || true
    [[ -z "$service_client_uuid" ]] || kc DELETE "/admin/realms/sovico/clients/$service_client_uuid" >/dev/null 2>&1 || true
  fi
  rm -rf "$response_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

usage() {
  cat <<'EOF'
Usage: ./scripts/test_unitycatalog_auth.sh

Creates temporary Keycloak human and service principals, verifies 401/403
enforcement and principal mapping against the deployed Unity Catalog server,
tests its generated bootstrap token, then removes all temporary principals.

Environment overrides:
  UNITYCATALOG_NAMESPACE   Kubernetes namespace (default: tower)
  UNITYCATALOG_RELEASE     Helm release name (default: unitycatalog)
  KEYCLOAK_URL             Keycloak URL (default: https://auth.mizuumi.test)
  KEYCLOAK_CA              Keycloak CA path (default: k8s/auth/tls/ca.crt)
  UNITYCATALOG_URL         API URL (default: https://unitycatalog.mizuumi.test)
  UNITYCATALOG_CA          API CA path (default: k8s/auth/tls/ca.crt)
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
  *) echo "Refusing auth test on non-local context: $context" >&2; exit 1 ;;
esac

server_deployment="$(kubectl -n "$namespace" get deployment \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}')"
admin_user="$(kubectl -n auth get deployment keycloak -o json | jq -er '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_BOOTSTRAP_ADMIN_USERNAME") | .value')"
admin_password="$(kubectl -n auth get secret keycloak-credentials -o json | jq -er '.data.ADMIN_PASSWORD | @base64d')"

for attempt in {1..60}; do
  if curl -fsS --cacert "$keycloak_ca" "$keycloak_url/realms/sovico/.well-known/openid-configuration" >/dev/null 2>&1 &&
     curl -fsS --cacert "$unitycatalog_ca" "$unitycatalog_url/health/livez" >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then echo "Keycloak or Unity Catalog is unavailable." >&2; exit 1; fi
  sleep 1
done

status="$(printf '%s' "$admin_password" | curl -sS --cacert "$keycloak_ca" -o "$response_dir/admin-token.json" -w '%{http_code}' -X POST \
  --data-urlencode 'password@-' --data-urlencode "username=$admin_user" \
  --data-urlencode 'client_id=admin-cli' --data-urlencode 'grant_type=password' \
  "$keycloak_url/realms/master/protocol/openid-connect/token")"
[[ "$status" == 200 ]] || {
  echo "Keycloak admin token request failed (HTTP $status): $(<"$response_dir/admin-token.json")" >&2
  exit 1
}
admin_token="$(jq -er '.access_token' "$response_dir/admin-token.json")"
unset admin_password

kc() {
  local method="$1" path="$2" status
  if [[ $# -eq 3 ]]; then
    status="$(printf '%s' "$3" | curl -sS --cacert "$keycloak_ca" -o "$response_dir/keycloak-response.json" -w '%{http_code}' -X "$method" \
      -H "Authorization: Bearer $admin_token" -H 'Content-Type: application/json' \
      --data-binary @- "$keycloak_url$path")"
  else
    status="$(curl -sS --cacert "$keycloak_ca" -o "$response_dir/keycloak-response.json" -w '%{http_code}' -X "$method" \
      -H "Authorization: Bearer $admin_token" "$keycloak_url$path")"
  fi
  [[ "$status" =~ ^2[0-9][0-9]$ ]] || {
    echo "Keycloak $method $path failed (HTTP $status): $(<"$response_dir/keycloak-response.json")" >&2
    return 1
  }
  cat "$response_dir/keycloak-response.json"
}

resource_clients="$(kc GET '/admin/realms/sovico/clients?clientId=unitycatalog')"
resource_client_uuid="$(printf '%s' "$resource_clients" | jq -er '[.[] | select(.clientId == "unitycatalog")][0].id')"
reader_role="$(kc GET "/admin/realms/sovico/clients/$resource_client_uuid/roles/unitycatalog-reader")"
writer_role="$(kc GET "/admin/realms/sovico/clients/$resource_client_uuid/roles/unitycatalog-writer")"

human_payload="$(jq -cn --arg id "$human_client" \
  '{clientId:$id,enabled:true,protocol:"openid-connect",publicClient:false,clientAuthenticatorType:"client-secret",standardFlowEnabled:false,directAccessGrantsEnabled:true,serviceAccountsEnabled:false}')"
kc POST /admin/realms/sovico/clients "$human_payload" >/dev/null
human_client_uuid="$(kc GET "/admin/realms/sovico/clients?clientId=$human_client" | jq -er --arg id "$human_client" '[.[] | select(.clientId == $id)][0].id')"
human_secret="$(kc GET "/admin/realms/sovico/clients/$human_client_uuid/client-secret" | jq -er '.value')"

service_payload="$(jq -cn --arg id "$service_client" \
  '{clientId:$id,enabled:true,protocol:"openid-connect",publicClient:false,clientAuthenticatorType:"client-secret",standardFlowEnabled:false,directAccessGrantsEnabled:false,serviceAccountsEnabled:true}')"
kc POST /admin/realms/sovico/clients "$service_payload" >/dev/null
service_client_uuid="$(kc GET "/admin/realms/sovico/clients?clientId=$service_client" | jq -er --arg id "$service_client" '[.[] | select(.clientId == $id)][0].id')"
service_secret="$(kc GET "/admin/realms/sovico/clients/$service_client_uuid/client-secret" | jq -er '.value')"

audience_payload='{"name":"unitycatalog-audience","protocol":"openid-connect","protocolMapper":"oidc-audience-mapper","consentRequired":false,"config":{"included.client.audience":"unitycatalog","access.token.claim":"true","id.token.claim":"false"}}'
kc POST "/admin/realms/sovico/clients/$human_client_uuid/protocol-mappers/models" "$audience_payload" >/dev/null
kc POST "/admin/realms/sovico/clients/$service_client_uuid/protocol-mappers/models" "$audience_payload" >/dev/null

user_payload="$(jq -cn --arg username "$username" --arg email "$email" --arg password "$password" \
  '{username:$username,firstName:"Unity Catalog",lastName:"Auth Test",email:$email,emailVerified:true,enabled:true,requiredActions:[],credentials:[{type:"password",value:$password,temporary:false}]}')"
kc POST /admin/realms/sovico/users "$user_payload" >/dev/null
user_id="$(kc GET "/admin/realms/sovico/users?username=$username&exact=true" | jq -er --arg username "$username" '[.[] | select(.username == $username)][0].id')"
kc POST "/admin/realms/sovico/users/$user_id/role-mappings/clients/$resource_client_uuid" "[$reader_role]" >/dev/null
service_user_id="$(kc GET "/admin/realms/sovico/clients/$service_client_uuid/service-account-user" | jq -er '.id')"
kc POST "/admin/realms/sovico/users/$service_user_id/role-mappings/clients/$resource_client_uuid" "[$writer_role]" >/dev/null

status="$(printf '%s' "$password" | curl -sS --cacert "$keycloak_ca" -o "$response_dir/human-token.json" -w '%{http_code}' -X POST \
  --data-urlencode 'password@-' --data-urlencode "username=$username" \
  --data-urlencode "client_id=$human_client" --data-urlencode "client_secret=$human_secret" \
  --data-urlencode 'grant_type=password' \
  "$keycloak_url/realms/sovico/protocol/openid-connect/token")"
[[ "$status" == 200 ]] || {
  echo "Keycloak human token request failed (HTTP $status): $(<"$response_dir/human-token.json")" >&2
  exit 1
}
human_token="$(jq -er '.access_token' "$response_dir/human-token.json")"
status="$(curl -sS --cacert "$keycloak_ca" -o "$response_dir/service-token.json" -w '%{http_code}' -X POST \
  --data-urlencode "client_id=$service_client" --data-urlencode "client_secret=$service_secret" \
  --data-urlencode 'grant_type=client_credentials' \
  "$keycloak_url/realms/sovico/protocol/openid-connect/token")"
[[ "$status" == 200 ]] || {
  echo "Keycloak service token request failed (HTTP $status): $(<"$response_dir/service-token.json")" >&2
  exit 1
}
service_token="$(jq -er '.access_token' "$response_dir/service-token.json")"
unset password human_secret service_secret

status="$(curl -sS --cacert "$unitycatalog_ca" -o "$response_dir/unauthorized.json" -w '%{http_code}' "$unitycatalog_url/api/auth/me")"
[[ "$status" == 401 ]] || { echo "Expected anonymous request to return 401, got $status." >&2; exit 1; }

human_response="$(curl -fsS --cacert "$unitycatalog_ca" -H "Authorization: Bearer $human_token" "$unitycatalog_url/api/auth/me")"
jq -e --arg email "$email" '.kind == "human" and .id == $email' <<<"$human_response" >/dev/null || {
  echo "Human principal mapping failed: $human_response" >&2; exit 1;
}
status="$(curl -sS --cacert "$unitycatalog_ca" -o "$response_dir/human-write.json" -w '%{http_code}' -X POST \
  -H "Authorization: Bearer $human_token" "$unitycatalog_url/api/hello")"
[[ "$status" == 403 ]] || { echo "Expected reader write to return 403, got $status." >&2; exit 1; }

service_response="$(curl -fsS --cacert "$unitycatalog_ca" -H "Authorization: Bearer $service_token" "$unitycatalog_url/api/auth/me")"
jq -e --arg id "$service_client" '.kind == "service" and .id == $id' <<<"$service_response" >/dev/null || {
  echo "Service principal mapping failed: $service_response" >&2; exit 1;
}
status="$(printf '{}' | curl -sS --cacert "$unitycatalog_ca" -o "$response_dir/service-write.json" -w '%{http_code}' -X POST \
  -H "Authorization: Bearer $service_token" -H 'Content-Type: application/json' \
  --data-binary @- "$unitycatalog_url/api/2.1/unity-catalog/catalogs")"
[[ "$status" != 401 && "$status" != 403 ]] || {
  echo "Writer service principal was rejected with HTTP $status." >&2; exit 1;
}

bootstrap_token="$(kubectl -n "$namespace" exec deployment/"$server_deployment" -c server -- \
  sh -c 'cat /var/run/unitycatalog/bootstrap-token')"
bootstrap_response="$(curl -fsS --cacert "$unitycatalog_ca" -H "Authorization: Bearer $bootstrap_token" "$unitycatalog_url/api/auth/me")"
jq -e '.kind == "bootstrap" and .id == "bootstrap"' <<<"$bootstrap_response" >/dev/null || {
  echo "Bootstrap principal mapping failed: $bootstrap_response" >&2; exit 1;
}

echo "Unity Catalog authentication passed: anonymous denied, human reader authorized, service writer authorized, bootstrap admin authorized."
