#!/usr/bin/env bash
set -euo pipefail

context="${OPENSHELL_KUBE_CONTEXT:-$(kubectl config current-context)}"
namespace="openshell-network-policy-test"

for command in kubectl; do
  command -v "$command" >/dev/null || {
    echo "Missing required command: $command" >&2
    exit 1
  }
done

cleanup() {
  kubectl --context "$context" delete namespace "$namespace" \
    --ignore-not-found --wait=false >/dev/null 2>&1 || true
}
trap cleanup EXIT

kubectl --context "$context" delete namespace "$namespace" \
  --ignore-not-found --wait=true --timeout=2m >/dev/null
kubectl --context "$context" create namespace "$namespace" >/dev/null

kubectl --context "$context" -n "$namespace" apply -f - <<'EOF'
apiVersion: v1
kind: Pod
metadata:
  name: server
  labels:
    app: server
spec:
  containers:
    - name: server
      image: busybox:1.37
      command:
        - sh
        - -c
        - echo ok >/tmp/index.html; exec httpd -f -p 8080 -h /tmp
---
apiVersion: v1
kind: Service
metadata:
  name: server
spec:
  selector:
    app: server
  ports:
    - port: 8080
---
apiVersion: v1
kind: Pod
metadata:
  name: client
  labels:
    app: client
spec:
  containers:
    - name: client
      image: busybox:1.37
      command: ["sleep", "3600"]
EOF

kubectl --context "$context" -n "$namespace" wait \
  --for=condition=Ready pod/server pod/client --timeout=3m
server_ip="$(kubectl --context "$context" -n "$namespace" \
  get pod server -o jsonpath='{.status.podIP}')"
[[ -n "$server_ip" ]] || {
  echo "NetworkPolicy probe server has no pod IP." >&2
  exit 1
}

can_connect() {
  kubectl --context "$context" -n "$namespace" exec client -- \
    wget -q -T 3 -O /dev/null "http://$server_ip:8080"
}

wait_for_result() {
  local expected="$1"
  for _ in {1..15}; do
    if can_connect >/dev/null 2>&1; then
      [[ "$expected" == allowed ]] && return 0
    else
      [[ "$expected" == denied ]] && return 0
    fi
    sleep 1
  done
  return 1
}

wait_for_result allowed || {
  echo "NetworkPolicy probe could not establish baseline pod connectivity." >&2
  exit 1
}

kubectl --context "$context" -n "$namespace" apply -f - <<'EOF'
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-server-ingress
spec:
  podSelector:
    matchLabels:
      app: server
  policyTypes: [Ingress]
  ingress: []
EOF
wait_for_result denied || {
  echo "The cluster did not enforce ingress NetworkPolicy." >&2
  exit 1
}

kubectl --context "$context" -n "$namespace" delete networkpolicy deny-server-ingress
wait_for_result allowed || {
  echo "Connectivity did not recover after removing the ingress policy." >&2
  exit 1
}

kubectl --context "$context" -n "$namespace" apply -f - <<'EOF'
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-client-egress
spec:
  podSelector:
    matchLabels:
      app: client
  policyTypes: [Egress]
  egress: []
EOF
wait_for_result denied || {
  echo "The cluster did not enforce egress NetworkPolicy." >&2
  exit 1
}

echo "Ingress and egress NetworkPolicy enforcement verified in context $context."
