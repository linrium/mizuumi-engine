"""Shared Redpanda-to-Unity-Catalog Structured Streaming ingestion."""

from __future__ import annotations

from dataclasses import dataclass
import os
import re

from pyspark.sql import SparkSession
from pyspark.sql.functions import col, current_timestamp, from_json
from pyspark.sql.types import MapType, StringType, StructField, StructType, TimestampType


IDENTIFIER = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
TOPIC = re.compile(r"^[A-Za-z0-9._-]+$")


@dataclass(frozen=True)
class JobConfig:
    source: str
    topic: str


def required_env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise RuntimeError(f"{name} must be set")
    return value


def checked_identifier(value: str, label: str) -> str:
    if not IDENTIFIER.fullmatch(value):
        raise RuntimeError(f"Invalid {label}: {value!r}")
    return value


def event_schema() -> StructType:
    return StructType(
        [
            StructField("specversion", StringType()),
            StructField("id", StringType()),
            StructField("source", StringType()),
            StructField("type", StringType()),
            StructField("time", TimestampType()),
            StructField("datacontenttype", StringType()),
            StructField("subject", StringType()),
            StructField("data", MapType(StringType(), StringType())),
        ]
    )


def run(config: JobConfig) -> None:
    catalog = checked_identifier(required_env("UNITY_CATALOG_NAME"), "catalog name")
    if catalog != config.source:
        raise RuntimeError(
            f"The {config.source} job requires the {config.source!r} catalog, got {catalog!r}"
        )
    schema = checked_identifier(os.environ.get("UNITY_SCHEMA_NAME", "bronze"), "schema name")
    table = checked_identifier(os.environ.get("UNITY_TABLE_NAME", "events"), "table name")
    topic = os.environ.get("KAFKA_TOPIC", config.topic).strip()
    if not TOPIC.fullmatch(topic):
        raise RuntimeError(f"Invalid Kafka topic: {topic!r}")

    token_url = required_env("KEYCLOAK_TOKEN_URL")
    client_id = required_env("KEYCLOAK_CLIENT_ID")
    client_secret = required_env("KEYCLOAK_CLIENT_SECRET")
    kafka_bootstrap = os.environ.get(
        "KAFKA_BOOTSTRAP_SERVERS",
        "redpanda.redpanda.svc.cluster.local:9092",
    )
    starting_offsets = os.environ.get("KAFKA_STARTING_OFFSETS", "earliest")
    trigger_mode = os.environ.get("STREAMING_TRIGGER", "30 seconds")
    storage_root = os.environ.get("UNITY_STORAGE_ROOT", f"s3://unitycatalog/{catalog}").rstrip("/")
    checkpoint_root = storage_root.replace("s3://", "s3a://", 1)
    checkpoint = os.environ.get(
        "STREAMING_CHECKPOINT",
        f"{checkpoint_root}/_checkpoints/{topic.replace('.', '_')}",
    )
    table_name = f"{catalog}.{schema}.{table}"
    query_name = f"{config.source}-events-bronze"

    spark = SparkSession.builder.appName(query_name).getOrCreate()
    try:
        spark.sparkContext.setLogLevel("WARN")
        s3_access_key = os.environ.get("S3_ACCESS_KEY", "").strip()
        s3_secret_key = os.environ.get("S3_SECRET_KEY", "").strip()
        if s3_access_key and s3_secret_key:
            hadoop_conf = spark.sparkContext._jsc.hadoopConfiguration()
            hadoop_conf.set("fs.s3a.access.key", s3_access_key)
            hadoop_conf.set("fs.s3a.secret.key", s3_secret_key)

        catalog_auth = f"spark.sql.catalog.{catalog}.auth"
        spark.conf.set(f"{catalog_auth}.type", "oauth")
        spark.conf.set(f"{catalog_auth}.oauth.uri", token_url)
        spark.conf.set(f"{catalog_auth}.oauth.clientId", client_id)
        spark.conf.set(f"{catalog_auth}.oauth.clientSecret", client_secret)
        spark.conf.set(f"{catalog_auth}.oauth.scope", "openid")

        spark.sql(
            f"CREATE TABLE IF NOT EXISTS {table_name} ("
            "event_id STRING, "
            "event_source STRING, "
            "event_type STRING, "
            "event_time TIMESTAMP, "
            "subject STRING, "
            "data MAP<STRING, STRING>, "
            "raw_json STRING, "
            "kafka_key STRING, "
            "topic STRING, "
            "`partition` INT, "
            "`offset` BIGINT, "
            "kafka_timestamp TIMESTAMP, "
            "headers ARRAY<STRUCT<key: STRING, value: BINARY>>, "
            "ingest_ts TIMESTAMP"
            ") USING DELTA"
        )

        kafka_stream = (
            spark.readStream.format("kafka")
            .option("kafka.bootstrap.servers", kafka_bootstrap)
            .option("subscribe", topic)
            .option("startingOffsets", starting_offsets)
            .option("includeHeaders", "true")
            .option("failOnDataLoss", "false")
            .load()
        )
        parsed = kafka_stream.withColumn("raw_json", col("value").cast("string")).withColumn(
            "event", from_json(col("raw_json"), event_schema())
        )
        bronze = parsed.select(
            col("event.id").alias("event_id"),
            col("event.source").alias("event_source"),
            col("event.type").alias("event_type"),
            col("event.time").alias("event_time"),
            col("event.subject").alias("subject"),
            col("event.data").alias("data"),
            "raw_json",
            col("key").cast("string").alias("kafka_key"),
            "topic",
            "partition",
            "offset",
            col("timestamp").alias("kafka_timestamp"),
            "headers",
            current_timestamp().alias("ingest_ts"),
        )
        writer = (
            bronze.writeStream.format("delta")
            .option("checkpointLocation", checkpoint)
            .outputMode("append")
            .queryName(query_name)
        )
        if trigger_mode == "availableNow":
            query = writer.trigger(availableNow=True).toTable(table_name)
        elif trigger_mode == "once":
            query = writer.trigger(once=True).toTable(table_name)
        else:
            query = writer.trigger(processingTime=trigger_mode).toTable(table_name)

        query.awaitTermination()
        print(f"Streaming query stopped: {topic} -> {table_name}")
        if query.lastProgress is not None:
            print(f"Last progress: {query.lastProgress}")
    finally:
        spark.stop()
