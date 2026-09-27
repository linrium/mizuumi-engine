# Spark runtime

The container extends Apache Spark `4.0.1` with the repository's PySpark
examples. Maven dependencies are declared by the `SparkApplication`, allowing
the Spark Operator to resolve Delta Lake, Unity Catalog, and Hadoop S3A at
submission time.

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
submits the `unity-catalog-rustfs` SparkApplication. On the first run, set
`UNITYCATALOG_USER_TOKEN` or `UNITYCATALOG_USER_TOKEN_FILE`; later runs reuse
the token already stored in the Spark runtime Secret.
