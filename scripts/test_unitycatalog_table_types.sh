#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
api_url="${UNITYCATALOG_URL:-https://unitycatalog.mizuumi.test}"
ca="${UNITYCATALOG_CA:-$repo_root/k8s/auth/tls/ca.crt}"
namespace="${UNITYCATALOG_NAMESPACE:-tower}"
release="${UNITYCATALOG_RELEASE:-unitycatalog}"
storage_root="${UNITYCATALOG_TEST_STORAGE_ROOT:-}"
token="${UNITYCATALOG_TOKEN:-}"
response_dir="$(mktemp -d "${TMPDIR:-/tmp}/unitycatalog-table-types.XXXXXX")"
catalog_created=0
catalog=""

cleanup() {
  if [[ "$catalog_created" == 1 ]]; then
    curl -fsS "${tls_args[@]}" -X DELETE -H "Authorization: Bearer $token" \
      "$api_url/api/2.1/unity-catalog/catalogs/$catalog?force=true" >/dev/null || \
      echo "Could not remove test catalog $catalog; remove it manually." >&2
  fi
  rm -rf "$response_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

usage() {
  cat <<'EOF'
Usage: UNITYCATALOG_TEST_STORAGE_ROOT=s3://bucket/test-root ./scripts/test_unitycatalog_table_types.sh

Runs an API smoke test for /staging-tables, all six table_type values, and
GET/POST /delta/preview/commits. Creates a uniquely named catalog and deletes
it on exit. No objects are written to the storage root.

Set UNITYCATALOG_TOKEN to an admin bearer token, or leave it unset to read the
local Kubernetes server's bootstrap token. Optional: UNITYCATALOG_URL,
UNITYCATALOG_CA, UNITYCATALOG_NAMESPACE, UNITYCATALOG_RELEASE.
EOF
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
  "") ;;
  *) usage >&2; exit 2 ;;
esac

