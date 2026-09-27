# Unity Catalog + RustFS Spark example

This example runs a small bronze/silver/gold Delta Lake pipeline on the
Kubeflow Spark Operator. Unity Catalog stores the table metadata and vends
short-lived, path-scoped credentials that Spark uses to read and write data in
RustFS.

The example is intentionally small, but it exercises the complete integration:

1. Spark authenticates to Unity Catalog with a user token.
2. Spark creates or loads external Delta tables through the Unity Catalog Spark
   connector.
3. Unity Catalog requests temporary STS credentials from RustFS for the table
   path and operation.
4. The connector installs those credentials in a credential-scoped Hadoop
   filesystem.
5. Executors read and write Delta files in RustFS without receiving a
   long-lived RustFS access key.
6. The application reads the result back and validates the row count and total
   revenue.

## Data flow

The application starts with four in-memory order records and produces three
Unity Catalog tables:

| Layer | Table | RustFS location | Purpose |
| --- | --- | --- | --- |
| Bronze | `unity.bronze.orders_raw` | `s3://unitycatalog/spark/bronze/orders_raw` | Raw orders plus ingestion time |
| Silver | `unity.silver.orders_clean` | `s3://unitycatalog/spark/silver/orders_clean` | Typed dates and calculated order amounts |
| Gold | `unity.gold.daily_product_sales` | `s3://unitycatalog/spark/gold/daily_product_sales` | Daily units and revenue by product |

Each run uses `CREATE TABLE IF NOT EXISTS`, followed by `INSERT OVERWRITE`, so
the example is safe to rerun. It finishes by checking that the bronze table has
four rows and that total gold revenue is `5850`.

```text
                           CONTROL PLANE
                    table definitions and paths
                 +-------------------------------+
                 |                               v
+----------------------+                  +---------------+
| Spark application    |<---------------->| Unity Catalog |
|                      |  catalog/schema/ |               |
|  in-memory orders    |  table metadata | - unity       |
|          |           |                  | - bronze      |
|          v           |                  | - silver      |
|  bronze.orders_raw   |                  | - gold        |
|          |           |                  +---------------+
|          v           |
|  silver.orders_clean |
|          |           |                           DATA PLANE
|          v           |                    Delta files and logs
|  gold.daily_product  |                +--------------------------+
|       _sales         |                |                          v
|          |           |        temporary STS credentials   +----------+
|          v           |----------------------------------->|  RustFS  |
|  read-back checks    |<-----------------------------------| S3 API   |
+----------------------+           S3A reads/writes          +----------+
```

## Architecture

The authentication and credential-vending flow is:

```text
  Kubernetes Secret
  spark-runtime-credentials
  +-------------------------+
  | UNITY_CATALOG_TOKEN     |
  | UNITY_CATALOG_NAME      |
  +------------+------------+
               |
               | mounted as environment variables
               v
  +------------+------------+                         +----------------------+
  | Spark driver            |                         | Unity Catalog server |
  |                         |  1. Bearer user token   |                      |
  | Unity Catalog connector +------------------------>| authenticate request |
  | UCSingleCatalog         |                         | authorize operation  |
  +------------+------------+                         +----------+-----------+
               ^                                                 |
               | 5. metadata + temporary credentials             |
               +-------------------------------------------------+
                                                                 |
                                      2. AssumeRole request       |
                                         + restricted policy     |
                                         | for one table path     |
                                         v                        |
                              +----------+-----------+            |
                              | RustFS STS endpoint  |<-----------+
                              |                      |
                              | validates UC's       |
                              | service account      |
                              +----------+-----------+
                                         |
                                      3. | temporary access key
                                         | temporary secret key
                                         | session token + expiry
                                         v
                              +----------+-----------+
                              | Unity Catalog server |
                              | returns credentials  |
                              | scoped to requested  |
                              | path and operation   |
                              +----------------------+

  Important trust boundary:

  Spark pods receive                 Unity Catalog alone holds
  +----------------------------+     +-------------------------------+
  | Unity Catalog user token   |     | RustFS service-account key    |
  | vended temporary STS keys  |     | used only to call RustFS STS  |
  +----------------------------+     +-------------------------------+

  Spark pods never receive the long-lived RustFS service-account key.
```

After authentication, table access uses this read/write flow:

