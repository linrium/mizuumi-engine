#!/usr/bin/env python3
"""Run DuckDB's local Quack endpoint and an identity-aware Unity query API."""

from __future__ import annotations

import json
import math
import os
import signal
import ssl
import sys
import threading
from datetime import date, datetime, time
from decimal import Decimal
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib import error, parse, request
from uuid import UUID

import duckdb

MAX_RESULT_ROWS = 10_000
QUERY_TIMEOUT_SECONDS = 60


def required_env(name: str) -> str:
    value = os.environ.get(name, "")
    if not value:
        raise RuntimeError(f"missing required environment variable: {name}")
    return value


def sql_string(value: str) -> str:
    return "'" + value.replace("'", "''") + "'"


def sql_identifier(value: str) -> str:
    return '"' + value.replace('"', '""') + '"'


def configured_unity_catalogs() -> list[tuple[str, str, str]]:
    raw_catalogs = os.environ.get("UNITY_CATALOGS", "")
    if not raw_catalogs:
        catalog = os.environ.get("UNITY_CATALOG_NAME", "unity")
        default_schema = os.environ.get("UNITY_CATALOG_DEFAULT_SCHEMA", "demo")
        return [(catalog, "unity", default_schema)]

    try:
        catalogs = json.loads(raw_catalogs)
    except json.JSONDecodeError as exc:
        raise RuntimeError("UNITY_CATALOGS must be valid JSON") from exc
    if not isinstance(catalogs, list) or not catalogs:
        raise RuntimeError("UNITY_CATALOGS must be a non-empty JSON array")

    configured: list[tuple[str, str, str]] = []
    aliases: set[str] = set()
    for index, catalog in enumerate(catalogs):
        if not isinstance(catalog, dict):
            raise RuntimeError(f"UNITY_CATALOGS[{index}] must be an object")
        name = catalog.get("name")
        alias = catalog.get("alias", name)
        default_schema = catalog.get("defaultSchema")
        for field, value in (
            ("name", name),
            ("alias", alias),
            ("defaultSchema", default_schema),
        ):
            if not isinstance(value, str) or not value.strip():
                raise RuntimeError(
                    f"UNITY_CATALOGS[{index}].{field} must be a non-empty string"
                )
        assert isinstance(name, str)
        assert isinstance(alias, str)
        assert isinstance(default_schema, str)
        if alias in aliases:
            raise RuntimeError(f"duplicate Unity Catalog alias: {alias}")
        aliases.add(alias)
        configured.append((name, alias, default_schema))
    return configured


def keycloak_ssl_context() -> ssl.SSLContext:
    ca_certificate = required_env("KEYCLOAK_CA_CERT")
    if not os.path.isfile(ca_certificate):
        raise RuntimeError(f"KEYCLOAK_CA_CERT does not exist: {ca_certificate}")
    return ssl.create_default_context(cafile=ca_certificate)


def introspect_access_token(token: str) -> dict[str, object]:
    introspection_url = required_env("KEYCLOAK_INTROSPECTION_URL")
    if not introspection_url.startswith("https://"):
        raise RuntimeError("KEYCLOAK_INTROSPECTION_URL must use https://")

    payload = parse.urlencode(
        {
            "token": token,
            "client_id": required_env("KEYCLOAK_CLIENT_ID"),
            "client_secret": required_env("KEYCLOAK_CLIENT_SECRET"),
        }
    ).encode("ascii")
    introspection_request = request.Request(
        introspection_url,
        data=payload,
        headers={"Content-Type": "application/x-www-form-urlencoded"},
        method="POST",
    )
    try:
        with request.urlopen(
            introspection_request,
            context=keycloak_ssl_context(),
            timeout=15,
        ) as response:
            token_response = json.load(response)
    except error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(
            f"Keycloak token introspection failed with HTTP {exc.code}: {detail}"
        ) from exc
    except (error.URLError, OSError, ValueError) as exc:
        raise RuntimeError(f"Keycloak token introspection failed: {exc}") from exc

    if not isinstance(token_response, dict):
        raise RuntimeError("Keycloak introspection returned an invalid response")
    return token_response


