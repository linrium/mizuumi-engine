#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
context="${OPENSHELL_KUBE_CONTEXT:-docker-desktop}"
node_container="${DOCKER_DESKTOP_K8S_NODE:-desktop-control-plane}"
cilium_version="${CILIUM_VERSION:-1.20.2}"
cilium_values="$repo_root/k8s/openshell/cilium-docker-desktop-values.yaml"

usage() {
  cat <<'EOF'
Usage: ./scripts/openshell/setup-docker-desktop.sh [additional OpenShell Helm arguments]

Adds Cilium NetworkPolicy enforcement to Docker Desktop's existing kindnet CNI,
verifies ingress and egress policy enforcement, and installs OpenShell.

Environment overrides:
  OPENSHELL_KUBE_CONTEXT      Kubernetes context (default: docker-desktop)
  DOCKER_DESKTOP_K8S_NODE    Kind node container (default: desktop-control-plane)
  CILIUM_VERSION             Cilium chart version (default: 1.20.2)

Additional arguments are forwarded to scripts/openshell/setup.sh.
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

for command in docker kubectl helm jq cilium; do
  command -v "$command" >/dev/null || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done
[[ -f "$cilium_values" ]] || {
  echo "Missing Cilium values file: $cilium_values" >&2
  exit 1
}

docker info >/dev/null
kubectl --context "$context" get nodes >/dev/null
docker inspect "$node_container" >/dev/null

kindnet_config="$(docker exec "$node_container" cat /etc/cni/net.d/10-kindnet.conflist)"
if ! jq -e '
  .plugins | type == "array" and
  any(.[]; .type == "ptp" and .ipam.type == "host-local")
' >/dev/null <<<"$kindnet_config"; then
  echo "Docker Desktop's kindnet configuration is not a supported veth setup." >&2
  exit 1
fi

chained_config="$(jq '
  if any(.plugins[]; .type == "cilium-cni") then .
  else .plugins += [{"type":"cilium-cni", "chaining-mode":"generic-veth"}]
  end
' <<<"$kindnet_config")"

kubectl --context "$context" -n kube-system create configmap cni-configuration \
  --from-literal="cni-config=$chained_config" \
  --dry-run=client -o yaml |
  kubectl --context "$context" apply -f -

helm upgrade --install cilium oci://quay.io/cilium/charts/cilium \
  --kube-context "$context" \
  --version "$cilium_version" \
  --namespace kube-system \
  --values "$cilium_values" \
  --wait --timeout 10m

cilium status --context "$context" --wait
OPENSHELL_KUBE_CONTEXT="$context" \
  "$repo_root/scripts/openshell/verify-network-policy.sh"

kubectl config use-context "$context" >/dev/null
OPENSHELL_NETWORK_POLICY_ENFORCED=true \
  exec "$repo_root/scripts/openshell/setup.sh" "$@"

