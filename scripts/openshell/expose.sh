#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
openshell_namespace="${OPENSHELL_NAMESPACE:-openshell}"
openshell_release="${OPENSHELL_RELEASE:-openshell}"
auth_namespace=auth
auth_release=keycloak
gateway_deployment=keycloak-gateway
gateway_client_secret=openshell-gateway-client-tls

for command in kubectl helm jq openssl; do
  command -v "$command" >/dev/null || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done

kubectl -n "$openshell_namespace" get service "$openshell_release" >/dev/null
kubectl -n "$openshell_namespace" get secret "${openshell_release}-client-tls" >/dev/null
kubectl -n "$auth_namespace" get deployment "$gateway_deployment" >/dev/null

tls_dir="$repo_root/k8s/auth/tls"
if [[ ! -f "$tls_dir/ca.crt" || ! -f "$tls_dir/ca.key" ]]; then
  echo "Missing local gateway CA material. Run ./scripts/setup_auth.sh first." >&2
  exit 1
fi

"$repo_root/scripts/ensure_local_tls.sh" "$tls_dir" \
  "$repo_root/k8s/auth/gateway-tls.cnf" "$repo_root/k8s/auth/ca.cnf" \
  gateway.crt gateway.key

kubectl -n "$auth_namespace" create secret generic local-gateway-tls \
  --from-file=tls.crt="$tls_dir/gateway.crt" \
  --from-file=tls.key="$tls_dir/gateway.key" \
  --from-file=ca.crt="$tls_dir/ca.crt" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

client_tls_manifest="$(kubectl -n "$openshell_namespace" \
  get secret "${openshell_release}-client-tls" -o json | jq \
  --arg name "$gateway_client_secret" --arg namespace "$auth_namespace" '
    {
      apiVersion: "v1",
      kind: "Secret",
      metadata: {name: $name, namespace: $namespace},
      type: .type,
      data: .data
    }
  ')"
printf '%s\n' "$client_tls_manifest" | kubectl apply -f - >/dev/null

helm upgrade --install "$auth_release" "$repo_root/k8s/auth" \
  --namespace "$auth_namespace" \
  --set gateway.openshell.enabled=true \
  --set-string "gateway.openshell.namespace=$openshell_namespace" \
  --set-string "gateway.openshell.serviceName=$openshell_release" \
  --wait --timeout 10m
"$repo_root/scripts/configure_cluster_dns.sh"

# Secret contents are not part of the pod-template checksum. Restart so Caddy
# always reloads a renewed gateway or OpenShell certificate.
kubectl -n "$auth_namespace" rollout restart deployment/"$gateway_deployment" >/dev/null
kubectl -n "$auth_namespace" rollout status deployment/"$gateway_deployment" --timeout=5m

echo "OpenShell is available at https://openshell.mizuumi.test."
echo "If needed, run: sudo ./scripts/configure_workstation_dns.sh"
