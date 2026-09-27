#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
keycloak_ca="$repo_root/k8s/auth/tls/ca.crt"
studio_env="$repo_root/packages/studio/.env.local"
realm="${KEYCLOAK_REALM:-sovico}"
client_id="${STUDIO_KEYCLOAK_CLIENT_ID:-studio}"
studio_url="${STUDIO_URL:-http://localhost:3000}"
public_keycloak_url="${KEYCLOAK_PUBLIC_URL:-https://auth.mizuumi.test}"
keycloak_url="https://127.0.0.1:18080"
rotate_secret=0

usage() {
  cat <<'EOF'
Usage: ./scripts/bootstrap_studio.sh [--rotate-secret]

Create or update the Mizuumi Studio OIDC client in the sovico Keycloak realm
and write its credentials to packages/studio/.env.local.

  --rotate-secret  Generate a new client secret instead of reusing the current one.
  -h, --help       Show this help.

Environment overrides:
  KEYCLOAK_REALM              Default: sovico
  KEYCLOAK_PUBLIC_URL         Default: https://auth.mizuumi.test
  STUDIO_KEYCLOAK_CLIENT_ID   Default: studio
  STUDIO_URL                  Default: http://localhost:3000
EOF
}

while (( $# > 0 )); do
  case "$1" in
    --rotate-secret) rotate_secret=1 ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

for command in kubectl curl jq awk; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

case "$client_id" in
  *[!A-Za-z0-9._-]*|'')
    echo "STUDIO_KEYCLOAK_CLIENT_ID must contain only letters, numbers, dots, underscores, or hyphens." >&2
    exit 1
    ;;
esac

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing local bootstrap on non-local context: $context" >&2; exit 1 ;;
esac

[[ -f "$keycloak_ca" ]] || {
  echo "Missing Keycloak CA: $keycloak_ca" >&2
  echo "Run ./scripts/setup_auth.sh and ./scripts/bootstrap_sovico.sh first." >&2
  exit 1
}

admin_user="$(kubectl -n auth get deployment keycloak -o json | jq -er '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_BOOTSTRAP_ADMIN_USERNAME") | .value')"
admin_password="$(kubectl -n auth get secret keycloak-credentials -o json | jq -er '.data.ADMIN_PASSWORD | @base64d')"
keycloak_pid=""
env_tmp=""

cleanup() {
  [[ -z "$keycloak_pid" ]] || { kill "$keycloak_pid" 2>/dev/null || true; wait "$keycloak_pid" 2>/dev/null || true; }
  [[ -z "$env_tmp" || ! -f "$env_tmp" ]] || rm -f "$env_tmp"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

kubectl -n auth port-forward --address 127.0.0.1 service/keycloak 18080:8080 >/dev/null 2>&1 &
keycloak_pid=$!

for attempt in {1..60}; do
  if curl -fsS --cacert "$keycloak_ca" \
    "$keycloak_url/realms/$realm/.well-known/openid-configuration" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$keycloak_pid" 2>/dev/null; then
    echo "Keycloak port-forward stopped unexpectedly; is port 18080 already in use?" >&2
    exit 1
  fi
  if [[ "$attempt" -eq 60 ]]; then
    echo "Keycloak realm $realm is unavailable." >&2
    echo "Run ./scripts/bootstrap_sovico.sh first." >&2
    exit 1
  fi
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
    curl -fsS --cacert "$keycloak_ca" -X "$method" \
      -H "Authorization: Bearer $admin_token" "$keycloak_url$path"
  fi
}

studio_url="${studio_url%/}"
public_keycloak_url="${public_keycloak_url%/}"
callback_url="$studio_url/api/auth/callback/keycloak"
post_logout_url="$studio_url/"
issuer="$public_keycloak_url/realms/$realm"

clients="$(kc GET "/admin/realms/$realm/clients?clientId=$client_id")"
client_uuid="$(printf '%s' "$clients" | jq -r --arg id "$client_id" '[.[] | select(.clientId == $id)][0].id // empty')"

