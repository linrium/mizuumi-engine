#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace="${UNITYCATALOG_NAMESPACE:-tower}"
release="${UNITYCATALOG_RELEASE:-unitycatalog}"
catalog="${SPARK_UNITY_CATALOG:-unity}"
principal="${UNITYCATALOG_USER_EMAIL:-khaopad@mizuumi.test}"
storage_root="${SPARK_UNITY_STORAGE_ROOT:-s3://unitycatalog/spark}"
credential="${SPARK_UNITY_CREDENTIAL:-rustfs_unitycatalog}"
location="${SPARK_UNITY_EXTERNAL_LOCATION:-rustfs_spark}"
credentials_secret="${UNITYCATALOG_CREDENTIALS_SECRET:-unitycatalog-credentials}"
unitycatalog_url="${UNITYCATALOG_URL:-https://unitycatalog.mizuumi.test}"
unitycatalog_ca="${UNITYCATALOG_CA:-$repo_root/k8s/auth/tls/ca.crt}"
work_dir=""
auth_token=""

cleanup() {
  [[ -z "$work_dir" ]] || rm -rf "$work_dir"
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage: ./scripts/init_spark_catalog.sh

Idempotently creates the catalog, RustFS storage credential and external
location, bronze/silver/gold schemas, and grants used by the Spark example.

Environment overrides:
  UNITYCATALOG_NAMESPACE          Unity Catalog namespace (default: tower)
  UNITYCATALOG_RELEASE            Unity Catalog Helm release (default: unitycatalog)
  UNITYCATALOG_USER_EMAIL         Principal receiving access
  SPARK_UNITY_CATALOG             Catalog name (default: unity)
  SPARK_UNITY_STORAGE_ROOT        RustFS URI prefix (default: s3://unitycatalog/spark)
  SPARK_UNITY_CREDENTIAL          Storage credential name
  SPARK_UNITY_EXTERNAL_LOCATION   External location name
  UNITYCATALOG_URL                API URL (default: https://unitycatalog.mizuumi.test)
  UNITYCATALOG_CA                 API CA bundle (default: k8s/auth/tls/ca.crt)
EOF
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
  "") ;;
  *) usage >&2; exit 2 ;;
esac

for command in kubectl curl jq; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

deployment="$(kubectl -n "$namespace" get deployment \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || true)"
if [[ -z "$deployment" ]]; then
  echo "Unity Catalog is not installed. Run ./scripts/setup_unitycatalog.sh first." >&2
  exit 1
fi
[[ -f "$unitycatalog_ca" ]] || { echo "Missing Unity Catalog CA: $unitycatalog_ca" >&2; exit 1; }
kubectl -n "$namespace" rollout status deployment/"$deployment" --timeout=5m >/dev/null
kubectl -n "$namespace" get secret "$credentials_secret" >/dev/null 2>&1 || {
  echo "Missing Unity Catalog RustFS credentials: $namespace/$credentials_secret" >&2
  exit 1
}

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/spark-catalog-init.XXXXXX")"
unitycatalog_url="${unitycatalog_url%/}"
base_url="$unitycatalog_url/api/2.1/unity-catalog"
curl -fsS --cacert "$unitycatalog_ca" "$unitycatalog_url/health/readyz" >/dev/null || {
  echo "Unity Catalog is unavailable at $unitycatalog_url." >&2
  exit 1
}
auth_token="$(kubectl -n "$namespace" exec deployment/"$deployment" -c server -- \
  sh -c 'cat /var/run/unitycatalog/bootstrap-token')"
[[ -n "$auth_token" ]] || { echo "Unity Catalog bootstrap token is empty." >&2; exit 1; }

ensure_resource() {
  local label="$1" get_path="$2" post_path="$3" payload="$4" status
  status="$(curl -sS --cacert "$unitycatalog_ca" -o "$work_dir/response.json" \
    -w '%{http_code}' -H "Authorization: Bearer $auth_token" "$base_url$get_path")"
  case "$status" in
    200) return 0 ;;
    404) ;;
    *) echo "Failed to inspect $label (HTTP $status): $(<"$work_dir/response.json")" >&2; return 1 ;;
  esac

  status="$(printf '%s' "$payload" | curl -sS --cacert "$unitycatalog_ca" \
    -o "$work_dir/response.json" -w '%{http_code}' -X POST \
    -H "Authorization: Bearer $auth_token" \
    -H 'Content-Type: application/json' --data-binary @- "$base_url$post_path")"
  case "$status" in
    200|201) ;;
    *) echo "Failed to create $label (HTTP $status): $(<"$work_dir/response.json")" >&2; return 1 ;;
  esac
}

catalog_payload="$(jq -cn --arg name "$catalog" --arg root "$storage_root" \
  '{name:$name,storage_root:$root,comment:"Spark example catalog"}')"
ensure_resource "catalog $catalog" "/catalogs/$catalog" /catalogs "$catalog_payload"

s3_access_key="$(kubectl -n "$namespace" get secret "$credentials_secret" \
  -o jsonpath='{.data.S3_ACCESS_KEY}' | base64 --decode)"
s3_secret_key="$(kubectl -n "$namespace" get secret "$credentials_secret" \
  -o jsonpath='{.data.S3_SECRET_KEY}' | base64 --decode)"
credential_payload="$(jq -cn --arg name "$credential" --arg access "$s3_access_key" \
  --arg secret "$s3_secret_key" \
  '{name:$name,purpose:"STORAGE",comment:"RustFS credential for Spark",rustfs_service_account:{access_key:$access,secret_key:$secret}}')"
unset s3_access_key s3_secret_key
ensure_resource "credential $credential" "/credentials/$credential" /credentials "$credential_payload"
unset credential_payload

location_payload="$(jq -cn --arg name "$location" --arg url "$storage_root" \
  --arg credential "$credential" \
  '{name:$name,url:$url,credential_name:$credential,comment:"RustFS location for Spark tables"}')"
ensure_resource "external location $location" "/external-locations/$location" \
  /external-locations "$location_payload"

for schema in bronze silver gold; do
  schema_payload="$(jq -cn --arg catalog "$catalog" --arg name "$schema" \
    --arg root "$storage_root/$schema" \
    '{catalog_name:$catalog,name:$name,storage_root:$root,comment:("Spark medallion " + $name + " layer")}')"
  ensure_resource "schema $catalog.$schema" "/schemas/$catalog.$schema" /schemas "$schema_payload"
done

run_uc_admin() {
  local output
  if output="$(kubectl -n "$namespace" exec deployment/"$deployment" -c server -- \
    bin/uc --server http://127.0.0.1:8080 --auth-token "$auth_token" "$@" 2>&1)"; then
    return 0
  fi
  if [[ "$output" == *ALREADY_EXISTS* || "$output" == *"already exists"* ]]; then
    return 0
  fi
  printf '%s\n' "$output" >&2
  return 1
}

run_uc_admin permission create --securable_type catalog --name "$catalog" \
  --privilege 'USE CATALOG' --principal "$principal"
for schema in bronze silver gold; do
  run_uc_admin permission create --securable_type schema --name "$catalog.$schema" \
    --privilege 'USE SCHEMA' --principal "$principal"
  run_uc_admin permission create --securable_type schema --name "$catalog.$schema" \
    --privilege 'CREATE TABLE' --principal "$principal"
done
for privilege in 'READ FILES' 'WRITE FILES' 'CREATE EXTERNAL TABLE'; do
  run_uc_admin permission create --securable_type external_location --name "$location" \
    --privilege "$privilege" --principal "$principal"
done

echo "Unity Catalog is ready: $catalog.{bronze,silver,gold} at $storage_root"
