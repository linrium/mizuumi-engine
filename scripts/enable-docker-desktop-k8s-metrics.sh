#!/usr/bin/env bash
set -euo pipefail

context="${KUBE_CONTEXT:-docker-desktop}"
metrics_server_version="${METRICS_SERVER_VERSION:-v0.8.0}"
namespace="kube-system"
deployment="metrics-server"
manifest_url="https://github.com/kubernetes-sigs/metrics-server/releases/download/${metrics_server_version}/components.yaml"

usage() {
  cat <<'EOF'
Usage: ./scripts/enable-docker-desktop-k8s-metrics.sh

Installs Metrics Server in Docker Desktop Kubernetes and enables kubectl top.

Environment overrides:
  KUBE_CONTEXT             Kubernetes context (default: docker-desktop)
  METRICS_SERVER_VERSION  Metrics Server release (default: v0.8.0)
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

command -v kubectl >/dev/null 2>&1 || {
  echo "Missing required command: kubectl" >&2
  exit 1
}

if [[ "$context" != "docker-desktop" ]]; then
  echo "Refusing to configure metrics for non-Docker Desktop context: $context" >&2
  exit 1
fi

if [[ "$(kubectl config get-contexts "$context" -o name 2>/dev/null)" != "$context" ]]; then
  echo "Kubernetes context not found: $context" >&2
  echo "Enable Kubernetes in Docker Desktop and try again." >&2
  exit 1
fi

echo "Installing Metrics Server $metrics_server_version in context $context..."
kubectl --context "$context" apply -f "$manifest_url"

# Docker Desktop's kubelet uses a locally issued serving certificate that the
# stock Metrics Server manifest cannot verify. This exception is appropriate
# only for this local development cluster.
args="$(kubectl --context "$context" -n "$namespace" get deployment "$deployment" \
  -o jsonpath='{.spec.template.spec.containers[?(@.name=="metrics-server")].args[*]}')"
if [[ " $args " != *" --kubelet-insecure-tls "* ]]; then
  kubectl --context "$context" -n "$namespace" patch deployment "$deployment" \
    --type=json \
    -p='[{"op":"add","path":"/spec/template/spec/containers/0/args/-","value":"--kubelet-insecure-tls"}]'
fi

kubectl --context "$context" -n "$namespace" rollout status deployment/"$deployment" --timeout=3m
kubectl --context "$context" wait --for=condition=Available \
  apiservice/v1beta1.metrics.k8s.io --timeout=2m

echo
metrics_ready=false
for _ in {1..24}; do
  if kubectl --context "$context" top nodes >/dev/null 2>&1; then
    metrics_ready=true
    break
  fi
  sleep 5
done

if [[ "$metrics_ready" != true ]]; then
  echo "Metrics Server is running, but node metrics did not become available within 2 minutes." >&2
  kubectl --context "$context" -n "$namespace" logs deployment/"$deployment" --tail=50 >&2
  exit 1
fi

kubectl --context "$context" top nodes
echo
echo "Docker Desktop Kubernetes metrics are ready."
echo "View pod usage with: kubectl --context $context top pods --all-namespaces"
