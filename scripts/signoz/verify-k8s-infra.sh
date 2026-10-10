#!/usr/bin/env bash
set -euo pipefail

namespace=signoz-infra
release=signoz-k8s-infra

for command in kubectl helm jq; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|docker-desktop)
    ;;
  *)
    echo "Refusing to verify SigNoz k8s-infra in unsupported context: $context" >&2
    exit 1
    ;;
esac

if [[ "$(helm -n "$namespace" status "$release" -o json | jq -r '.info.status')" != deployed ]]; then
  echo "Helm release $namespace/$release is not deployed." >&2
  exit 1
fi

kubectl -n "$namespace" rollout status daemonset/"$release-otel-agent" --timeout=3m
kubectl -n "$namespace" rollout status deployment/"$release-otel-deployment" --timeout=3m

echo
kubectl get pods -n "$namespace"
echo
echo "SigNoz k8s-infra collectors are ready."
