"""Minimal Bronze, Silver, and Gold pipeline using pre-provisioned Unity Catalog schemas."""

import os

from pyspark.sql import SparkSession, functions as F
from pyspark.sql.types import DoubleType, IntegerType, StringType, StructField, StructType


CATALOG = os.getenv("UNITY_CATALOG_NAME", "unity")
STORAGE_ROOT = os.getenv("UNITY_STORAGE_ROOT", "s3://unitycatalog").rstrip("/")


def location(layer: str, name: str) -> str:
    return f"{STORAGE_ROOT}/{layer}/{name}"


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
    bronze.write.format("delta").mode("overwrite").save(location("bronze", "orders_raw"))

    silver = (
        spark.read.format("delta").load(location("bronze", "orders_raw"))
        .filter((F.col("quantity") > 0) & (F.col("unit_price") >= 0))
        .dropDuplicates(["order_id"])
        .withColumn("order_date", F.to_date("order_date"))
        .withColumn("order_total", F.round(F.col("quantity") * F.col("unit_price"), 2))
    )
    silver.write.format("delta").mode("overwrite").save(location("silver", "orders"))

    gold = (
        spark.read.format("delta").load(location("silver", "orders"))
        .groupBy("order_date")
        .agg(
            F.count("*").alias("order_count"),
            F.round(F.sum("order_total"), 2).alias("gross_revenue"),
        )
        .orderBy("order_date")
    )
    gold.write.format("delta").mode("overwrite").save(location("gold", "daily_sales"))

    print("Unity Catalog medallion schemas")
    spark.sql(f"SHOW SCHEMAS IN {CATALOG}").filter(
        F.col("namespace").isin("bronze", "silver", "gold")
    ).show(truncate=False)
    print("Gold table: daily sales")
    gold.show(truncate=False)
    spark.stop()


if __name__ == "__main__":
    main()
