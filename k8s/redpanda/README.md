# Redpanda on local Kubernetes

This installs the official Redpanda Helm chart 5.9.18 with one persistent Redpanda broker and Redpanda Console in the `redpanda` namespace. It is a local Docker Desktop/kind development deployment, not a production topology.

## SigNoz telemetry

The cluster-level `signoz-k8s-infra` OpenTelemetry Collector scrapes the broker's low-cardinality `/public_metrics` endpoint every 30 seconds, then exports the metrics to SigNoz. Metrics use the service name `redpanda`, the `redpanda` service and Kubernetes namespaces, the `local` deployment environment, and the `docker-desktop` cluster name.

The node-level collector sends broker, Console, and configuration-watcher container logs to SigNoz with consistent service names. The Redpanda broker does not expose a native OTLP tracing exporter, so this integration provides metrics and logs rather than synthetic traces.

Install or upgrade the components individually in this order:

```sh
./scripts/signoz/setup.sh
./scripts/redpanda/setup.sh
./scripts/signoz/setup-k8s-infra.sh
```

Run `./scripts/redpanda/bootstrap.sh` to create the local example topics and generate broker activity. The HTTP API and Console remain available through `https://api.redpanda.mizuumi.test` and `https://console.redpanda.mizuumi.test` after configuring workstation DNS.
