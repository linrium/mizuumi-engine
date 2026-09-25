#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
namespace=vault
release=vault
chart_version=0.34.1

for command in kubectl helm openssl jq; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

context="$(kubectl config current-context)"
case "$context" in
  kind-*|k3d-*|minikube|minikube-*|docker-desktop|rancher-desktop|orbstack|microk8s|colima|k3s)
    ;;
  *)
    echo "Refusing to install local Vault values into non-local context: $context" >&2
    exit 1
    ;;
esac
echo "Using Kubernetes context: $context"
if kubectl -n "$namespace" get statefulset "$release" -o json 2>/dev/null | \
   jq -e '.spec.template.spec.containers[] | select(.name == "vault") | .env[] | select(.name == "VAULT_DEV_ROOT_TOKEN_ID")' >/dev/null 2>&1; then
  echo "Existing Vault is in dev mode. Its StatefulSet has no PVC, and Kubernetes cannot add one in place." >&2
  echo "Back up any dev data, then explicitly uninstall the old release and rerun this script:" >&2
  echo "  helm -n vault uninstall vault" >&2
  echo "This destroys the current in-memory Vault state; rerun bootstrap_sovico.sh afterward." >&2
  exit 1
fi
tls_dir="$repo_root/k8s/vault/tls"
previous_cert="$(openssl x509 -in "$tls_dir/tls.crt" -noout -fingerprint -sha256 2>/dev/null || true)"
"$repo_root/scripts/ensure_local_tls.sh" "$tls_dir" \
  "$repo_root/k8s/vault/tls.cnf" "$repo_root/k8s/vault/ca.cnf"
current_cert="$(openssl x509 -in "$tls_dir/tls.crt" -noout -fingerprint -sha256)"
kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f -
kubectl -n "$namespace" create secret generic vault-tls \
  --from-file=tls.crt="$tls_dir/tls.crt" \
  --from-file=tls.key="$tls_dir/tls.key" \
  --from-file=ca.crt="$tls_dir/ca.crt" \
  --dry-run=client -o yaml | kubectl apply -f -
helm repo add hashicorp https://helm.releases.hashicorp.com --force-update
helm repo update hashicorp

helm upgrade --install "$release" hashicorp/vault \
  --version "$chart_version" \
  --values "$repo_root/k8s/vault/values.yaml" \
  --namespace "$namespace" --create-namespace

for attempt in {1..120}; do
  pod_phase="$(kubectl -n "$namespace" get pod "$release-0" -o jsonpath='{.status.phase}' 2>/dev/null || true)"
  [[ "$pod_phase" == Running ]] && break
  if [[ "$attempt" -eq 120 ]]; then
    echo "Vault pod did not reach Running; inspect kubectl -n vault describe pod vault-0." >&2
    exit 1
  fi
  sleep 2
done

# The official chart uses OnDelete for its StatefulSet, so Helm can report a
# successful upgrade while the old dev pod is still running.
desired_revision="$(kubectl -n "$namespace" get statefulset "$release" -o jsonpath='{.status.updateRevision}')"
pod_revision="$(kubectl -n "$namespace" get pod "$release-0" -o jsonpath='{.metadata.labels.controller-revision-hash}')"
if [[ -n "$desired_revision" && "$pod_revision" != "$desired_revision" ]]; then
  echo "Vault's pod still uses the previous chart revision." >&2
  echo "Back up any existing dev-mode data before replacing the pod. Then run:" >&2
  echo "  kubectl -n $namespace delete pod $release-0" >&2
  exit 1
fi

if [[ -n "$previous_cert" && "$previous_cert" != "$current_cert" ]]; then
  echo "Restarting Vault to load its new TLS certificate."
  kubectl -n "$namespace" delete pod "$release-0"
  for attempt in {1..120}; do
    pod_phase="$(kubectl -n "$namespace" get pod "$release-0" -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    [[ "$pod_phase" == Running ]] && break
    if [[ "$attempt" -eq 120 ]]; then
      echo "Vault pod did not return to Running after its certificate restart." >&2
      exit 1
    fi
    sleep 2
  done
  if [[ -f "$repo_root/k8s/vault/init.json" ]]; then
    "$repo_root/scripts/unseal_vault.sh"
  fi
fi

echo "Vault is deployed. On first install, initialize and unseal with ./scripts/init_vault.sh."
echo "After each restart, unseal with ./scripts/unseal_vault.sh."
echo "Local UI: https://vault.localhost:8200/ui/ (trust k8s/vault/tls/ca.crt)."
