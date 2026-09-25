#!/usr/bin/env bash
set -euo pipefail

namespace="${UNITYCATALOG_NAMESPACE:-tower}"
release="${UNITYCATALOG_RELEASE:-unitycatalog}"
catalog="${SPARK_UNITY_CATALOG:-unity}"
principal="${UNITYCATALOG_USER_EMAIL:-khaopad@mizuumi.test}"

usage() {
  cat <<'EOF'
Usage: ./scripts/init_spark_catalog.sh

Creates the bronze, silver, and gold schemas used by the Spark medallion
example and grants the configured Unity Catalog user access. The operation is
idempotent and uses the Unity Catalog CLI in the running server container.

Environment overrides:
  UNITYCATALOG_NAMESPACE   Unity Catalog namespace (default: tower)
  UNITYCATALOG_RELEASE     Unity Catalog Helm release (default: unitycatalog)
  UNITYCATALOG_USER_EMAIL  Principal receiving schema access
  SPARK_UNITY_CATALOG      Catalog name (default: unity)
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
  "")
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

command -v kubectl >/dev/null || { echo "Missing required command: kubectl" >&2; exit 1; }

deployment="$(kubectl -n "$namespace" get deployment \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || true)"
if [[ -z "$deployment" ]]; then
  echo "Unity Catalog is not installed. Run ./scripts/setup_unitycatalog.sh first." >&2
  exit 1
fi
kubectl -n "$namespace" rollout status deployment/"$deployment" --timeout=5m >/dev/null

run_uc_admin() {
  local output
  if output="$(kubectl -n "$namespace" exec deployment/"$deployment" -c server -- \
    /bin/bash -ec 'token="$(< /home/unitycatalog/etc/conf/token.txt)"; exec bin/uc --auth_token "$token" "$@"' -- "$@" 2>&1)"; then
    return 0
  fi
  if [[ "$output" == *ALREADY_EXISTS* || "$output" == *"already exists"* ]]; then
    return 0
  fi
  printf '%s\n' "$output" >&2
  return 1
}

run_uc_admin catalog get --name "$catalog" >/dev/null
for schema in bronze silver gold; do
  run_uc_admin schema create --catalog "$catalog" --name "$schema" \
    --comment "Spark medallion ${schema} layer"
  run_uc_admin permission create --securable_type schema \
    --name "$catalog.$schema" --privilege 'USE SCHEMA' --principal "$principal"
  run_uc_admin permission create --securable_type schema \
    --name "$catalog.$schema" --privilege 'CREATE TABLE' --principal "$principal"
done

for privilege in 'READ FILES' 'WRITE FILES' 'CREATE EXTERNAL TABLE'; do
  run_uc_admin permission create --securable_type external_location \
    --name rustfs_unitycatalog --privilege "$privilege" --principal "$principal"
done

echo "Unity Catalog schemas are ready: $catalog.{bronze,silver,gold}"
