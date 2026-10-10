# SigNoz on local Kubernetes

This installs the official SigNoz Helm chart as a persistent, single-node
development deployment. It is intended for Docker Desktop's Kubernetes cluster
or a `kind` cluster running in Docker, not for production.

SigNoz recommends at least 4 CPUs and 8 GiB of memory for a local cluster. The
cluster must have a `standard` StorageClass. The setup script checks both the
active context and the StorageClass before it makes changes.

From the repository root, install or upgrade SigNoz:

```sh
./scripts/signoz/setup.sh
```

The chart version is pinned in the script so upgrades are intentional. The
installation creates a `signoz` namespace and three persistent volumes: 10 GiB
for ClickHouse, 2 GiB for ZooKeeper, and 1 GiB for the SigNoz metadata store.
Removing the Helm release does not necessarily remove its PVCs; inspect them
before deleting local data.

Verify the API through a temporary port-forward:

```sh
./scripts/signoz/verify.sh
```

After the shared gateway and workstation DNS are configured, open the UI at
<https://console.signoz.mizuumi.test>. The public collector endpoint is
<https://otlp.signoz.mizuumi.test>; it accepts OTLP/HTTP and OTLP/gRPC on port
443. On a fresh installation, create the initial user and organization in the
console before sending telemetry; SigNoz activates the tenant's collector
pipelines during that onboarding. For example, applications using OTLP/HTTP
can then set:

```sh
OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp.signoz.mizuumi.test
```

Both names use the local gateway CA. Configure and trust it with:

```sh
sudo ./scripts/configure_workstation_dns.sh
./scripts/forward.sh --trust
```

Workloads inside the cluster can send directly to
`signoz-otel-collector.signoz.svc.cluster.local` on the same ports.
