# Spark runtime

The container extends Apache Spark `4.0.1` with the repository's PySpark
examples. Maven dependencies are declared by the `SparkApplication`, allowing
the Spark Operator to resolve Delta Lake, Unity Catalog, and Hadoop S3A at
submission time.

`examples/minimal` validates the `unity.bronze`, `unity.silver`, and
`unity.gold` schemas and writes Delta datasets to matching RustFS paths.
`scripts/init_spark_catalog.sh` provisions those schemas before the job runs.
The example uses S3A with the dedicated IAM credentials because RustFS STS
tokens are not currently accepted by the Unity Catalog OSS AWS client.
