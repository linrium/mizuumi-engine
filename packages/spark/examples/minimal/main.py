"""Minimal Bronze, Silver, and Gold pipeline using Unity Catalog and RustFS."""

import os

from pyspark.sql import DataFrame, SparkSession, functions as F
from pyspark.sql.types import DoubleType, IntegerType, StringType, StructField, StructType


CATALOG = os.getenv("UNITY_CATALOG_NAME", "unity")
STORAGE_ROOT = os.getenv("UNITY_STORAGE_ROOT", "s3://unitycatalog").rstrip("/")


def location(layer: str, name: str) -> str:
    return f"{STORAGE_ROOT}/{layer}/{name}"


def table(layer: str, name: str) -> str:
    return f"{CATALOG}.{layer}.{name}"


def write_table(frame: DataFrame, layer: str, name: str) -> None:
    # Going through UCSingleCatalog makes the connector vend and renew scoped
    # credentials before Delta accesses the table's RustFS location.
    table_name = table(layer, name)
    if frame.sparkSession.catalog.tableExists(table_name):
        frame.write.mode("overwrite").insertInto(table_name)
        return

    (
        frame.write.format("delta")
        .option("path", location(layer, name))
        .saveAsTable(table_name)
    )


def main() -> None:
    token = os.environ["UNITY_CATALOG_TOKEN"]
    spark = (
        SparkSession.builder.appName("medallion-minimal")
        .config(f"spark.sql.catalog.{CATALOG}.token", token)
        .getOrCreate()
    )
    spark.sparkContext.setLogLevel("WARN")

    schema = StructType(
        [
            StructField("order_id", StringType(), False),
            StructField("customer_id", StringType(), False),
            StructField("quantity", IntegerType(), False),
            StructField("unit_price", DoubleType(), False),
            StructField("order_date", StringType(), False),
        ]
    )
    raw_orders = [
        ("o-1001", "c-001", 2, 19.95, "2026-09-24"),
        ("o-1002", "c-002", 1, 99.00, "2026-09-24"),
        ("o-1003", "c-001", 3, 8.50, "2026-09-25"),
        ("o-1003", "c-001", 3, 8.50, "2026-09-25"),
        ("o-1004", "c-003", -1, 12.00, "2026-09-25"),
    ]

    bronze = spark.createDataFrame(raw_orders, schema).withColumn(
        "ingested_at", F.current_timestamp()
    )
    write_table(bronze, "bronze", "orders_raw")

    silver = (
        spark.table(table("bronze", "orders_raw"))
        .filter((F.col("quantity") > 0) & (F.col("unit_price") >= 0))
        .dropDuplicates(["order_id"])
        .withColumn("order_date", F.to_date("order_date"))
        .withColumn("order_total", F.round(F.col("quantity") * F.col("unit_price"), 2))
    )
    write_table(silver, "silver", "orders")

    gold = (
        spark.table(table("silver", "orders"))
        .groupBy("order_date")
        .agg(
            F.count("*").alias("order_count"),
            F.round(F.sum("order_total"), 2).alias("gross_revenue"),
        )
        .orderBy("order_date")
    )
    write_table(gold, "gold", "daily_sales")

    print("Unity Catalog medallion schemas")
    spark.sql(f"SHOW SCHEMAS IN {CATALOG}").filter(
        F.col("namespace").isin("bronze", "silver", "gold")
    ).show(truncate=False)
    print("Gold table: daily sales")
    spark.table(table("gold", "daily_sales")).orderBy("order_date").show(truncate=False)
    spark.stop()


if __name__ == "__main__":
    main()
