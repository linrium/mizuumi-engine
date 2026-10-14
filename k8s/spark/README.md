# Spark Operator and application chart

This directory contains values for Kubeflow Spark Operator `2.5.2` and a
repository-owned Helm chart that submits the example `SparkApplication`.
Spark `4.0.1`, Delta Lake `4.3.1`, and the Unity Catalog Spark connector
`0.5.0` use the existing `unity` catalog backed by the RustFS
`s3://unitycatalog` bucket. The operator webhook is enabled because it injects
the Secret-backed environment into driver and executor pods.

The setup runs `scripts/spark/init-catalog.sh` to create the Bronze, Silver,
and Gold schemas through the Unity Catalog CLI and grant access to the RustFS
external location. The driver authenticates as the dedicated
`unitycatalog-spark` Keycloak service principal and receives short-lived
Keycloak and RustFS credentials instead of a human token or long-lived S3
identity. Bronze-to-silver drivers additionally use the dedicated
`spark-controlplane` service principal to obtain short-lived bearer tokens for
publishing and reconciling data contracts. Executors never receive that
Control Plane client secret.

Unity Catalog-to-RustFS automatic credential vending is not enabled here:
RustFS `1.0.0` currently rejects the request emitted by the Unity Catalog
`0.6.0` Java STS client, although its STS endpoint works with standard CLI
clients. Re-run setup to rotate the Spark session before its one-hour expiry.

Bootstrap and install Unity Catalog first, then run:

```sh
./scripts/unitycatalog/bootstrap.sh
./scripts/unitycatalog/setup.sh
./scripts/spark/setup.sh
```

Spark setup reads the generated service-principal client ID and secret from
`tower/unitycatalog-credentials`. Only the driver receives the secret and
exchanges it at the canonical Keycloak token endpoint.

The script builds the reusable base image from `packages/spark/Dockerfile`,
imports it into common local Kubernetes runtimes, uploads the Python entry
point to RustFS, installs the operator, synchronizes runtime credentials and
RustFS trust, and installs this chart in the `spark` namespace. The image has
separate dependency classpaths for the connector versions used by both
examples, so the operator does not resolve Maven packages. A driver init
container fetches the entry point with an object-only presigned URL before
Spark starts.

Override the image for a registry-backed cluster with:

```sh
SPARK_IMAGE_REPOSITORY=registry.example.com/mizuumi/spark \
SPARK_IMAGE_TAG=4.0.1 ./scripts/spark/setup.sh
```

Set `SPARK_BUILD_IMAGE=false` when that image is already available. Python
application changes are uploaded separately and do not require another image.
Additional arguments are passed to this chart's `helm upgrade --install`
command.

When Flare is enabled, the chart also passes its OTLP endpoint and driver
service identity to the Python batch runtime. This exports the engine-neutral
`mizuumi.compute.operation.*` comparison metrics through the same collector;
executors continue to use Flare for native Spark telemetry.

Inspect a run with:

```sh
kubectl -n spark get sparkapplications
kubectl -n spark logs -l spark-role=driver --tail=-1
```

Delete and recreate the `medallion-minimal` resource to run the deterministic
example again. The pipeline overwrites its three Delta tables, so reruns do not
duplicate data.
