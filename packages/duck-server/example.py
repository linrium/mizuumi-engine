#!/usr/bin/env python3
"""Connect to the local Mizuumi Quack endpoint and query both storage layers."""

from __future__ import annotations

import os
from pathlib import Path

import duckdb


def sql_string(value: str) -> str:
    return "'" + value.replace("'", "''") + "'"


token = os.environ["QUACK_TOKEN"]
uri = os.environ.get("QUACK_URI", "quack:query.mizuumi.test:443")
ca_file = os.environ.get(
    "QUACK_CA_FILE",
    str(Path(__file__).parents[2] / "k8s" / "auth" / "tls" / "ca.crt"),
)

connection = duckdb.connect()
connection.execute("INSTALL quack")
connection.execute(f"SET ca_cert_file = {sql_string(ca_file)}")
connection.execute("SET enable_server_cert_verification = true")
connection.execute("LOAD quack")

print("Persistent DuckDB table:")
connection.sql(
    f"""
    FROM quack_query(
      {sql_string(uri)},
      'FROM hello',
      token = {sql_string(token)}
    )
    """
).show()
print("Parquet data read by the server from RustFS:")
connection.sql(
    f"""
    FROM quack_query(
      {sql_string(uri)},
      'FROM rustfs_events ORDER BY id',
      token = {sql_string(token)}
    )
    """
).show()
print("Managed Delta table read through Unity Catalog by the Quack server:")
connection.sql(
    f"""
    FROM quack_query(
      {sql_string(uri)},
      'FROM unity.demo.orders ORDER BY order_id',
      token = {sql_string(token)}
    )
    """
).show()
