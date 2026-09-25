#!/usr/bin/env bash
set -euo pipefail

namespace=auth
service=keycloak-gateway
hostnames=(auth.mizuumi.test vault.mizuumi.test storage.mizuumi.test api.storage.mizuumi.test uc.mizuumi.test)
hostnames_joined="${hostnames[*]}"
marker_begin='    # BEGIN mizuumi auth DNS'
marker_end='    # END mizuumi auth DNS'

for command in kubectl jq awk; do
  command -v "$command" >/dev/null || { echo "Missing required command: $command" >&2; exit 1; }
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s) ;;
  *) echo "Refusing to change cluster DNS on non-local context: $context" >&2; exit 1 ;;
esac

gateway_ip="$(kubectl -n "$namespace" get service "$service" -o jsonpath='{.spec.clusterIP}')"
if [[ -z "$gateway_ip" || "$gateway_ip" == None ]]; then
  echo "Service $namespace/$service does not have a ClusterIP." >&2
  exit 1
fi

corefile="$(kubectl -n kube-system get configmap coredns -o jsonpath='{.data.Corefile}')"
updated_corefile="$(printf '%s\n' "$corefile" | awk \
  -v ip="$gateway_ip" -v hostnames="$hostnames_joined" -v begin="$marker_begin" -v end="$marker_end" '
    $0 == begin { skipping = 1; next }
    $0 == end { skipping = 0; next }
    skipping { next }
    !inserted && /^[[:space:]]*\.:53[[:space:]]*\{[[:space:]]*$/ {
      print
      print begin
      print "    hosts {"
      print "        " ip " " hostnames
      print "        fallthrough"
      print "    }"
      print end
      inserted = 1
      next
    }
    { print }
    END { if (!inserted) exit 42 }
  ')" || {
    echo "Could not find the default .:53 server block in the CoreDNS Corefile." >&2
    exit 1
  }

if [[ "$updated_corefile" != "$corefile" ]]; then
  payload="$(jq -cn --arg corefile "$updated_corefile"$'\n' '{data:{Corefile:$corefile}}')"
  kubectl -n kube-system patch configmap coredns --type=merge -p "$payload" >/dev/null
  dns_deployment="$(kubectl -n kube-system get deployment -l k8s-app=kube-dns -o jsonpath='{.items[0].metadata.name}')"
  if [[ -z "$dns_deployment" ]]; then
    echo "CoreDNS ConfigMap updated, but its Deployment could not be found." >&2
    exit 1
  fi
  kubectl -n kube-system rollout restart deployment/"$dns_deployment" >/dev/null
  kubectl -n kube-system rollout status deployment/"$dns_deployment" --timeout=2m
fi

echo "$hostnames_joined resolve to $gateway_ip inside the cluster."