def configure_rustfs(connection: duckdb.DuckDBPyConnection) -> str:
    endpoint = os.environ.get(
        "S3_ENDPOINT", "rustfs-svc.storage.svc.cluster.local:9000"
    )
    bucket = os.environ.get("S3_BUCKET", "duck-server")
    region = os.environ.get("S3_REGION", "us-east-1")
    ca_file = os.environ.get("S3_CA_FILE", "/etc/duck-server/trust/rustfs-ca.crt")

    connection.execute("LOAD httpfs")
    # Quack executes requests in fresh server-side connections, so transport
    # settings must live in DuckDB's global configuration.
    connection.execute(f"SET GLOBAL ca_cert_file = {sql_string(ca_file)}")
    connection.execute("SET GLOBAL enable_server_cert_verification = true")
    connection.execute(
        f"""
        CREATE OR REPLACE TEMPORARY SECRET rustfs (
          TYPE s3,
          PROVIDER config,
          KEY_ID {sql_string(required_env('S3_ACCESS_KEY'))},
          SECRET {sql_string(required_env('S3_SECRET_KEY'))},
          REGION {sql_string(region)},
          ENDPOINT {sql_string(endpoint)},
          URL_STYLE path,
          USE_SSL true,
          VERIFY_SSL true,
          SCOPE {sql_string(f's3://{bucket}')}
        )
        """
    )
    return bucket


def configure_unity_catalogs(
    connection: duckdb.DuckDBPyConnection, access_token: str
) -> list[str]:
    extension_dir = os.environ.get("DUCKDB_EXTENSION_DIR", "/opt/duckdb/extensions")
    endpoint = required_env("UNITY_CATALOG_ENDPOINT")
    s3_endpoint = required_env("S3_ENDPOINT")
    region = os.environ.get("S3_REGION", "us-east-1")

    connection.execute(
        f"LOAD {sql_string(os.path.join(extension_dir, 'delta.duckdb_extension'))}"
    )
    connection.execute(
        f"LOAD {sql_string(os.path.join(extension_dir, 'unity_catalog.duckdb_extension'))}"
    )
    connection.execute(
        f"SET ca_cert_file = {sql_string(os.environ.get('S3_CA_FILE', '/etc/duck-server/trust/rustfs-ca.crt'))}"
    )
    connection.execute("SET enable_server_cert_verification = true")
    connection.execute(
        f"""
        CREATE OR REPLACE TEMPORARY SECRET unity_catalog_auth (
          TYPE unity_catalog,
          TOKEN {sql_string(access_token)},
          ENDPOINT {sql_string(endpoint)},
          AWS_REGION {sql_string(region)},
          S3_ENDPOINT {sql_string(s3_endpoint)},
          S3_URL_STYLE path,
          S3_USE_SSL true
        )
        """
    )
    aliases = []
    for catalog, alias, default_schema in configured_unity_catalogs():
        connection.execute(
            f"""
            ATTACH {sql_string(catalog)} AS {sql_identifier(alias)} (
              TYPE unity_catalog,
              READ_ONLY,
              SECRET unity_catalog_auth,
              DEFAULT_SCHEMA {sql_string(default_schema)}
            )
            """
        )
        aliases.append(alias)
    return aliases


def json_value(value: object) -> object:
    if isinstance(value, (datetime, date, time)):
        return value.isoformat()
    if isinstance(value, (Decimal, UUID)):
        return str(value)
    if isinstance(value, bytes):
        return value.hex()
    if isinstance(value, float) and not math.isfinite(value):
        return str(value)
    if isinstance(value, (list, tuple)):
        return [json_value(item) for item in value]
    if isinstance(value, dict):
        return {str(key): json_value(item) for key, item in value.items()}
    return value


def ensure_read_only(connection: duckdb.DuckDBPyConnection, query: str) -> None:
    statements = connection.extract_statements(query)
    if len(statements) != 1:
        raise ValueError("Exactly one read-only SQL statement is required")
    statement_type = str(statements[0].type).rsplit(".", 1)[-1].upper()
    if statement_type not in {"SELECT", "EXPLAIN", "SHOW", "DESCRIBE"}:
        raise ValueError("Only read-only SELECT, EXPLAIN, SHOW, and DESCRIBE queries are allowed")