if [[ -z "$client_uuid" ]]; then
  client_payload="$(jq -cn \
    --arg id "$client_id" --arg callback "$callback_url" --arg origin "$studio_url" --arg post_logout "$post_logout_url" \
    '{clientId:$id,name:"Mizuumi Studio",enabled:true,protocol:"openid-connect",publicClient:false,clientAuthenticatorType:"client-secret",standardFlowEnabled:true,directAccessGrantsEnabled:false,serviceAccountsEnabled:false,redirectUris:[$callback],webOrigins:[$origin],attributes:{"pkce.code.challenge.method":"S256","post.logout.redirect.uris":$post_logout}}')"
  kc POST "/admin/realms/$realm/clients" "$client_payload" >/dev/null
  client_uuid="$(kc GET "/admin/realms/$realm/clients?clientId=$client_id" | jq -er --arg id "$client_id" '[.[] | select(.clientId == $id)][0].id')"
  echo "Created Keycloak client $realm/$client_id."
else
  current_client="$(kc GET "/admin/realms/$realm/clients/$client_uuid")"
  client_payload="$(printf '%s' "$current_client" | jq -c \
    --arg id "$client_id" --arg callback "$callback_url" --arg origin "$studio_url" --arg post_logout "$post_logout_url" \
    '.clientId=$id | .name="Mizuumi Studio" | .enabled=true | .protocol="openid-connect" | .publicClient=false | .clientAuthenticatorType="client-secret" | .standardFlowEnabled=true | .directAccessGrantsEnabled=false | .serviceAccountsEnabled=false | .redirectUris=[$callback] | .webOrigins=[$origin] | .attributes=(.attributes // {}) | .attributes["pkce.code.challenge.method"]="S256" | .attributes["post.logout.redirect.uris"]=$post_logout')"
  kc PUT "/admin/realms/$realm/clients/$client_uuid" "$client_payload" >/dev/null
  echo "Updated Keycloak client $realm/$client_id."
fi

# Unity Catalog validates the audience on incoming user access tokens. Add its
# audience to tokens issued for Studio so the user's token can be forwarded to
# the API without substituting a service token.
audience_payload='{"name":"unitycatalog-audience","protocol":"openid-connect","protocolMapper":"oidc-audience-mapper","consentRequired":false,"config":{"included.client.audience":"unitycatalog","access.token.claim":"true","id.token.claim":"false"}}'
audience_mappers="$(kc GET "/admin/realms/$realm/clients/$client_uuid/protocol-mappers/models")"
if ! printf '%s' "$audience_mappers" | jq -e 'any(.[]; .name == "unitycatalog-audience")' >/dev/null; then
  kc POST "/admin/realms/$realm/clients/$client_uuid/protocol-mappers/models" "$audience_payload" >/dev/null
  echo "Added the Unity Catalog audience to Studio access tokens."
fi

if (( rotate_secret )); then
  client_secret="$(kc POST "/admin/realms/$realm/clients/$client_uuid/client-secret" '{}' | jq -er '.value')"
  echo "Rotated the Studio client secret."
else
  client_secret="$(kc GET "/admin/realms/$realm/clients/$client_uuid/client-secret" | jq -r '.value // empty')"
  if [[ -z "$client_secret" ]]; then
    client_secret="$(kc POST "/admin/realms/$realm/clients/$client_uuid/client-secret" '{}' | jq -er '.value')"
    echo "Generated the Studio client secret."
  fi
fi

umask 077
env_tmp="$(mktemp "$repo_root/packages/studio/.env.local.XXXXXX")"
if [[ -f "$studio_env" ]]; then
  awk '!/^(KEYCLOAK_CLIENT_ID|KEYCLOAK_CLIENT_SECRET|KEYCLOAK_ISSUER)=/' "$studio_env" > "$env_tmp"
fi
printf 'KEYCLOAK_CLIENT_ID=%s\nKEYCLOAK_CLIENT_SECRET=%s\nKEYCLOAK_ISSUER=%s\n' \
  "$client_id" "$client_secret" "$issuer" >> "$env_tmp"
chmod 600 "$env_tmp"
mv "$env_tmp" "$studio_env"
env_tmp=""

unset admin_token client_secret
echo "Studio credentials written to $studio_env (mode 600)."
echo "Redirect URI: $callback_url"
echo "Post-logout redirect URI: $post_logout_url"
echo "Start Studio with the local CA available to Node.js:"
echo "  cd packages/studio && NODE_EXTRA_CA_CERTS=../../k8s/auth/tls/ca.crt npm run dev"
