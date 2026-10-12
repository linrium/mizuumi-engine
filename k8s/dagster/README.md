# Dagster

Umbrella Helm chart that installs the upstream
[`dagster/dagster`](https://github.com/dagster-io/dagster) chart alongside the
Mizuumi user-code deployment defined in `packages/dagster/`.

The example code location (`mizuumi_dagster`) ships one job and one schedule:

- Job `unity_catalog_managed_job` uses `dagster_k8s.k8s_job_op` to launch a
  one-shot Kubernetes Job (image `alpine/k8s`) that snapshots the live
  `unity-catalog-managed` SparkApplication with `kubectl get -o yaml | yq`,
  deletes it, and re-applies the snapshot. The Spark Operator then runs it
  again as a fresh SparkApplication.
- Schedule `unity_catalog_managed_every_10_minutes` fires `*/10 * * * *` (UTC).

Each schedule tick becomes a visible `kubectl -n dagster get jobs` entry.

## Prerequisites

1. The Spark Operator and the `unity-catalog-managed` SparkApplication are
   already installed via `./scripts/spark/setup-managed.sh`.
2. A local Kubernetes context (kind, k3d, Docker Desktop, Rancher Desktop,
   orbstack, colima, …).

## Install

```bash
./scripts/dagster/setup.sh
./scripts/auth/setup.sh      # regenerate gateway cert (new SANs) and
                             # pick up the Caddy routes for Dagster
./scripts/forward.sh         # on macOS, add --trust the first time
```

Open the UI at <https://console.dagster.mizuumi.test> and the GraphQL API at
<https://api.dagster.mizuumi.test/graphql>. Both route through the shared
Caddy gateway to the Dagster webserver Service.

In the UI, enable the schedule `unity_catalog_managed_every_10_minutes` if
auto-start is off, then watch:

```bash
kubectl -n spark get sparkapplications -w
kubectl -n dagster get jobs -w
```

## Layout

- `Chart.yaml` / `values.yaml` — umbrella chart referencing the upstream chart.
- `templates/spark-trigger-rbac.yaml` — grants the Dagster service account
  permission to manage SparkApplications in the Spark namespace.
- `../../packages/dagster/` — Python code location image (Dockerfile +
  `mizuumi_dagster` module).
- `../../scripts/dagster/setup.sh` — builds the image, loads it into the local
  cluster, and installs the chart.
