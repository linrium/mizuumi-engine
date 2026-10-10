#!/usr/bin/env python3
"""Run one persistent DuckDB catalog and expose it through Quack."""

from __future__ import annotations

import os
import signal
import sys
import threading

import duckdb


def required_env(name: str) -> str:
    value = os.environ.get(name, "")
    if not value:
        raise RuntimeError(f"missing required environment variable: {name}")
    return value


def sql_string(value: str) -> str:
    return "'" + value.replace("'", "''") + "'"


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

    connection = duckdb.connect(database_path)
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

    # DuckDB 1.5.3's beta Quack dispatcher has its own global-setting slot.
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

    print(f"DuckDB Quack is listening at {result[0]}; RustFS bucket: {bucket}")

    stopped = threading.Event()

    def stop(_signum: int, _frame: object) -> None:
        stopped.set()

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    stopped.wait()
    connection.execute(f"CALL quack_stop({sql_string(listen_uri)})")
    connection.close()
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"duck-server failed: {error}", file=sys.stderr)
        raise
