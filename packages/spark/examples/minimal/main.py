"""Small Unity Catalog + RustFS medallion example for Kubeflow Spark."""

from __future__ import annotations

import json
import os
import re
import ssl
from urllib import error, parse, request

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


def service_principal_token() -> str:
    """Exchange the Spark service principal for a short-lived Keycloak token."""
    token_url = required_env("KEYCLOAK_TOKEN_URL")
    if not token_url.startswith("https://"):
        raise RuntimeError("KEYCLOAK_TOKEN_URL must use https://")

    client_id = required_env("KEYCLOAK_CLIENT_ID")
    client_secret = required_env("KEYCLOAK_CLIENT_SECRET")
    ca_certificate = required_env("KEYCLOAK_CA_CERT")
    if not os.path.isfile(ca_certificate):
        raise RuntimeError(f"KEYCLOAK_CA_CERT does not exist: {ca_certificate}")

    payload = parse.urlencode(
        {
            "grant_type": "client_credentials",
            "client_id": client_id,
            "client_secret": client_secret,
        }
    ).encode("ascii")
    token_request = request.Request(
        token_url,
        data=payload,
        headers={"Content-Type": "application/x-www-form-urlencoded"},
        method="POST",
    )
    try:
        with request.urlopen(
            token_request,
            context=ssl.create_default_context(cafile=ca_certificate),
            timeout=15,
        ) as response:
            token_response = json.load(response)
    except error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(
            f"Keycloak token request failed with HTTP {exc.code}: {detail}"
        ) from exc
    except (error.URLError, OSError, ValueError) as exc:
        raise RuntimeError(f"Keycloak token request failed: {exc}") from exc

    if not isinstance(token_response, dict):
        raise RuntimeError("Keycloak token response was not a JSON object")
    token = token_response.get("access_token")
    if not isinstance(token, str) or not token:
        raise RuntimeError("Keycloak token response did not contain an access_token")
    return token


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
    token = service_principal_token()
    storage_root = required_env("UNITY_STORAGE_ROOT").rstrip("/")
    if not storage_root.startswith("s3://"):
        raise RuntimeError("UNITY_STORAGE_ROOT must be an s3:// URI")

    spark = SparkSession.builder.appName("unity-catalog-rustfs-example").getOrCreate()
    spark.sparkContext.setLogLevel("WARN")

    # The catalog is initialized lazily. Exchange the client secret only in the
    # driver and pass the resulting short-lived access token to the connector.
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