```text
 +--------------------+
 | PySpark SQL        |
 |                    |
 | CREATE TABLE       |
 | INSERT OVERWRITE   |
 | SELECT / validation|
 +---------+----------+
           |
           v
 +---------+-------------------+
 | Unity Catalog connector     |
 |                             |
 | 1. Resolve table metadata   |
 | 2. Request READ or          |
 |    READ_WRITE credentials   |
 | 3. Configure credential-    |
 |    scoped Hadoop filesystem |
 +---------+-------------------+
           |
           | task plan and scoped filesystem configuration
           v
 +---------+----------+             +-------------------------------+
 | Spark executor     |  S3A/HTTPS  | RustFS                        |
 |                    +------------>|                               |
 | AwsVendedToken-    |             | s3://unitycatalog/spark/      |
 | Provider           |<------------+   bronze/orders_raw/          |
 +---------+----------+  Delta data |   silver/orders_clean/        |
           |             and logs   |   gold/daily_product_sales/   |
           |                        +-------------------------------+
           |
           | credentials near expiry
           v
 +---------+----------+             +-------------------------------+
 | Unity Catalog      |------------>| RustFS STS                    |
 | renewal request    |  AssumeRole | issues replacement temporary  |
 +--------------------+<------------+ credentials                    |
                                    +-------------------------------+
```

Unity Catalog is the control plane: it knows which table maps to which RustFS
path and whether the user may read or modify it. RustFS is the data plane: it
stores the Delta transaction log and Parquet data and accepts only the
temporary credentials scoped by Unity Catalog to the requested path and
operation.

The Spark driver receives these environment variables:

- `UNITY_CATALOG_TOKEN`: user token from the `spark-runtime-credentials`
  Kubernetes Secret.
- `UNITY_CATALOG_NAME`: catalog name, normally `unity`.
- `UNITY_STORAGE_ROOT`: external table prefix, normally
  `s3://unitycatalog/spark`.

The driver and executors do **not** receive `AWS_ACCESS_KEY_ID`,
`AWS_SECRET_ACCESS_KEY`, or `AWS_SESSION_TOKEN`. Unity Catalog's Spark connector
obtains temporary credentials when it creates or loads a table. Credential
renewal is enabled for jobs that outlive the original STS session.

The Spark chart configures:

- Spark `4.0.1` with Scala `2.13`.
- Delta Lake `4.3.1`.
- Unity Catalog Spark connector `0.5.0`.
- Hadoop AWS `3.4.1` for S3A access.
- The internal Unity Catalog service URL.
- The internal TLS-enabled RustFS S3 endpoint.
- A Java truststore containing the RustFS CA.
- Credential-scoped filesystems and credential renewal.

`deltaRestApi.enabled` is disabled because this repository's Rust Unity Catalog
server implements the external-table and temporary-credential APIs, but not the
newer Delta REST commit API. Delta transaction logs and data are handled through
S3A instead.

## Prerequisites

The local platform must already have the following components running:

- A supported local Kubernetes context, such as Docker Desktop, kind, k3d, or
  minikube.
- RustFS in the `storage` namespace.
- Unity Catalog in the `tower` namespace.
- A Unity Catalog RustFS service account stored in
  `tower/unitycatalog-credentials`.
- A Unity Catalog user token supplied on the first deployment.
- Docker, Helm, `kubectl`, Java, `keytool`, `curl`, and `jq` on the workstation.

The setup uses `https://unitycatalog.mizuumi.test` and validates it with
`k8s/auth/tls/ca.crt`. Ensure the hostname resolves to the local gateway before
deploying.

## Initialize Unity Catalog only

From the repository root:

```bash
./scripts/init_spark_catalog.sh
```

The initializer is idempotent. It creates or verifies:

- Catalog: `unity`
- Storage credential: `rustfs_unitycatalog`
- External location: `rustfs_spark` at `s3://unitycatalog/spark`
- Schemas: `unity.bronze`, `unity.silver`, and `unity.gold`
- Catalog, schema, and external-location grants for the configured user

Common overrides include:

```bash
UNITYCATALOG_USER_EMAIL=user@example.com \
SPARK_UNITY_CATALOG=unity \
SPARK_UNITY_STORAGE_ROOT=s3://unitycatalog/spark \
./scripts/init_spark_catalog.sh
```

## Deploy and run

On the first run, pass a Unity Catalog user token directly or through a file:

```bash
UNITYCATALOG_USER_TOKEN='token-value' ./scripts/setup_spark.sh
```

or:

```bash
UNITYCATALOG_USER_TOKEN_FILE=/secure/path/unitycatalog.token \
./scripts/setup_spark.sh
```

The setup script performs the complete deployment:

1. Initializes the Unity Catalog resources.
2. Builds `mizuumi/spark:4.0.1-uc-rustfs` from `packages/spark/Dockerfile`.
3. Creates the Spark namespace and runtime Secrets.
4. Creates a Java truststore containing the RustFS CA.
5. Installs or upgrades Kubeflow Spark Operator.
6. Installs the Spark Helm chart and submits `unity-catalog-rustfs`.

After the first deployment, the script can reuse the user token already stored
in `spark/spark-runtime-credentials`:

```bash
./scripts/setup_spark.sh
```

To use an image that is already available to the cluster:

