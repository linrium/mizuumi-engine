"""Shared bronze-to-silver Structured Streaming ETL support."""

from __future__ import annotations

from dataclasses import dataclass
import os
import re

from pyspark.sql import DataFrame, SparkSession
from pyspark.sql.functions import col


IDENTIFIER = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")


@dataclass(frozen=True)
class Column:
    name: str
    data_type: str
    comment: str


@dataclass(frozen=True)
class Table:
    name: str
    event_type: str
    columns: tuple[Column, ...]


@dataclass(frozen=True)
class JobConfig:
    source: str
    tables: tuple[Table, ...]


STRING = "STRING"
DATE = "DATE"
TIMESTAMP = "TIMESTAMP"
BIGINT = "BIGINT"
INT = "INT"
BOOLEAN = "BOOLEAN"
DECIMAL_5_2 = "DECIMAL(5, 2)"


def required_env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise RuntimeError(f"{name} must be set")
    return value


def checked_identifier(value: str, label: str) -> str:
    if not IDENTIFIER.fullmatch(value):
        raise RuntimeError(f"Invalid {label}: {value!r}")
    return value


def configure_spark(spark: SparkSession, catalog: str) -> None:
    s3_access_key = os.environ.get("S3_ACCESS_KEY", "").strip()
    s3_secret_key = os.environ.get("S3_SECRET_KEY", "").strip()
    if s3_access_key and s3_secret_key:
        hadoop_conf = spark.sparkContext._jsc.hadoopConfiguration()
        hadoop_conf.set("fs.s3a.access.key", s3_access_key)
        hadoop_conf.set("fs.s3a.secret.key", s3_secret_key)

    catalog_auth = f"spark.sql.catalog.{catalog}.auth"
    spark.conf.set(f"{catalog_auth}.type", "oauth")
    spark.conf.set(f"{catalog_auth}.oauth.uri", required_env("KEYCLOAK_TOKEN_URL"))
    spark.conf.set(f"{catalog_auth}.oauth.clientId", required_env("KEYCLOAK_CLIENT_ID"))
    spark.conf.set(f"{catalog_auth}.oauth.clientSecret", required_env("KEYCLOAK_CLIENT_SECRET"))
    spark.conf.set(f"{catalog_auth}.oauth.scope", "openid")


def create_table(spark: SparkSession, table_name: str, table: Table) -> None:
    columns = [
        f"{column.name} {column.data_type} COMMENT '{column.comment.replace(chr(39), chr(39) * 2)}'"
        for column in table.columns
    ]
    columns.extend(
        [
            "kafka_ts TIMESTAMP COMMENT 'Timestamp assigned to the source record by Kafka'",
            "ingest_ts TIMESTAMP COMMENT 'Timestamp when the record was ingested into the bronze table'",
        ]
    )
    spark.sql(f"CREATE TABLE IF NOT EXISTS {table_name} ({', '.join(columns)}) USING DELTA")


def silver_rows(bronze: DataFrame, table: Table) -> DataFrame:
    business_columns = [
        col("data").getItem(column.name).cast(column.data_type).alias(column.name)
        for column in table.columns
    ]
    return bronze.where(col("event_type") == table.event_type).select(
        *business_columns,
        col("kafka_timestamp").alias("kafka_ts"),
        col("ingest_ts"),
    )


def run(config: JobConfig) -> None:
    catalog = checked_identifier(required_env("UNITY_CATALOG_NAME"), "catalog name")
    if catalog != config.source:
        raise RuntimeError(
            f"The {config.source} job requires the {config.source!r} catalog, got {catalog!r}"
        )

    bronze_schema = checked_identifier(
        os.environ.get("BRONZE_SCHEMA_NAME", "bronze"), "bronze schema name"
    )
    bronze_table = checked_identifier(
        os.environ.get("BRONZE_TABLE_NAME", "events"), "bronze table name"
    )
    silver_schema = checked_identifier(
        os.environ.get("SILVER_SCHEMA_NAME", "silver"), "silver schema name"
    )
    trigger_mode = os.environ.get("ETL_TRIGGER", "30 seconds").strip()
    storage_root = os.environ.get("UNITY_STORAGE_ROOT", f"s3://unitycatalog/{catalog}").rstrip("/")
    checkpoint_root = os.environ.get(
        "ETL_CHECKPOINT_ROOT",
        f"{storage_root.replace('s3://', 's3a://', 1)}/_checkpoints/silver",
    ).rstrip("/")
    bronze_name = f"{catalog}.{bronze_schema}.{bronze_table}"
    query_prefix = f"{config.source}-bronze-to-silver"

    spark = SparkSession.builder.appName(query_prefix).getOrCreate()
    queries = []
    try:
        spark.sparkContext.setLogLevel("WARN")
        configure_spark(spark, catalog)
        bronze = spark.readStream.table(bronze_name)

        for table in config.tables:
            checked_identifier(table.name, "silver table name")
            for column in table.columns:
                checked_identifier(column.name, f"column name in {table.name}")
            table_name = f"{catalog}.{silver_schema}.{table.name}"
            create_table(spark, table_name, table)
            writer = (
                silver_rows(bronze, table)
                .writeStream.format("delta")
                .option("checkpointLocation", f"{checkpoint_root}/{table.name}")
                .outputMode("append")
                .queryName(f"{query_prefix}-{table.name}")
            )
            if trigger_mode == "availableNow":
                query = writer.trigger(availableNow=True).toTable(table_name)
            elif trigger_mode == "once":
                query = writer.trigger(once=True).toTable(table_name)
            else:
                query = writer.trigger(processingTime=trigger_mode).toTable(table_name)
            queries.append(query)

        if trigger_mode in ("availableNow", "once"):
            for query in queries:
                query.awaitTermination()
        else:
            spark.streams.awaitAnyTermination()
        failures = [query.exception() for query in queries if query.exception() is not None]
        if failures:
            raise RuntimeError(f"Silver streaming query failed: {failures[0]}")
    finally:
        for query in queries:
            if query.isActive:
                query.stop()
        spark.stop()
