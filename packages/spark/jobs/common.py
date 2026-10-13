"""Shared bronze-to-silver batch ETL support."""

from __future__ import annotations

from dataclasses import dataclass
import json
import os
import re
from typing import Any
import urllib.error
import urllib.parse
import urllib.request
import uuid

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
    contract_version: str = "1.1.0"


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


def escaped_sql_string(value: str) -> str:
    return value.replace("\\", "\\\\").replace("'", "\\'")


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


def normalized_type(value: str) -> str:
    return "".join(value.upper().split())


def logical_type(physical_type: str) -> str:
    normalized = normalized_type(physical_type)
    if normalized == "STRING":
        return "string"
    if normalized == "DATE":
        return "date"
    if normalized == "TIMESTAMP":
        return "timestamp"
    if normalized in {"BIGINT", "INT"}:
        return "integer"
    if normalized == "BOOLEAN":
        return "boolean"
    if normalized.startswith("DECIMAL("):
        return "number"
    raise RuntimeError(f"Unsupported ODCS physical type: {physical_type}")


def contract_property(
    full_name: str, column: Column, *, primary_key: bool = False
) -> dict[str, Any]:
    property_document: dict[str, Any] = {
        "id": str(uuid.uuid5(uuid.NAMESPACE_URL, f"{full_name}#{column.name}")),
        "name": column.name,
        "logicalType": logical_type(column.data_type),
        "physicalType": column.data_type,
        "required": False,
        "primaryKey": primary_key,
        "description": column.comment,
    }
    if primary_key:
        property_document["quality"] = [
            {
                "id": f"{column.name}_not_null",
                "type": "library",
                "metric": "nullValues",
                "mustBe": 0,
                "unit": "rows",
                "dimension": "completeness",
                "severity": "error",
                "description": f"{column.name} must be present on every row",
            }
        ]
    return property_document


def contract_team(source: str) -> dict[str, Any]:
    return {
        "id": f"{source}_data_team",
        "name": f"{source.upper()} Data Team",
        "description": f"Accountable team for the {source} source domain",
        "members": [
            {
                "id": f"{source}_owner",
                "username": f"data-owner@{source}.mizuumi.test",
                "role": "Owner",
            },
            {
                "id": f"{source}_steward",
                "username": f"data-steward@{source}.mizuumi.test",
                "role": "Data Steward",
            },
        ],
    }


def contract_sla(table: Table) -> list[dict[str, Any]]:
    return [
        {
            "id": "refresh_frequency",
            "property": "frequency",
            "value": 10,
            "unit": "minutes",
            "element": f"{table.name}.ingest_ts",
            "driver": "operational",
            "description": "Expected refresh interval for the silver table",
        },
        {
            "id": "ingestion_latency",
            "property": "latency",
            "value": 15,
            "unit": "minutes",
            "element": f"{table.name}.ingest_ts",
            "driver": "operational",
            "description": "Maximum delay from ingestion to governed availability",
        },
        {
            "id": "time_to_repair",
            "property": "timeToRepair",
            "value": 4,
            "unit": "hours",
            "driver": "operational",
        },
    ]


def contract_document(config: JobConfig, table: Table, full_name: str) -> dict[str, Any]:
    technical_columns = (
        Column(
            "kafka_ts",
            TIMESTAMP,
            "Timestamp assigned to the source record by Kafka",
        ),
        Column(
            "ingest_ts",
            TIMESTAMP,
            "Timestamp when the record was ingested into the bronze table",
        ),
    )
    return {
        "apiVersion": "v3.1.0",
        "kind": "DataContract",
        "id": str(uuid.uuid5(uuid.NAMESPACE_URL, full_name)),
        "name": full_name,
        "version": table.contract_version,
        "status": "active",
        "domain": config.source,
        "dataProduct": f"{config.source} source-aligned silver data",
        "description": {
            "purpose": f"Typed silver table generated from {table.event_type} bronze events",
            "limitations": (
                "Data must remain in the Vietnam mesh unless an approved "
                "cross-mesh transfer contract exists"
            ),
            "usage": "Governed analytics and approved cross-domain data products",
        },
        "team": contract_team(config.source),
        "slaProperties": contract_sla(table),
        "customProperties": [
            {
                "id": "jurisdiction",
                "property": "jurisdiction",
                "value": "VN",
                "description": "Legal jurisdiction governing this contract",
            },
            {
                "id": "residency_countries",
                "property": "residencyCountries",
                "value": ["VN"],
                "description": "Countries in which the governed data may reside",
            },
            {
                "id": "cross_border_transfer",
                "property": "crossBorderTransferAllowed",
                "value": False,
                "description": "Requires a separately approved transfer contract",
            },
        ],
        "schema": [
            {
                "id": str(uuid.uuid5(uuid.NAMESPACE_URL, f"{full_name}#table")),
                "name": table.name,
                "logicalType": "object",
                "physicalType": "table",
                "physicalName": full_name,
                "quality": [
                    {
                        "id": f"{table.name}_not_empty",
                        "type": "library",
                        "metric": "rowCount",
                        "mustBeGreaterThan": 0,
                        "unit": "rows",
                        "dimension": "completeness",
                        "severity": "error",
                        "description": "The published table must contain data",
                    }
                ],
                "customProperties": [
                    {
                        "id": "source_event_type",
                        "property": "sourceEventType",
                        "value": table.event_type,
                    }
                ],
                "properties": [
                    contract_property(full_name, column, primary_key=index == 0)
                    for index, column in enumerate((*table.columns, *technical_columns))
                ],
            }
        ],
    }


