# Managed Unity Catalog table example

This example creates one Delta table, `unity.demo.orders`, using `CREATE TABLE`
without a `LOCATION` clause. Unity Catalog allocates its managed storage path.
The application overwrites three sample rows on each run, reads them back, and
checks that Unity Catalog reports `table_type: MANAGED`.

From the repository root, after the local RustFS, Keycloak, and Unity Catalog
services are set up, deploy with:

```bash
./scripts/spark/setup-managed.sh
```

The script initializes the `demo` schema and its grants, mounts `main.py` from
a ConfigMap, builds the reusable Spark base image, installs the Spark Operator,
and submits `unity-catalog-managed`. It accepts the same environment overrides
and extra Helm options as `scripts/spark/setup.sh`. For an updated image that is
already available to the cluster:

```bash
SPARK_BUILD_IMAGE=false ./scripts/spark/setup-managed.sh
```

The base image preloads the Maven artifacts, Flare extension, and matching
OpenTelemetry Java agent used by the examples. Rebuild it when Spark, Delta
Lake, Hadoop AWS, Unity Catalog connector, Flare, or OpenTelemetry agent
versions change; editing `main.py` requires only another setup run.

The managed deployment uses Unity Catalog Spark connector `0.6.0` because its
OAuth provider supports the Keycloak `openid` scope. The Spark catalog uses the
connector's OAuth client-credentials provider with
the `unitycatalog-spark` Keycloak service principal. The provider renews the
Unity Catalog access token before it expires, so catalog requests and storage
credential renewal can continue during long-running jobs. The example's direct
table-metadata check requests its own fresh token when it runs.

Watch the job and inspect its output with:

```bash
kubectl -n spark get sparkapplication unity-catalog-managed -w
kubectl -n spark logs unity-catalog-managed-driver --tail=-1
```

The final log prints the allocated storage location and three rows. Re-running
the setup script resubmits the application, keeps the same table, and replaces
its data.

The managed example uses Flare `1.4.0` with OpenTelemetry Java agent `2.31.1`
to export traces and metrics over OTLP/HTTP to the in-cluster SigNoz collector.
Flare traces the Spark application, SQL executions, jobs, stages, and executor
tasks without application-level instrumentation. Install SigNoz before
submitting the example:

```bash
./scripts/signoz/setup.sh
./scripts/signoz/setup-k8s-infra.sh
```

The infrastructure collector continues to supply the Spark pod logs and
Kubernetes resource metrics. Flare's OTLP log exporter is disabled to avoid
duplicating those logs. In SigNoz, start with service
`unity-catalog-managed-driver`; executor task spans use service
`unity-catalog-managed-executor` in the same distributed trace.
