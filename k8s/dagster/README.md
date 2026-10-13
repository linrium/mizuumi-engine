# Dagster

Umbrella Helm chart that installs the upstream
[`dagster/dagster`](https://github.com/dagster-io/dagster) chart alongside the
Mizuumi user-code deployment in `packages/dagster/`.

## Architecture

The optional `managed_spark` code location orchestrates the managed-orders
Spark workload. It has **no** Kubeflow Spark Operator dependency at runtime;
each materialization runs `spark-submit --deploy-mode cluster` directly. The
default `sovico` code location uses the operator API only to observe external
streaming applications; it does not manage their lifecycle.

Per schedule tick (`*/10 * * * *` UTC) Dagster materializes the asset
`unity/demo/orders` by:

1. Creating a short-lived Kubernetes `Job` in the Spark namespace running the
   `mizuumi/spark` image. Its command is `spark-submit --master
   k8s://https://kubernetes.default.svc --deploy-mode cluster …`, with
   `--conf` flags that mirror the sparkConf, driver env (from the credentials
   Secret), truststore volume (from the trust Secret), and ConfigMap-mounted
   script that the SparkApplication CR used to encode.
2. Streaming the submission pod's stdout line-by-line into the Dagster run log
   with `CoreV1Api.read_namespaced_pod_log(follow=True)`, so the Spark driver
   output (which `spark-submit --conf spark.kubernetes.submission.waitAppCompletion=true`
   mirrors into the submission pod) appears live in the UI.
3. Polling the Job's `.status.{succeeded,failed}` once the log stream closes
   and surfacing success/failure as the asset materialization outcome.

## Prerequisites

The Spark image and credential/trust Secrets must exist in the Spark
namespace. Run `./scripts/spark/setup.sh` once to:

- build `mizuumi/spark:4.0.1-uc-kafka-rustfs-baked` and load it into the local
  cluster,
- create `spark-runtime-credentials` (Keycloak client + UC + RustFS creds),
- create `spark-rustfs-trust` (JVM cacerts + Keycloak CA + RustFS CA).

You do **not** need to run `./scripts/spark/setup-managed.sh` for Dagster —
that script also installs the Kubeflow Spark Operator and a SparkApplication
CR, neither of which this flow uses.

## Install

```bash
./scripts/dagster/setup.sh
./scripts/auth/setup.sh       # once, to pick up the Caddy routes/SANs
./scripts/forward.sh          # add --trust on first macOS run
```

- Dagster Console: <https://console.dagster.mizuumi.test>
- Dagster GraphQL: <https://api.dagster.mizuumi.test/graphql>

Open the asset `unity/demo/orders` under group `unity_catalog`, enable the
`unity_catalog_managed_orders_every_10_minutes` schedule, then watch:

```bash
kubectl -n spark get pods -l 'spark-role in (driver,executor)' -w
```

When the `sovico` code location is selected (the default in
`scripts/dagster/setup.sh`), Dagster also models and observes the externally
managed source-domain streaming pipelines:

```text
Redpanda topic -> <source>/bronze/events -> <source>/silver/<table>
```

Each source's physical silver tables are modeled as one non-subsettable
multi-asset because a single Spark submission updates all of them. Dagster
records every table as materialized only after that submission succeeds.

The `external_spark_streaming_observation` sensor starts enabled and polls the
CRM, HDBank, Vikki, and VietJet `SparkApplication` resources once per minute.
It records application state, driver pod state and restarts, submission
attempts, and Spark UI metadata on each bronze asset. These observations do
not start, stop, or restart the streaming applications.

Clean up completed and failed Dagster run-worker Jobs and their owned pods:

```bash
./scripts/dagster/cleanup-run-pods.sh --dry-run
./scripts/dagster/cleanup-run-pods.sh
```

The cleanup preserves active runs and the long-lived webserver, daemon,
user-code, and PostgreSQL pods. Set `DAGSTER_NAMESPACE` or `DAGSTER_RELEASE`
when using a non-default installation.

## RBAC

The chart creates two bindings:

- `<release>-spark-trigger` — Role in the Spark namespace granting the Dagster
  SA (`mizuumi-dagster`) permission to CRUD `batch/jobs`, read pods, and read
  `SparkApplication` status so the run worker can launch submission Jobs and
  the observation sensor can report external stream health.
- `dagster-spark-submit` — ServiceAccount + Role in the Spark namespace that
  `spark-submit` uses to create the driver pod, services, configmaps, and
  PVCs it needs (per Apache Spark's documented
  [RBAC requirements](https://spark.apache.org/docs/latest/running-on-kubernetes.html#rbac)).

## Layout

- `Chart.yaml` / `values.yaml` — umbrella chart referencing the upstream chart.
- `templates/spark-trigger-rbac.yaml` — Role for Dagster to manage Jobs and
  read pod and SparkApplication status in the Spark namespace.
- `templates/spark-submit-rbac.yaml` — ServiceAccount + Role that
  `spark-submit` uses in-cluster.
- `templates/job-launcher-rbac.yaml` — Role for the K8sRunLauncher to launch
  run-worker Jobs inside the Dagster namespace.
- `../../packages/dagster/` — Python code location image (Dockerfile +
  `managed_spark` module with the asset, schedule, and `--conf` builder).
- `../../scripts/dagster/setup.sh` — builds the user-code image, loads it
  into the local cluster, publishes the Spark script as a ConfigMap, and
  installs this chart.
