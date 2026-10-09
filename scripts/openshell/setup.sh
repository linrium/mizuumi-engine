#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
namespace="${OPENSHELL_NAMESPACE:-openshell}"
release="${OPENSHELL_RELEASE:-openshell}"
openshell_version="${OPENSHELL_VERSION:-0.1.2}"
agent_sandbox_version="${AGENT_SANDBOX_VERSION:-v1.0.5}"
chart="${OPENSHELL_CHART:-oci://ghcr.io/nvidia/openshell/helm-chart}"
values_file="${OPENSHELL_VALUES_FILE:-$repo_root/k8s/openshell/values.yaml}"
agent_sandbox_manifest="https://github.com/kubernetes-sigs/agent-sandbox/releases/download/${agent_sandbox_version}/sandbox.yaml"

usage() {
  cat <<'EOF'
Usage: ./scripts/openshell/setup.sh [additional Helm arguments]

Installs the Agent Sandbox controller and NVIDIA OpenShell into Kubernetes.

Environment overrides:
  OPENSHELL_NAMESPACE                  Namespace (default: openshell)
  OPENSHELL_RELEASE                    Helm release (default: openshell)
  OPENSHELL_VERSION                    Helm chart version (default: 0.1.2)
  OPENSHELL_CHART                      Helm OCI reference
  OPENSHELL_VALUES_FILE                Helm values file
  AGENT_SANDBOX_VERSION                Controller release (default: v1.0.5)
  OPENSHELL_NETWORK_POLICY_ENFORCED    Set true after verifying an unrecognized
                                       CNI enforces ingress and egress policies
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

for command in kubectl helm jq; do
  command -v "$command" >/dev/null || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done
[[ -f "$values_file" ]] || {
  echo "OpenShell values file does not exist: $values_file" >&2
  exit 1
}

context="$(kubectl config current-context)"
echo "Using Kubernetes context: $context"

if ! server_version_json="$(kubectl version -o json)"; then
  echo "Unable to reach Kubernetes context $context." >&2
  exit 1
fi
server_version="$(printf '%s' "$server_version_json" | jq -r '.serverVersion | [.major, (.minor | sub("[^0-9].*$"; ""))] | @tsv')"
read -r server_major server_minor <<<"$server_version"
if (( server_major < 1 || (server_major == 1 && server_minor < 29) )); then
  echo "OpenShell requires Kubernetes 1.29 or newer; cluster is ${server_major}.${server_minor}." >&2
  exit 1
fi

network_policy_enforced="${OPENSHELL_NETWORK_POLICY_ENFORCED:-false}"
if [[ "$network_policy_enforced" != true ]]; then
  cni_inventory="$(kubectl -n kube-system get daemonsets -o json | jq -r '[.items[] | .metadata.name, (.spec.template.spec.containers[]?.image // "")] | join(" ")' | tr '[:upper:]' '[:lower:]')"
  if [[ ! "$cni_inventory" =~ cilium|calico|antrea|kube-router ]]; then
    if [[ "$cni_inventory" =~ kindnet ]]; then
      cat >&2 <<EOF
Context $context uses kindnet, which does not enforce Kubernetes NetworkPolicy.
Do not bypass this check for OpenShell. Add Cilium policy enforcement to Docker
Desktop's existing kindnet network, then install OpenShell:
  ./scripts/openshell/setup-docker-desktop.sh
EOF
      exit 1
    fi
    cat >&2 <<EOF
Could not verify an enforcing NetworkPolicy CNI in context $context.
OpenShell sandboxes require ingress and egress NetworkPolicy enforcement.
Verify the cluster CNI, then rerun with OPENSHELL_NETWORK_POLICY_ENFORCED=true.
EOF
    exit 1
  fi
fi

if [[ "$namespace" == openshell ]]; then
  kubectl apply -f "$repo_root/k8s/openshell/namespace.yaml"
else
  kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f -
fi
kubectl apply --server-side -f "$agent_sandbox_manifest"
kubectl -n agent-sandbox-system wait \
  --for=condition=Available deployment --all --timeout=5m

helm upgrade --install "$release" "$chart" \
  --version "$openshell_version" \
  --namespace "$namespace" \
  --values "$values_file" \
  --set-string "fullnameOverride=$release" \
  --wait --timeout 10m \
  "$@"

workload_kind="$(helm get values "$release" -n "$namespace" -a -o json | jq -r '.workload.kind // "statefulset"')"
kubectl -n "$namespace" rollout status "$workload_kind/$release" --timeout=5m

if kubectl -n auth get deployment keycloak-gateway >/dev/null 2>&1; then
  "$repo_root/scripts/openshell/expose.sh"
else
  echo "The shared local gateway is not installed; skipping openshell.mizuumi.test exposure."
fi

echo "OpenShell $openshell_version is ready in namespace $namespace."
echo "Next: ./scripts/openshell/bootstrap.sh"
