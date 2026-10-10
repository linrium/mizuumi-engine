#!/usr/bin/env bash
set -euo pipefail

namespace="${DUCK_SERVER_NAMESPACE:-duck-server}"
release="${DUCK_SERVER_RELEASE:-duck-server}"

command -v kubectl >/dev/null || { echo "Missing required command: kubectl" >&2; exit 1; }

kubectl -n "$namespace" exec deployment/"$release" -- python -c '
import os
import time
import duckdb

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
        break
    except duckdb.Error as error:
        if "Bad Gateway" not in str(error):
            raise
        if attempt == 29:
            raise
        time.sleep(1)
assert hello == "hello from DuckDB Quack"
assert events == 2
print(f"Quack and RustFS verified: {hello}; {events} RustFS events")
'
