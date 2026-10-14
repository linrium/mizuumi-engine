#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${CHRONICLE_NAMESPACE:-chronicle}"
release="${CHRONICLE_RELEASE:-chronicle}"
rustfs_ca="$repo_root/k8s/storage/tls/ca.crt"
assume_yes=0
rustfs_pid=""
temp_dir=""
deployments_restored=0
declare -a deployments=()
declare -a replica_counts=()

usage() {
  cat <<'EOF'
Usage: ./scripts/chronicle/reset.sh [--yes]

Resets a local Chronicle installation to an empty log. The script stops
Chronicle and its witnesses, permanently deletes the configured Chronicle
bucket prefix from RustFS, recreates the Tessera MySQL database, and starts
the deployments again.

Options:
  -y, --yes  Skip the destructive-action confirmation
  -h, --help Show this help

Environment overrides:
  CHRONICLE_NAMESPACE  Kubernetes namespace (default: chronicle)
  CHRONICLE_RELEASE    Helm release name (default: chronicle)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -y|--yes) assume_yes=1 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 1 ;;
  esac
  shift
done

for command in kubectl curl jq python3; do
  command -v "$command" >/dev/null 2>&1 || { echo "Missing required command: $command" >&2; exit 1; }
done
[[ -f "$rustfs_ca" ]] || { echo "Missing RustFS CA: $rustfs_ca" >&2; exit 1; }

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing to reset Chronicle on non-local context: $context" >&2; exit 1 ;;
esac

deployment_json="$(kubectl -n "$namespace" get deployment "$release" -o json)" || {
  echo "Chronicle deployment '$release' was not found in namespace '$namespace'." >&2
  exit 1
}

env_value() {
  local name="$1"
  jq -er --arg name "$name" '
    .spec.template.spec.containers[]
    | select(.name == "chronicle")
    | .env[]
    | select(.name == $name)
    | .value
  ' <<<"$deployment_json"
}

bucket="$(env_value CHRONICLE_AWS_S3_BUCKET)"
prefix="$(env_value CHRONICLE_AWS_S3_BUCKET_PREFIX 2>/dev/null || true)"
region="$(env_value CHRONICLE_AWS_S3_REGION)"
credentials_secret="$(jq -er '
  .spec.template.spec.containers[]
  | select(.name == "chronicle")
  | .env[]
  | select(.name == "CHRONICLE_AWS_S3_ACCESS_KEY")
  | .valueFrom.secretKeyRef.name
' <<<"$deployment_json")"

credentials_json="$(kubectl -n "$namespace" get secret "$credentials_secret" -o json)"
s3_access_key="$(jq -er '.data.S3_ACCESS_KEY | @base64d' <<<"$credentials_json")"
s3_secret_key="$(jq -er '.data.S3_SECRET_KEY | @base64d' <<<"$credentials_json")"
unset credentials_json deployment_json

mysql_pod="$(kubectl -n "$namespace" get pod \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=mysql" \
  -o jsonpath='{.items[0].metadata.name}')"
[[ -n "$mysql_pod" ]] || { echo "Chronicle MySQL pod was not found." >&2; exit 1; }

while IFS=$'\t' read -r deployment replica_count; do
  deployments+=("$deployment")
  replica_counts+=("$replica_count")