```bash
SPARK_BUILD_IMAGE=false ./scripts/setup_spark.sh
```

## Observe and verify the run

Watch the application:

```bash
kubectl -n spark get sparkapplications -w
```

Inspect the driver log:

```bash
kubectl -n spark logs unity-catalog-rustfs-driver --tail=-1
```

A successful run ends with output similar to:

```text
Unity Catalog / RustFS validation passed
tables: unity.bronze.orders_raw, unity.silver.orders_clean, unity.gold.daily_product_sales
+----------+-------+-----+-------+
|order_date|product|units|revenue|
+----------+-------+-----+-------+
|2026-09-27|rice   |1    |1200   |
|2026-09-27|tea    |2    |900    |
|2026-09-28|rice   |2    |2400   |
|2026-09-28|tea    |3    |1350   |
+----------+-------+-----+-------+
```

Confirm the terminal Spark state:

```bash
kubectl -n spark get sparkapplication unity-catalog-rustfs \
  -o jsonpath='{.status.applicationState.state}{"\n"}'
```

The expected state is `COMPLETED`.

List the registered tables through the Unity Catalog API:

```bash
for schema in bronze silver gold; do
  curl --fail --silent --show-error \
    --cacert k8s/auth/tls/ca.crt \
    "https://unitycatalog.mizuumi.test/api/2.1/unity-catalog/tables?catalog_name=unity&schema_name=$schema"
done
```

Verify that no RustFS credentials are mounted in the driver:

```bash
kubectl -n spark get pod unity-catalog-rustfs-driver \
  -o jsonpath='{range .spec.containers[*].env[*]}{.name}{"\n"}{end}'
```

The output should include the three `UNITY_*` variables but no `AWS_*`
credential variables.

## Configuration

Default deployment values are in `k8s/spark/values.yaml`. Useful environment
overrides for `scripts/setup_spark.sh` include:

| Variable | Default | Description |
| --- | --- | --- |
| `SPARK_NAMESPACE` | `spark` | Namespace for the SparkApplication |
| `SPARK_OPERATOR_NAMESPACE` | `spark-operator` | Operator namespace |
| `SPARK_IMAGE_REPOSITORY` | `mizuumi/spark` | Spark image repository |
| `SPARK_IMAGE_TAG` | `4.0.1-uc-rustfs` | Spark image tag |
| `SPARK_BUILD_IMAGE` | `true` | Build/load the image before deployment |
| `SPARK_UNITY_CATALOG` | `unity` | Unity Catalog name |
| `SPARK_UNITY_STORAGE_ROOT` | `s3://unitycatalog/spark` | Root for example tables |
| `UNITYCATALOG_USER_TOKEN` | unset | User token provided directly |
| `UNITYCATALOG_USER_TOKEN_FILE` | unset | File containing the user token |

Additional Helm values can be passed directly to the setup script. For
example, to run two executors:

```bash
./scripts/setup_spark.sh --set application.executor.instances=2
```

## Troubleshooting

### SparkApplication remains empty or pending

Check the operator and Kubernetes events:

```bash
kubectl -n spark-operator logs deployment/spark-operator-controller --tail=200
kubectl -n spark get events --sort-by=.lastTimestamp
```

### Unity Catalog reports a missing catalog or schema

Rerun the idempotent initializer:

```bash
./scripts/init_spark_catalog.sh
```

Then confirm the resources through `https://unitycatalog.mizuumi.test`.

### RustFS returns access denied

Check that the external table path is below `s3://unitycatalog/spark`, the
`rustfs_spark` external location exists, and the Unity Catalog server can use
its RustFS service account:

```bash
./scripts/validate_unitycatalog_vending.sh
kubectl -n tower logs deployment/unitycatalog-server --tail=200
```

Do not work around vending failures by adding long-lived AWS credentials to
the Spark pods. That bypasses the path-scoped access model this example is
designed to validate.

### TLS errors when accessing RustFS

Confirm `k8s/storage/tls/ca.crt` exists, then rerun `scripts/setup_spark.sh` so
the `spark-rustfs-trust` Secret is rebuilt.

### Dependency downloads are slow

The first driver run downloads Delta Lake, Unity Catalog, and Hadoop AWS jars
from Maven Central. Later runs may still download them because the driver pod's
Ivy directory is ephemeral. Bake the dependencies into the Spark image or add
a persistent Ivy cache if faster cold starts are required.

## Related files

- `main.py`: PySpark application and result validation.
- `packages/spark/Dockerfile`: Spark runtime image.
- `k8s/spark/values.yaml`: default application and integration settings.
- `k8s/spark/templates/sparkapplication.yaml`: Kubeflow SparkApplication.
- `scripts/init_spark_catalog.sh`: catalog and storage metadata initialization.
- `scripts/setup_spark.sh`: image build, operator installation, and deployment.
