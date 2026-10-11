#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
realm="${KEYCLOAK_REALM:-sovico}"
keycloak_ca="$repo_root/k8s/auth/tls/ca.crt"
keycloak_url=https://127.0.0.1:18080

usage() {
  cat <<'EOF'
Usage: ./scripts/auth/bootstrap-organizations.sh [organization=group[=user] ...]

Enable Keycloak Organizations for a realm, then create organization-scoped
top-level groups and optionally assign a user. When no mappings are supplied,
these defaults are used:

  hdbank=hdbankGrow=khaosoi
  vietjet=vietjetGrow=khaopad

Set KEYCLOAK_REALM to target a realm other than sovico.
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

if [[ $# -gt 0 ]]; then
  organization_groups=("$@")
else
  organization_groups=(
    hdbank=hdbankGrow=khaosoi
    vietjet=vietjetGrow=khaopad
  )
fi

for command in kubectl curl jq; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

if [[ ! "$realm" =~ ^[A-Za-z0-9._-]+$ ]]; then
  echo "KEYCLOAK_REALM contains unsupported characters: $realm" >&2
  exit 1
fi

for mapping in "${organization_groups[@]}"; do
  if [[ "$mapping" != *=* ]]; then
    echo "Invalid mapping '$mapping'; expected organization=group or organization=group=user." >&2
    exit 1
  fi
  organization_name="${mapping%%=*}"
  mapping_remainder="${mapping#*=}"
  group_name="${mapping_remainder%%=*}"
  user_name=""
  if [[ "$mapping_remainder" == *=* ]]; then
    user_name="${mapping_remainder#*=}"
  fi
  if [[ -z "$organization_name" || -z "$group_name" ]]; then
    echo "Invalid mapping '$mapping'; organization and group must be non-empty." >&2
    exit 1
  fi
  if [[ "$mapping_remainder" == *=* && -z "$user_name" ]]; then
    echo "Invalid mapping '$mapping'; user must be non-empty when provided." >&2
    exit 1
  fi
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing to bootstrap local-only organizations on non-local context: $context" >&2; exit 1 ;;
esac

kubectl -n auth get deployment keycloak >/dev/null

hostname="$(kubectl -n auth get deployment keycloak -o json | jq -r '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_HOSTNAME") | .value')"
if [[ "$hostname" != "https://auth.mizuumi.test" ]]; then
  echo "Keycloak must use the local https://auth.mizuumi.test hostname for this bootstrap." >&2
  exit 1
fi
if [[ ! -f "$keycloak_ca" ]]; then
  echo "Missing Keycloak CA $keycloak_ca; run ./scripts/auth/setup.sh first." >&2
  exit 1
fi

admin_user="$(kubectl -n auth get deployment keycloak -o json | jq -er '.spec.template.spec.containers[] | select(.name == "keycloak") | .env[] | select(.name == "KC_BOOTSTRAP_ADMIN_USERNAME") | .value')"
admin_password="$(kubectl -n auth get secret keycloak-credentials -o json | jq -er '.data.ADMIN_PASSWORD | @base64d')"

keycloak_curl() { curl --cacert "$keycloak_ca" "$@"; }

keycloak_pid=""
cleanup() {
  if [[ -n "$keycloak_pid" ]]; then
    kill "$keycloak_pid" 2>/dev/null || true
    wait "$keycloak_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

kubectl -n auth port-forward --address 127.0.0.1 service/keycloak 18080:8080 >/dev/null 2>&1 &
keycloak_pid=$!

for attempt in {1..60}; do
  if keycloak_curl -fsS "$keycloak_url/realms/master/.well-known/openid-configuration" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$keycloak_pid" 2>/dev/null; then
    echo "The temporary Keycloak port-forward failed to start." >&2
    exit 1
  fi
  if [[ "$attempt" -eq 60 ]]; then
    echo "Timed out waiting for Keycloak." >&2
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
unset admin_password

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

realm_status="$(keycloak_curl -sS -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $admin_token" "$keycloak_url/admin/realms/$realm")"
if [[ "$realm_status" != 200 ]]; then
  if [[ "$realm_status" == 404 ]]; then
    echo "Keycloak realm $realm does not exist; create it first (for sovico, run ./scripts/auth/bootstrap-sovico.sh)." >&2
  else
    echo "Could not inspect Keycloak realm $realm (HTTP $realm_status)." >&2
  fi
  exit 1
fi

realm_payload="$(keycloak_request GET "/admin/realms/$realm")"
if [[ "$(printf '%s' "$realm_payload" | jq -r '.organizationsEnabled // false')" == true ]]; then
  echo "Organizations are already enabled for realm $realm."
else
  realm_payload="$(printf '%s' "$realm_payload" | jq -c '.organizationsEnabled = true')"
  keycloak_request PUT "/admin/realms/$realm" "$realm_payload" >/dev/null
  echo "Enabled Organizations for realm $realm."
fi

find_organization() {
  local organization_name="$1" encoded_name organizations
  encoded_name="$(printf '%s' "$organization_name" | jq -sRr @uri)"
  organizations="$(keycloak_request GET "/admin/realms/$realm/organizations?search=$encoded_name&exact=true&briefRepresentation=true&max=100")"
  printf '%s' "$organizations" | jq -c --arg name "$organization_name" \
    '[.[] | select(.alias == $name or .name == $name)][0] // empty'
}

ensure_organization() {
  local organization_name="$1" organization organization_id organization_payload
  organization="$(find_organization "$organization_name")"
  if [[ -z "$organization" ]]; then
    organization_payload="$(jq -cn --arg name "$organization_name" '{name:$name,alias:$name,enabled:true}')"
    keycloak_request POST "/admin/realms/$realm/organizations" "$organization_payload" >/dev/null
    organization="$(find_organization "$organization_name")"
    if [[ -z "$organization" ]]; then
      echo "Created organization $organization_name, but could not retrieve it." >&2
      exit 1
    fi
    echo "Created organization $organization_name." >&2
  else
    echo "Organization $organization_name already exists." >&2
  fi

  organization_id="$(printf '%s' "$organization" | jq -er '.id')"
  if [[ "$(printf '%s' "$organization" | jq -r '.enabled // false')" != true ]]; then
    organization_payload="$(keycloak_request GET "/admin/realms/$realm/organizations/$organization_id" | jq -c '.enabled = true')"
    keycloak_request PUT "/admin/realms/$realm/organizations/$organization_id" "$organization_payload" >/dev/null
    echo "Enabled organization $organization_name." >&2
  fi
  printf '%s' "$organization_id"
}

ensure_group() {
  local organization_name="$1" organization_id="$2" group_name="$3" groups group group_id group_payload
  groups="$(keycloak_request GET "/admin/realms/$realm/organizations/$organization_id/groups?briefRepresentation=true&max=100")"
  group="$(printf '%s' "$groups" | jq -c --arg name "$group_name" '[.[] | select(.name == $name)][0] // empty')"
  if [[ -z "$group" ]]; then
    group_payload="$(jq -cn --arg name "$group_name" '{name:$name}')"
    keycloak_request POST "/admin/realms/$realm/organizations/$organization_id/groups" "$group_payload" >/dev/null
    groups="$(keycloak_request GET "/admin/realms/$realm/organizations/$organization_id/groups?briefRepresentation=true&max=100")"
    group="$(printf '%s' "$groups" | jq -c --arg name "$group_name" '[.[] | select(.name == $name)][0] // empty')"
    if [[ -z "$group" ]]; then
      echo "Created organization group $organization_name/$group_name, but could not retrieve it." >&2
      exit 1
    fi
    echo "Created organization group $organization_name/$group_name." >&2
  else
    echo "Organization group $organization_name/$group_name already exists." >&2
  fi

  group_id="$(printf '%s' "$group" | jq -er '.id')"
  printf '%s' "$group_id"
}

find_user_id() {
  local user_name="$1" encoded_name users
  encoded_name="$(printf '%s' "$user_name" | jq -sRr @uri)"
  users="$(keycloak_request GET "/admin/realms/$realm/users?username=$encoded_name&exact=true")"
  printf '%s' "$users" | jq -r --arg username "$user_name" \
    '[.[] | select(.username == $username)][0].id // empty'
}

ensure_user_assignment() {
  local organization_name="$1" organization_id="$2" group_name="$3" group_id="$4" user_name="$5"
  local user_id member_status member_groups member_payload

  user_id="$(find_user_id "$user_name")"
  if [[ -z "$user_id" ]]; then
    echo "Keycloak user $user_name does not exist in realm $realm; run ./scripts/auth/bootstrap-sovico.sh first." >&2
    exit 1
  fi

  member_status="$(keycloak_curl -sS -o /dev/null -w '%{http_code}' \
    -H "Authorization: Bearer $admin_token" \
    "$keycloak_url/admin/realms/$realm/organizations/$organization_id/members/$user_id")"
  case "$member_status" in
    200) echo "User $user_name is already a member of organization $organization_name." ;;
    404)
      member_payload="$(jq -cn --arg id "$user_id" '$id')"
      keycloak_request POST "/admin/realms/$realm/organizations/$organization_id/members" "$member_payload" >/dev/null
      echo "Added user $user_name to organization $organization_name."
      ;;
    *)
      echo "Could not inspect membership for $user_name in $organization_name (HTTP $member_status)." >&2
      exit 1
      ;;
  esac

  member_groups="$(keycloak_request GET "/admin/realms/$realm/organizations/$organization_id/members/$user_id/groups?briefRepresentation=true&max=100")"
  if printf '%s' "$member_groups" | jq -e --arg id "$group_id" '.[] | select(.id == $id)' >/dev/null; then
    echo "User $user_name is already assigned to $organization_name/$group_name."
    return
  fi

  keycloak_request PUT "/admin/realms/$realm/organizations/$organization_id/groups/$group_id/members/$user_id" >/dev/null
  echo "Assigned user $user_name to $organization_name/$group_name."
}

for mapping in "${organization_groups[@]}"; do
  organization_name="${mapping%%=*}"
  mapping_remainder="${mapping#*=}"
  group_name="${mapping_remainder%%=*}"
  user_name=""
  if [[ "$mapping_remainder" == *=* ]]; then
    user_name="${mapping_remainder#*=}"
  fi
  organization_id="$(ensure_organization "$organization_name")"
  group_id="$(ensure_group "$organization_name" "$organization_id" "$group_name")"
  if [[ -n "$user_name" ]]; then
    ensure_user_assignment "$organization_name" "$organization_id" "$group_name" "$group_id" "$user_name"
  fi
done

unset admin_token
echo "Keycloak organizations are configured for realm $realm."
