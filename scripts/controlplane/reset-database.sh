#!/usr/bin/env bash
set -euo pipefail

namespace="${CONTROLPLANE_NAMESPACE:-tower}"
release="${CONTROLPLANE_RELEASE:-controlplane}"

usage() {
  cat <<'EOF'
Usage: ./scripts/controlplane/reset-database.sh

Drops all control-plane tables in the local Kubernetes PostgreSQL database, then restarts the
server so it recreates the current schema. This permanently deletes policy templates, access
requests, reviews, events, and fulfillment state.

Environment overrides:
  CONTROLPLANE_NAMESPACE  Kubernetes namespace (default: tower)
  CONTROLPLANE_RELEASE    Helm release name (default: controlplane)
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

command -v kubectl >/dev/null 2>&1 || { echo "Missing required command: kubectl" >&2; exit 1; }

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing to reset the control-plane database on non-local context: $context" >&2; exit 1 ;;
esac

postgres_pod="$(kubectl -n "$namespace" get pod \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=postgresql" \
  -o jsonpath='{.items[0].metadata.name}')"
server_deployment="$(kubectl -n "$namespace" get deployment \
  -l "app.kubernetes.io/instance=$release,app.kubernetes.io/component=server" \
  -o jsonpath='{.items[0].metadata.name}')"

[[ -n "$postgres_pod" ]] || { echo "Control-plane PostgreSQL pod not found." >&2; exit 1; }
[[ -n "$server_deployment" ]] || { echo "Control-plane server deployment not found." >&2; exit 1; }

echo "Resetting control-plane database in $context/$namespace..."
kubectl -n "$namespace" exec "$postgres_pod" -- sh -c \
  'psql --set ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" --command "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"'

kubectl -n "$namespace" rollout restart deployment/"$server_deployment"
kubectl -n "$namespace" rollout status deployment/"$server_deployment" --timeout=5m

echo "Control-plane database reset complete. Run ./scripts/controlplane/seed.sh to restore fixtures."