for command in curl jq openssl tr; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done
[[ "$storage_root" == s3://* || "$storage_root" == gs://* || "$storage_root" == abfs://* || "$storage_root" == abfss://* ]] || {
  echo "Set UNITYCATALOG_TEST_STORAGE_ROOT to a cloud storage URL." >&2; exit 2;
}
[[ "$storage_root" != *"__unitystorage"* ]] || {
  echo "Storage root must not include __unitystorage." >&2; exit 2;
}
api_url="${api_url%/}"
storage_root="${storage_root%/}"
tls_args=()
if [[ "$api_url" == https://* ]]; then
  [[ -f "$ca" ]] || { echo "Missing Unity Catalog CA: $ca" >&2; exit 1; }
  tls_args=(--cacert "$ca")
fi

if [[ -z "$token" ]]; then
  command -v kubectl >/dev/null || { echo "kubectl is needed when UNITYCATALOG_TOKEN is unset." >&2; exit 1; }
  context="$(kubectl config current-context)"
  case "$context" in
    kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
    *) echo "Refusing to use bootstrap token on non-local context: $context" >&2; exit 1 ;;
  esac
  server_deployment="$(kubectl -n "$namespace" get deployment \
    -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
    -o jsonpath='{.items[0].metadata.name}')"
  [[ -n "$server_deployment" ]] || { echo "Unity Catalog deployment not found." >&2; exit 1; }
  token="$(kubectl -n "$namespace" exec deployment/"$server_deployment" -c server -- \
    sh -c 'cat /var/run/unitycatalog/bootstrap-token')"
fi

request() {
  local method="$1" path="$2" status
  if [[ $# -eq 3 ]]; then
    status="$(printf '%s' "$3" | curl -sS "${tls_args[@]}" -o "$response_dir/response.json" -w '%{http_code}' \
      -X "$method" -H "Authorization: Bearer $token" -H 'Content-Type: application/json' \
      --data-binary @- "$api_url/api/2.1/unity-catalog$path")"
  else
    status="$(curl -sS "${tls_args[@]}" -o "$response_dir/response.json" -w '%{http_code}' \
      -X "$method" -H "Authorization: Bearer $token" "$api_url/api/2.1/unity-catalog$path")"
  fi
  [[ "$status" == 200 ]] || {
    echo "$method $path failed (HTTP $status): $(<"$response_dir/response.json")" >&2
    return 1
  }
  cat "$response_dir/response.json"
}

suffix="$(openssl rand -hex 4)"
catalog="uc_api_$suffix"
schema="checks"
managed="managed_$suffix"

catalog_body="$(jq -cn --arg name "$catalog" --arg root "$storage_root" '{name:$name,storage_root:$root}')"
request POST /catalogs "$catalog_body" >/dev/null
catalog_created=1
request POST /schemas "$(jq -cn --arg catalog "$catalog" --arg name "$schema" '{catalog_name:$catalog,name:$name}')" >/dev/null

staging="$(request POST /staging-tables \
  "$(jq -cn --arg catalog "$catalog" --arg schema "$schema" --arg name "$managed" \
    '{catalog_name:$catalog,schema_name:$schema,name:$name}')")"
staging_id="$(printf '%s' "$staging" | jq -er '.id')"
staging_location="$(printf '%s' "$staging" | jq -er '.staging_location')"

managed_body="$(jq -cn --arg catalog "$catalog" --arg schema "$schema" --arg name "$managed" \
  --arg location "$staging_location" \
  '{catalog_name:$catalog,schema_name:$schema,name:$name,table_type:"MANAGED",data_source_format:"DELTA",storage_location:$location,columns:[{name:"id",type_name:"LONG",position:0}]}')"
managed_response="$(request POST /tables "$managed_body")"
printf '%s' "$managed_response" | jq -e --arg id "$staging_id" --arg location "$staging_location" \
  '.table_type == "MANAGED" and .table_id == $id and .storage_location == $location' >/dev/null

commit_file="00000000000000000001.$suffix.json"
commit_body="$(jq -cn --arg id "$staging_id" --arg uri "$staging_location" --arg file "$commit_file" \
  '{table_id:$id,table_uri:$uri,commit_info:{version:1,timestamp:1,file_name:$file,file_size:1,file_modification_timestamp:1}}')"
request POST /delta/preview/commits "$commit_body" >/dev/null
request POST /delta/preview/commits "$commit_body" >/dev/null
get_commits_body="$(jq -cn --arg id "$staging_id" --arg uri "$staging_location" \
  '{table_id:$id,table_uri:$uri,start_version:0}')"
commits="$(request GET /delta/preview/commits "$get_commits_body")"
printf '%s' "$commits" | jq -e --arg file "$commit_file" \
  '.latest_table_version == 1 and (.commits | length) == 1 and .commits[0].file_name == $file' >/dev/null
request POST /delta/preview/commits \
  "$(jq -cn --arg id "$staging_id" --arg uri "$staging_location" \
    '{table_id:$id,table_uri:$uri,latest_backfilled_version:1}')" >/dev/null
commits="$(request GET /delta/preview/commits "$get_commits_body")"
printf '%s' "$commits" | jq -e '.latest_table_version == 1 and (.commits | length) == 0' >/dev/null

external_name="external_$suffix"
external_location="$storage_root/external/$suffix"
external_response="$(request POST /tables \
  "$(jq -cn --arg catalog "$catalog" --arg schema "$schema" --arg name "$external_name" \
    --arg location "$external_location" \
    '{catalog_name:$catalog,schema_name:$schema,name:$name,table_type:"EXTERNAL",data_source_format:"DELTA",storage_location:$location,columns:[]}' )")"
printf '%s' "$external_response" | jq -e '.table_type == "EXTERNAL"' >/dev/null

for table_type in STREAMING_TABLE MATERIALIZED_VIEW METRIC_VIEW VIEW; do
  name="$(printf '%s' "$table_type" | tr '[:upper:]' '[:lower:]')_$suffix"
  definition='SELECT 1 AS id'
  [[ "$table_type" != METRIC_VIEW ]] || definition=$'version: 1.0\nsource: SELECT 1 AS id'
  view_response="$(request POST /tables \
    "$(jq -cn --arg catalog "$catalog" --arg schema "$schema" --arg name "$name" \
      --arg type "$table_type" --arg definition "$definition" \
      '{catalog_name:$catalog,schema_name:$schema,name:$name,table_type:$type,columns:[],view_definition:$definition}')")"
  printf '%s' "$view_response" | jq -e --arg type "$table_type" --arg definition "$definition" \
    '.table_type == $type and .view_definition == $definition' >/dev/null
  table_info="$(request GET "/tables/$catalog.$schema.$name")"
  printf '%s' "$table_info" | jq -e --arg type "$table_type" '.table_type == $type' >/dev/null
done

echo "Unity Catalog staging tables, six table types, and Delta commit API passed."
