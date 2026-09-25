# Spark runtime

The container extends Apache Spark `4.0.1` with the repository's PySpark
examples. Maven dependencies are declared by the `SparkApplication`, allowing
the Spark Operator to resolve Delta Lake, Unity Catalog, and Hadoop S3A at
submission time.

`examples/minimal` validates `unity.bronze`, `unity.silver`, and `unity.gold`
through the Unity Catalog connector and writes Delta datasets to matching
RustFS paths. Spark authenticates to Unity Catalog with a user token and
receives one-hour RustFS STS credentials from the setup script; no long-lived
RustFS keys are mounted into Spark pods.
