#!/usr/bin/env bash
set -euo pipefail

namespace="${DUCK_SERVER_NAMESPACE:-duck-server}"
release="${DUCK_SERVER_RELEASE:-duck-server}"

command -v kubectl >/dev/null || { echo "Missing required command: kubectl" >&2; exit 1; }

kubectl -n "$namespace" exec deployment/"$release" -- python -c '
import os
import time
import duckdb
from urllib import error, request

def quote(value):
    return "\047" + value.replace("\047", "\047\047") + "\047"

connection = duckdb.connect()
connection.execute("SET ca_cert_file = \047/etc/duck-server/trust/gateway-ca.crt\047")
connection.execute("SET enable_server_cert_verification = true")
connection.execute("LOAD quack")
token = quote(os.environ["QUACK_TOKEN"])
uri = "\047quack:query.mizuumi.test:443\047"
for attempt in range(30):
    try:
        hello = connection.execute(f"FROM quack_query({uri}, \047SELECT message FROM hello WHERE id = 1\047, token = {token})").fetchone()[0]
        events = connection.execute(f"FROM quack_query({uri}, \047SELECT count(*) FROM rustfs_events\047, token = {token})").fetchone()[0]
        databases = connection.execute(f"FROM quack_query({uri}, \047SHOW DATABASES\047, token = {token})").fetchall()
        break
    except duckdb.Error as error:
        if "Bad Gateway" not in str(error):
            raise
        if attempt == 29:
            raise
        time.sleep(1)
assert hello == "hello from DuckDB Quack"
assert events == 2
assert "unity" not in {row[0] for row in databases}

health = request.urlopen("http://127.0.0.1:9495/health", timeout=5)
assert health.status == 200
try:
    request.urlopen(request.Request("http://127.0.0.1:9495/api/query", data=b"{}", method="POST"), timeout=5)
    raise AssertionError("identity query endpoint accepted an unauthenticated request")
except error.HTTPError as response:
    assert response.code == 401

invalid_token_request = request.Request(
    "http://127.0.0.1:9495/api/query",
    data=b"{\"sql\":\"SELECT 1\"}",
    headers={"Authorization": "Bearer invalid", "Content-Type": "application/json"},
    method="POST",
)
try:
    request.urlopen(invalid_token_request, timeout=10)
    raise AssertionError("identity query endpoint accepted an invalid access token")
except error.HTTPError as response:
    assert response.code == 401

print(f"Quack, Keycloak introspection, and query isolation verified: {hello}; {events} events; no global Unity Catalog attachment")
'
