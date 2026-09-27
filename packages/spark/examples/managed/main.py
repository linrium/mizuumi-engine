"""Create and validate one Unity Catalog managed Delta table."""

from __future__ import annotations

import json
import os
import re
import ssl
from urllib import error, parse, request

from pyspark.sql import SparkSession


IDENTIFIER = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")


def required_env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise RuntimeError(f"{name} must be set")
    return value


def service_principal_token() -> str:
    token_url = required_env("KEYCLOAK_TOKEN_URL")
    if not token_url.startswith("https://"):
        raise RuntimeError("KEYCLOAK_TOKEN_URL must use https://")

    ca_certificate = required_env("KEYCLOAK_CA_CERT")
    if not os.path.isfile(ca_certificate):
        raise RuntimeError(f"KEYCLOAK_CA_CERT does not exist: {ca_certificate}")

    payload = parse.urlencode(
        {
            "grant_type": "client_credentials",
            "client_id": required_env("KEYCLOAK_CLIENT_ID"),
            "client_secret": required_env("KEYCLOAK_CLIENT_SECRET"),
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

    token = token_response.get("access_token")
    if not isinstance(token, str) or not token:
        raise RuntimeError("Keycloak token response did not contain an access_token")
    return token


def verify_managed_table(catalog_uri: str, table_name: str, token: str) -> str:
    table_url = (
        f"{catalog_uri.rstrip('/')}/api/2.1/unity-catalog/tables/"
        f"{parse.quote(table_name, safe='')}"
    )
    table_request = request.Request(
        table_url, headers={"Authorization": f"Bearer {token}"}
    )
    try:
        with request.urlopen(table_request, timeout=15) as response:
            metadata = json.load(response)
    except (error.URLError, OSError, ValueError) as exc:
        raise RuntimeError(f"Unity Catalog table lookup failed: {exc}") from exc

    if metadata.get("table_type") != "MANAGED":
        raise RuntimeError(
            f"Expected a MANAGED table, got {metadata.get('table_type')!r}"
        )
    return metadata["storage_location"]


def main() -> None:
    catalog = required_env("UNITY_CATALOG_NAME")
    if not IDENTIFIER.fullmatch(catalog):
        raise RuntimeError(f"Invalid catalog name: {catalog!r}")
    token = service_principal_token()

    spark = SparkSession.builder.appName("unity-catalog-managed-example").getOrCreate()
    try:
        spark.sparkContext.setLogLevel("WARN")
        spark.conf.set(f"spark.sql.catalog.{catalog}.token", token)

        table_name = f"{catalog}.demo.orders"
        # Omitting LOCATION asks Unity Catalog to allocate managed storage.
        spark.sql(
            f"CREATE TABLE IF NOT EXISTS {table_name} ("
            "order_id BIGINT, product STRING, quantity INT) USING DELTA"
        )
        location = verify_managed_table(
            spark.conf.get(f"spark.sql.catalog.{catalog}.uri"), table_name, token
        )

        spark.sql(
            f"INSERT OVERWRITE TABLE {table_name} VALUES "
            "(1, 'tea', 2), (2, 'rice', 1), (3, 'tea', 3)"
        )
        result = spark.table(table_name)
        if result.count() != 3 or result.agg({"quantity": "sum"}).first()[0] != 6:
            raise RuntimeError("Managed table validation failed")

        print(f"Managed table validation passed: {table_name}")
        print(f"Unity Catalog storage location: {location}")
        result.orderBy("order_id").show(truncate=False)
    finally:
        spark.stop()


if __name__ == "__main__":
    main()
