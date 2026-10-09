"""Stream raw Redpanda events into a Unity Catalog bronze Delta table."""

from __future__ import annotations

import os
import re

from pyspark.sql import SparkSession


IDENTIFIER = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")


def required_env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise RuntimeError(f"{name} must be set")
    return value


def main() -> None:
    catalog = required_env("UNITY_CATALOG_NAME")
    if not IDENTIFIER.fullmatch(catalog):
        raise RuntimeError(f"Invalid catalog name: {catalog!r}")

    token_url = required_env("KEYCLOAK_TOKEN_URL")
    client_id = required_env("KEYCLOAK_CLIENT_ID")
    client_secret = required_env("KEYCLOAK_CLIENT_SECRET")

    kafka_bootstrap = os.environ.get(
        "KAFKA_BOOTSTRAP_SERVERS",
        "redpanda.redpanda.svc.cluster.local:9092",
    )
    kafka_topic = os.environ.get("KAFKA_TOPIC", "t24.payment.events")
    starting_offsets = os.environ.get("KAFKA_STARTING_OFFSETS", "earliest")
    trigger_mode = os.environ.get("STREAMING_TRIGGER", "30 seconds")
    checkpoint_location = os.environ.get(
        "STREAMING_CHECKPOINT",
        f"s3a://unitycatalog/{catalog}/_checkpoints/t24_payment_events",
    )

    schema_name = "bronze"
    table_name = f"{catalog}.{schema_name}.t24_payment_events"

    spark = SparkSession.builder.appName("t24-payment-events-bronze").getOrCreate()
    try:
        spark.sparkContext.setLogLevel("WARN")

        s3_access_key = os.environ.get("S3_ACCESS_KEY", "").strip()
        s3_secret_key = os.environ.get("S3_SECRET_KEY", "").strip()
        if s3_access_key and s3_secret_key:
            hadoop_conf = spark.sparkContext._jsc.hadoopConfiguration()
            hadoop_conf.set("fs.s3a.access.key", s3_access_key)
            hadoop_conf.set("fs.s3a.secret.key", s3_secret_key)

        catalog_conf = f"spark.sql.catalog.{catalog}.auth"
        spark.conf.set(f"{catalog_conf}.type", "oauth")
        spark.conf.set(f"{catalog_conf}.oauth.uri", token_url)
        spark.conf.set(f"{catalog_conf}.oauth.clientId", client_id)
        spark.conf.set(f"{catalog_conf}.oauth.clientSecret", client_secret)
        spark.conf.set(f"{catalog_conf}.oauth.scope", "openid")

        spark.sql(
            f"CREATE TABLE IF NOT EXISTS {table_name} ("
            "kafka_key BINARY, "
            "kafka_value BINARY, "
            "topic STRING, "
            "`partition` INT, "
            "`offset` BIGINT, "
            "kafka_timestamp TIMESTAMP, "
            "timestamp_type INT, "
            "headers ARRAY<STRUCT<key: STRING, value: BINARY>>, "
            "ingest_ts TIMESTAMP"
            ") USING DELTA"
        )

        kafka_stream = (
            spark.readStream.format("kafka")
            .option("kafka.bootstrap.servers", kafka_bootstrap)
            .option("subscribe", kafka_topic)
            .option("startingOffsets", starting_offsets)
            .option("includeHeaders", "true")
            .option("failOnDataLoss", "false")
            .load()
        )

        bronze = kafka_stream.selectExpr(
            "key AS kafka_key",
            "value AS kafka_value",
            "topic",
            "partition",
            "offset",
            "timestamp AS kafka_timestamp",
            "timestampType AS timestamp_type",
            "headers",
            "current_timestamp() AS ingest_ts",
        )

        writer = (
            bronze.writeStream.format("delta")
            .option("checkpointLocation", checkpoint_location)
            .outputMode("append")
            .queryName("t24-payment-events-bronze")
        )

        if trigger_mode == "availableNow":
            query = writer.trigger(availableNow=True).toTable(table_name)
        elif trigger_mode == "once":
            query = writer.trigger(once=True).toTable(table_name)
        else:
            query = writer.trigger(processingTime=trigger_mode).toTable(table_name)

        query.awaitTermination()
        print(f"Streaming batch finished: {table_name}")
        progress = query.lastProgress
        if progress is not None:
            print(f"Last progress: {progress}")
    finally:
        spark.stop()


if __name__ == "__main__":
    main()