done < <(kubectl -n "$namespace" get deployment \
  -l "app.kubernetes.io/instance=$release" -o json | jq -r '
    .items[]
    | select(.metadata.labels["app.kubernetes.io/component"] == "chronicle"
        or .metadata.labels["app.kubernetes.io/component"] == "witness")
    | [.metadata.name, (.spec.replicas // 1)]
    | @tsv
  ')
[[ ${#deployments[@]} -gt 0 ]] || { echo "No Chronicle deployments were found." >&2; exit 1; }

target="s3://$bucket"
[[ -z "$prefix" ]] || target="$target/$prefix"
echo "Context: $context"
echo "Namespace: $namespace"
echo "RustFS target: $target"
echo "This permanently deletes the Chronicle log and its Tessera sequencing state."
if [[ "$assume_yes" != 1 ]]; then
  [[ -t 0 ]] || { echo "Confirmation requires a terminal; rerun with --yes." >&2; exit 1; }
  read -r -p "Type 'reset' to continue: " confirmation
  [[ "$confirmation" == reset ]] || { echo "Reset cancelled."; exit 0; }
fi

restore_deployments() {
  local index
  [[ "$deployments_restored" == 0 ]] || return 0
  deployments_restored=1
  for index in "${!deployments[@]}"; do
    kubectl -n "$namespace" scale deployment "${deployments[$index]}" \
      --replicas="${replica_counts[$index]}" >/dev/null || true
  done
}

cleanup() {
  local exit_code=$?
  [[ -z "$rustfs_pid" ]] || { kill "$rustfs_pid" 2>/dev/null || true; wait "$rustfs_pid" 2>/dev/null || true; }
  [[ -z "$temp_dir" ]] || rm -rf "$temp_dir"
  restore_deployments
  unset s3_access_key s3_secret_key
  return "$exit_code"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "Stopping Chronicle writers..."
for deployment in "${deployments[@]}"; do
  kubectl -n "$namespace" scale deployment "$deployment" --replicas=0 >/dev/null
done
for attempt in {1..60}; do
  writer_count="$(kubectl -n "$namespace" get pod -l "app.kubernetes.io/instance=$release" -o json | jq '[
    .items[]
    | select(.metadata.labels["app.kubernetes.io/component"] == "chronicle"
        or .metadata.labels["app.kubernetes.io/component"] == "witness")
  ] | length')"
  [[ "$writer_count" == 0 ]] && break
  [[ "$attempt" -lt 60 ]] || { echo "Timed out waiting for Chronicle writers to stop." >&2; exit 1; }
  sleep 1
done

temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/chronicle-reset.XXXXXX")"
rustfs_url=https://127.0.0.1:19000
kubectl -n storage port-forward --address 127.0.0.1 service/rustfs-svc 19000:9000 >/dev/null 2>&1 &
rustfs_pid=$!
for attempt in {1..60}; do
  if curl -fsS --cacert "$rustfs_ca" "$rustfs_url/minio/health/live" >/dev/null 2>&1; then
    break
  fi
  [[ "$attempt" -lt 60 ]] || { echo "RustFS is unavailable." >&2; exit 1; }
  sleep 1
done

echo "Deleting objects from $target..."
deleted=0
while :; do
  list_args=(--silent --show-error --cacert "$rustfs_ca" --get
    --aws-sigv4 "aws:amz:$region:s3" --user "$s3_access_key:$s3_secret_key"
    --data-urlencode 'list-type=2' --data-urlencode 'encoding-type=url')
  [[ -z "$prefix" ]] || list_args+=(--data-urlencode "prefix=$prefix")
  status="$(curl "${list_args[@]}" -o "$temp_dir/list.xml" -w '%{http_code}' "$rustfs_url/$bucket")"
  [[ "$status" == 200 ]] || {
    echo "RustFS object listing failed (HTTP $status): $(<"$temp_dir/list.xml")" >&2
    exit 1
  }

  python3 -c '
import sys
import urllib.parse
import xml.etree.ElementTree as ET

root = ET.parse(sys.argv[1]).getroot()
namespace = {"s3": root.tag.partition("}")[0].lstrip("{")} if "}" in root.tag else {}
path = "s3:Contents/s3:Key" if namespace else "Contents/Key"
with open(sys.argv[2], "w", encoding="utf-8") as output:
    for element in root.findall(path, namespace):
        output.write(urllib.parse.unquote(element.text or "") + "\n")
' "$temp_dir/list.xml" "$temp_dir/keys"

  [[ -s "$temp_dir/keys" ]] || break

  while IFS= read -r key; do
    [[ -n "$key" ]] || continue
    encoded_key="$(python3 -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe="/-_.~"))' "$key")"
    status="$(curl --silent --show-error --cacert "$rustfs_ca" -o "$temp_dir/delete" -w '%{http_code}' \
      -X DELETE --aws-sigv4 "aws:amz:$region:s3" --user "$s3_access_key:$s3_secret_key" \
      "$rustfs_url/$bucket/$encoded_key")"
    [[ "$status" == 204 || "$status" == 200 ]] || {
      echo "Failed to delete s3://$bucket/$key (HTTP $status): $(<"$temp_dir/delete")" >&2
      exit 1
    }
    deleted=$((deleted + 1))
  done < "$temp_dir/keys"

done

echo "Recreating the Tessera MySQL database..."
kubectl -n "$namespace" exec "$mysql_pod" -- sh -ec '
  MYSQL_PWD="$MYSQL_ROOT_PASSWORD" mysql --user=root --execute="
    DROP DATABASE IF EXISTS \`$MYSQL_DATABASE\`;
    CREATE DATABASE \`$MYSQL_DATABASE\`;
  "
'

echo "Starting Chronicle deployments..."
restore_deployments
for deployment in "${deployments[@]}"; do
  kubectl -n "$namespace" rollout status deployment/"$deployment" --timeout=5m
done

echo "Chronicle reset complete: deleted $deleted RustFS object(s) and recreated the Tessera database."