def controlplane_request(
    method: str, path: str, body: dict[str, Any] | None = None
) -> dict[str, Any]:
    base_url = required_env("CONTROLPLANE_URL").rstrip("/")
    data = None if body is None else json.dumps(body).encode("utf-8")
    request = urllib.request.Request(
        f"{base_url}{path}",
        data=data,
        headers={"Content-Type": "application/json"},
        method=method,
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        detail = error.read().decode("utf-8", errors="replace")
        raise RuntimeError(
            f"Control Plane {method} {path} returned HTTP {error.code}: {detail}"
        ) from error
    except urllib.error.URLError as error:
        raise RuntimeError(f"Control Plane {method} {path} failed: {error.reason}") from error


def publish_contract(config: JobConfig, table: Table, full_name: str) -> dict[str, Any]:
    encoded_name = urllib.parse.quote(full_name, safe="")
    encoded_version = urllib.parse.quote(table.contract_version, safe="")
    return controlplane_request(
        "PUT",
        f"/api/data-contracts/{encoded_name}/versions/{encoded_version}",
        {
            "document": contract_document(config, table, full_name),
            "created_by": f"spark:{config.source}-bronze-to-silver",
        },
    )


def contract_table_properties(contract: dict[str, Any]) -> dict[str, str]:
    return {
        "mizuumi.odcs.id": contract["id"],
        "mizuumi.odcs.version": contract["version"],
        "mizuumi.odcs.documentHash": contract["document_hash"],
        "mizuumi.odcs.schemaHash": contract["schema_hash"],
    }


def table_properties_sql(properties: dict[str, str]) -> str:
    return ", ".join(
        f"'{escaped_sql_string(key)}' = '{escaped_sql_string(value)}'"
        for key, value in properties.items()
    )


def create_table(
    spark: SparkSession, table_name: str, table: Table, contract: dict[str, Any]
) -> None:
    columns = [
        f"{column.name} {column.data_type} COMMENT '{escaped_sql_string(column.comment)}'"
        for column in table.columns
    ]
    columns.extend(
        [
            "kafka_ts TIMESTAMP COMMENT 'Timestamp assigned to the source record by Kafka'",
            "ingest_ts TIMESTAMP COMMENT 'Timestamp when the record was ingested into the bronze table'",
        ]
    )
    properties = table_properties_sql(contract_table_properties(contract))
    spark.sql(
        f"CREATE TABLE IF NOT EXISTS {table_name} ({', '.join(columns)}) "
        f"USING DELTA TBLPROPERTIES ({properties})"
    )


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


def validate_dataframe(rows: DataFrame, contract: dict[str, Any]) -> None:
    properties = contract["document"]["schema"][0]["properties"]
    expected = [
        (
            property["name"],
            normalized_type(property["physicalType"]),
            property.get("required", False),
        )
        for property in properties
    ]
    actual = [
        (field.name, normalized_type(field.dataType.simpleString()), not field.nullable)
        for field in rows.schema.fields
    ]
    if actual != expected:
        raise RuntimeError(
            "Outgoing DataFrame does not satisfy the active data contract: "
            f"expected={expected!r}, actual={actual!r}"
        )


def reconcile_contract(full_name: str) -> None:
    encoded_name = urllib.parse.quote(full_name, safe="")
    result = controlplane_request("POST", f"/api/data-contracts/{encoded_name}/reconcile")
    if result["sync_state"] != "IN_SYNC":
        raise RuntimeError(
            f"Data contract reconciliation for {full_name} is {result['sync_state']}: "
            f"{result['differences']}"
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
    bronze_name = f"{catalog}.{bronze_schema}.{bronze_table}"
    job_name = f"{config.source}-bronze-to-silver"

    spark = SparkSession.builder.appName(job_name).getOrCreate()
    try:
        spark.sparkContext.setLogLevel("WARN")
        configure_spark(spark, catalog)
        bronze = spark.read.table(bronze_name)

        for table in config.tables:
            checked_identifier(table.name, "silver table name")
            for column in table.columns:
                checked_identifier(column.name, f"column name in {table.name}")
            table_name = f"{catalog}.{silver_schema}.{table.name}"
            contract = publish_contract(config, table, table_name)
            create_table(spark, table_name, table, contract)
            rows = silver_rows(bronze, table)
            validate_dataframe(rows, contract)
            rows.write.insertInto(table_name, overwrite=True)
            reconcile_contract(table_name)
    finally:
        spark.stop()
