# Managed Unity Catalog table example

This example creates one Delta table, `unity.demo.orders`, using `CREATE TABLE`
without a `LOCATION` clause. Unity Catalog allocates its managed storage path.
The application overwrites three sample rows on each run, reads them back, and
checks that Unity Catalog reports `table_type: MANAGED`.

From the repository root, after the local RustFS, Keycloak, and Unity Catalog
services are set up, deploy with:

```bash
./scripts/setup_spark_managed.sh
```

The script initializes the `demo` schema and its grants, uploads `main.py` to a
content-addressed object in RustFS, builds the reusable Spark base image,
installs the Spark Operator, and submits `unity-catalog-managed`. The driver
downloads the script through a one-object presigned URL before Spark starts.
It accepts the same environment overrides and extra Helm options as
`setup_spark.sh`. For an image already available to the cluster:

```bash
SPARK_BUILD_IMAGE=false ./scripts/setup_spark_managed.sh
```

The base image preloads the Maven artifacts used by both examples. Rebuild it
only when Spark, Delta Lake, Hadoop AWS, or a Unity Catalog connector version
changes; editing `main.py` requires only another setup run and RustFS upload.
The presigned URL expires after one hour by default. Set
`SPARK_APPLICATION_URL_EXPIRATION` (up to 604800 seconds) when a queued job may
take longer to start.

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