def execute_identity_query(query: str, access_token: str) -> dict[str, object]:
    connection = duckdb.connect(":memory:", config={"allow_unsigned_extensions": "true"})
    timeout = threading.Timer(QUERY_TIMEOUT_SECONDS, connection.interrupt)
    try:
        ensure_read_only(connection, query)
        configure_unity_catalogs(connection, access_token)
        timeout.start()
        result = connection.execute(query)
        description = result.description or []
        raw_rows = result.fetchmany(MAX_RESULT_ROWS + 1)
        if len(raw_rows) > MAX_RESULT_ROWS:
            raise ValueError(f"Query results are limited to {MAX_RESULT_ROWS:,} rows")
        rows = [[json_value(value) for value in row] for row in raw_rows]
        return {
            "columns": [column[0] for column in description],
            "columnTypes": [str(column[1]) for column in description],
            "rows": rows,
        }
    finally:
        timeout.cancel()
        connection.close()


class QueryHandler(BaseHTTPRequestHandler):
    server_version = "MizuumiDuckQuery/1.0"

    def send_json(self, status: HTTPStatus, payload: dict[str, object]) -> None:
        body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:
        if self.path == "/health":
            self.send_json(HTTPStatus.OK, {"status": "ok"})
        else:
            self.send_json(HTTPStatus.NOT_FOUND, {"error": "Not found"})

    def do_POST(self) -> None:
        if self.path != "/api/query":
            self.send_json(HTTPStatus.NOT_FOUND, {"error": "Not found"})
            return

        authorization = self.headers.get("Authorization", "")
        if not authorization.startswith("Bearer "):
            self.send_json(HTTPStatus.UNAUTHORIZED, {"error": "Bearer token required"})
            return
        access_token = authorization.removeprefix("Bearer ").strip()
        try:
            token_details = introspect_access_token(access_token)
        except RuntimeError as exc:
            print(f"Token introspection error: {exc}", file=sys.stderr)
            self.send_json(HTTPStatus.BAD_GATEWAY, {"error": "Unable to validate access token"})
            return
        if token_details.get("active") is not True:
            self.send_json(
                HTTPStatus.UNAUTHORIZED,
                {
                    "error": (
                        "Access token is invalid, expired, or missing the "
                        "Duck server audience"
                    )
                },
            )
            return
        audience_claim = token_details.get("aud", [])
        audiences = (
            {audience_claim}
            if isinstance(audience_claim, str)
            else set(audience_claim) if isinstance(audience_claim, list) else set()
        )
        if required_env("KEYCLOAK_CLIENT_ID") not in audiences:
            self.send_json(
                HTTPStatus.FORBIDDEN,
                {"error": "Access token is not intended for the Duck query service"},
            )
            return

        try:
            content_length = int(self.headers.get("Content-Length", "0"))
        except ValueError:
            content_length = 0
        if content_length <= 0 or content_length > 1_000_000:
            self.send_json(HTTPStatus.BAD_REQUEST, {"error": "Invalid request body size"})
            return
        try:
            payload = json.loads(self.rfile.read(content_length))
            query = payload.get("sql", "") if isinstance(payload, dict) else ""
            if not isinstance(query, str) or not query.strip():
                raise ValueError("A non-empty sql string is required")
            if len(query) > 100_000:
                raise ValueError("SQL is limited to 100,000 characters")
            result = execute_identity_query(query.strip(), access_token)
        except (ValueError, json.JSONDecodeError) as exc:
            self.send_json(HTTPStatus.BAD_REQUEST, {"error": str(exc)})
            return
        except duckdb.Error as exc:
            self.send_json(HTTPStatus.FORBIDDEN, {"error": str(exc)})
            return
        except Exception as exc:
            print(f"Query execution error: {exc}", file=sys.stderr)
            self.send_json(HTTPStatus.INTERNAL_SERVER_ERROR, {"error": "Unable to execute SQL"})
            return
        self.send_json(HTTPStatus.OK, result)

    def log_message(self, format: str, *args: object) -> None:
        print(f"query-api: {format % args}")


