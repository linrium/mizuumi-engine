# Spark runtime

The reusable container extends Apache Spark `4.0.1` and preloads Delta Lake,
Hadoop S3A, and the Unity Catalog connector versions used by the examples.
Each connector version has a dependency classpath under
`/opt/spark/dependencies`; the chart selects one without asking the Spark
Operator to run Ivy or download Maven packages.

Application code is not coupled to the image. Setup uploads each Python entry
point to a content-addressed RustFS object and stores a short-lived presigned
GET URL in a Kubernetes Secret. A driver init container downloads the object
to an `emptyDir` before Spark starts. The presigned URL grants access to only
that object, so no long-lived RustFS key is mounted in the driver or executor.

`examples/minimal` writes a tiny bronze/silver/gold Delta pipeline through the
Unity Catalog connector and reads the result back for validation. The catalog
tables point at `s3://unitycatalog/spark/...` in RustFS.

Spark authenticates to Unity Catalog with a user token. The connector requests
short-lived, path-scoped RustFS credentials from Unity Catalog for each table;
no RustFS access key, secret key, or session token is mounted into the driver or
executor pods.

Deploy the operator and example with:

```bash
./scripts/setup_spark.sh
```

The setup initializes the `unity` catalog, the RustFS storage credential and
external location, and the `bronze`, `silver`, and `gold` schemas before it
submits the `unity-catalog-rustfs` SparkApplication. The Spark driver exchanges
its Keycloak service-principal credential for a short-lived catalog token.

For a single managed Delta table, see `examples/managed` and deploy it with
`./scripts/setup_spark_managed.sh`.
