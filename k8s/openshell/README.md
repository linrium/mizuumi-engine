# OpenShell on Kubernetes

This directory deploys NVIDIA OpenShell with its upstream OCI Helm chart. The
default values are intended for local evaluation through the shared local
gateway. They enable the chart's built-in PKI, use a single SQLite-backed
gateway, and validate Keycloak access tokens for the `openshell-cli` audience.
The setup script installs the local Keycloak CA as the gateway's OIDC trust
bundle; mTLS remains in use at the transport layer.

## Requirements

- Kubernetes 1.29 or newer with RBAC enabled.
- Docker Desktop Kubernetes, Helm 3, `kubectl`, `jq`, and the Cilium CLI for
  the included Docker Desktop setup.
- A CNI that enforces both ingress and egress `NetworkPolicy`. OpenShell creates
  policies for sandbox workloads, but a cluster without an enforcing CNI will
  silently accept and ignore them.
- The `openshell` CLI on the workstation. Install the same release as the chart
  with NVIDIA's installer if it is not already available.

## Install

The setup script installs the pinned Agent Sandbox controller and OpenShell
chart, then waits for both rollouts:

```sh
./scripts/openshell/setup.sh
```

Known enforcing CNIs such as Cilium, Calico, Antrea, and kube-router are
detected automatically. For another CNI, verify enforcement first and run:

```sh
OPENSHELL_NETWORK_POLICY_ENFORCED=true ./scripts/openshell/setup.sh
```

### Docker Desktop

Docker Desktop's Kind-based Kubernetes cluster uses `kindnet`, which does not
enforce Kubernetes `NetworkPolicy` by itself. The Docker Desktop setup script
adds Cilium in generic veth chaining mode, preserving kindnet for pod networking
while Cilium enforces policy:

```sh
./scripts/openshell/setup-docker-desktop.sh
```

This reads Docker Desktop's live kindnet CNI configuration, appends the Cilium
chaining plugin, installs Cilium `1.20.2`, and proves both ingress and egress
policy enforcement with disposable test pods. It then installs OpenShell into
the existing `docker-desktop` context. Existing pods continue using kindnet;
pods created after setup join the chained CNI configuration.

Override the pinned versions or namespace when testing an upgrade:

```sh
OPENSHELL_VERSION=0.1.2 \
AGENT_SANDBOX_VERSION=v1.0.5 \
OPENSHELL_NAMESPACE=openshell \
./scripts/openshell/setup.sh
```

Additional arguments are passed to `helm upgrade --install`.

## Bootstrap the CLI and example

With the gateway installed and the OpenShell CLI available, run:

```sh
./scripts/studio/bootstrap.sh
./scripts/openshell/bootstrap.sh
```

The Studio bootstrap enables the Keycloak service account, audience, and
OpenShell roles. The OpenShell bootstrap then exposes the gateway at
`https://openshell.mizuumi.test`, copies its TLS identity and CA to the local
OpenShell configuration, registers the `k8s` gateway for OIDC, and runs the
minimal example under `packages/openshell`. Later CLI commands reconnect
through that hostname:

```sh
openshell status
```

Run `sudo ./scripts/configure_workstation_dns.sh` once if the workstation does
not resolve the hostname yet. The shared gateway verifies both the Keycloak
bearer token and the OpenShell CLI certificate, then uses mTLS again for the
upstream connection. The bootstrap supplies the local CA to the CLI for OIDC
discovery. For later direct CLI use, trust that CA once with
`./scripts/forward.sh --trust` or set `SSL_CERT_FILE=k8s/auth/tls/ca.crt`.

Pass `--skip-example` to configure and verify the gateway without creating a
sandbox. Environment overrides are documented by each script's `--help`.

## Shared clusters

The bundled values reject unauthenticated user requests and configure the local
Keycloak issuer. For a shared cluster, replace the issuer, audience, CA, and
role names with that environment's identity-provider settings. Keep the Service
as `ClusterIP` unless the external access path provides the required TLS
behavior.

The setup follows NVIDIA's Kubernetes deployment model: the Agent Sandbox
controller owns workload pods, while OpenShell creates the supervisor, service,
workspace PVC, and network policies around each sandbox.