def seed_example(connection: duckdb.DuckDBPyConnection, bucket: str) -> None:
    connection.execute(
        """
        CREATE TABLE IF NOT EXISTS hello (
          id INTEGER PRIMARY KEY,
          message VARCHAR NOT NULL,
          created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
        )
        """
    )
    connection.execute(
        """
        INSERT INTO hello (id, message)
        VALUES (1, 'hello from DuckDB Quack')
        ON CONFLICT (id) DO NOTHING
        """
    )

    parquet_path = f"s3://{bucket}/examples/events.parquet"
    try:
        connection.execute(
            f"SELECT count(*) FROM read_parquet({sql_string(parquet_path)})"
        ).fetchone()
    except duckdb.Error:
        connection.execute(
            f"""
            COPY (
              SELECT *
              FROM (VALUES
                (1, 'started', current_timestamp),
                (2, 'connected-to-rustfs', current_timestamp)
              ) AS events(id, event, created_at)
            ) TO {sql_string(parquet_path)} (FORMAT parquet, COMPRESSION zstd)
            """
        )

    connection.execute(
        f"""
        CREATE OR REPLACE VIEW rustfs_events AS
        SELECT * FROM read_parquet({sql_string(parquet_path)})
        """
    )


def main() -> int:
    database_path = os.environ.get("DUCKDB_DATABASE", "/var/lib/duckdb/mizuumi.duckdb")
    listen_uri = os.environ.get("QUACK_LISTEN_URI", "quack:0.0.0.0:9494")
    internal_uri = os.environ.get("QUACK_INTERNAL_URI", "quack:127.0.0.1:9494")
    token = required_env("QUACK_TOKEN")

    connection = duckdb.connect(
        database_path, config={"allow_unsigned_extensions": "true"}
    )
    bucket = configure_rustfs(connection)
    seed_example(connection, bucket)
    connection.execute("LOAD quack")
    result = connection.execute(
        f"""
        CALL quack_serve(
          {sql_string(listen_uri)},
          token = {sql_string(token)},
          allow_other_hostname = true
        )
        """
    ).fetchone()

    # Quack's dispatcher has its own global-setting slot.
    # Apply the CA through a loopback Quack request so every later request
    # connection inherits it. The RustFS secret itself is already process-wide.
    ca_file = os.environ.get("S3_CA_FILE", "/etc/duck-server/trust/rustfs-ca.crt")
    initialization_client = duckdb.connect()
    try:
        initialization_client.execute("LOAD quack")
        for setting in (
            f"SET GLOBAL ca_cert_file = {sql_string(ca_file)}",
            "SET GLOBAL enable_server_cert_verification = true",
        ):
            initialization_client.execute(
                f"""
                FROM quack_query(
                  {sql_string(internal_uri)},
                  {sql_string(setting)},
                  token = {sql_string(token)},
                  disable_ssl = true
                )
                """
            ).fetchall()
    finally:
        initialization_client.close()

    print(
        f"DuckDB Quack is listening at {result[0]}; RustFS bucket: {bucket}; "
        "Unity catalogs are attached only for authenticated user requests"
    )

    stopped = threading.Event()
    query_listen = os.environ.get("QUERY_HTTP_LISTEN", "0.0.0.0:9495")
    query_host, query_port = query_listen.rsplit(":", 1)
    query_server = ThreadingHTTPServer((query_host, int(query_port)), QueryHandler)
    query_server.daemon_threads = True
    query_thread = threading.Thread(
        target=query_server.serve_forever, name="identity-query-api", daemon=True
    )
    query_thread.start()
    print(f"Identity-aware query API is listening at http://{query_listen}/api/query")

    def stop(_signum: int, _frame: object) -> None:
        stopped.set()

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    stopped.wait()
    query_server.shutdown()
    query_server.server_close()
    query_thread.join(timeout=15)
    connection.execute(f"CALL quack_stop({sql_string(listen_uri)})")
    connection.close()
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"duck-server failed: {error}", file=sys.stderr)
        raise
