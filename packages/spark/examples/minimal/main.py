"""Small Unity Catalog + RustFS medallion example for Kubeflow Spark."""

from __future__ import annotations

import os
import re

from pyspark.sql import DataFrame, SparkSession, functions as F


IDENTIFIER = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")


def required_env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise RuntimeError(f"{name} must be set")
    return value


def checked_identifier(value: str, label: str) -> str:
    if not IDENTIFIER.fullmatch(value):
        raise RuntimeError(f"{label} is not a valid Spark identifier: {value!r}")
    return value


def write_external_delta(
    spark: SparkSession,
    frame: DataFrame,
    catalog: str,
    schema: str,
    table: str,
    storage_root: str,
) -> str:
    """Create table metadata once, then replace its data on every run."""
    full_name = f"{catalog}.{schema}.{table}"
    location = f"{storage_root}/{schema}/{table}"
    ddl = ", ".join(f"`{field.name}` {field.dataType.simpleString()}" for field in frame.schema)

    spark.sql(
        f"CREATE TABLE IF NOT EXISTS {full_name} ({ddl}) "
        f"USING DELTA LOCATION '{location}'"
    )
    view_name = f"incoming_{schema}_{table}"
    frame.createOrReplaceTempView(view_name)
    spark.sql(f"INSERT OVERWRITE TABLE {full_name} SELECT * FROM {view_name}")
    return full_name


def main() -> None:
    catalog = checked_identifier(required_env("UNITY_CATALOG_NAME"), "catalog")
    token = required_env("UNITY_CATALOG_TOKEN")
    storage_root = required_env("UNITY_STORAGE_ROOT").rstrip("/")
    if not storage_root.startswith("s3://"):
        raise RuntimeError("UNITY_STORAGE_ROOT must be an s3:// URI")

    spark = SparkSession.builder.appName("unity-catalog-rustfs-example").getOrCreate()
    spark.sparkContext.setLogLevel("WARN")

    # The catalog is initialized lazily, so the token can come from a Kubernetes
    # Secret without embedding it in the SparkApplication or Spark event logs.
    spark.conf.set(f"spark.sql.catalog.{catalog}.token", token)

    orders = spark.createDataFrame(
        [
            (1, "2026-09-27", "tea", 2, 450),
            (2, "2026-09-27", "rice", 1, 1200),
            (3, "2026-09-28", "tea", 3, 450),
            (4, "2026-09-28", "rice", 2, 1200),
        ],
        "order_id long, order_date string, product string, quantity int, unit_price int",
    )
    bronze = orders.withColumn("ingested_at", F.current_timestamp())
    bronze_table = write_external_delta(
        spark, bronze, catalog, "bronze", "orders_raw", storage_root
    )

    silver = (
        spark.table(bronze_table)
        .withColumn("order_date", F.to_date("order_date"))
        .withColumn("amount", F.col("quantity") * F.col("unit_price"))
        .select("order_id", "order_date", "product", "quantity", "amount")
    )
    silver_table = write_external_delta(
        spark, silver, catalog, "silver", "orders_clean", storage_root
    )

    gold = (
        spark.table(silver_table)
        .groupBy("order_date", "product")
        .agg(
            F.sum("quantity").alias("units"),
            F.sum("amount").alias("revenue"),
        )
    )
    gold_table = write_external_delta(
        spark, gold, catalog, "gold", "daily_product_sales", storage_root
    )

    actual_orders = spark.table(bronze_table).count()
    actual_revenue = spark.table(gold_table).agg(F.sum("revenue")).first()[0]
    if actual_orders != 4 or actual_revenue != 5850:
        raise RuntimeError(
            f"validation failed: orders={actual_orders}, revenue={actual_revenue}"
        )

    print("Unity Catalog / RustFS validation passed")
    print(f"tables: {bronze_table}, {silver_table}, {gold_table}")
    spark.table(gold_table).orderBy("order_date", "product").show(truncate=False)
    spark.stop()


if __name__ == "__main__":
    main()
