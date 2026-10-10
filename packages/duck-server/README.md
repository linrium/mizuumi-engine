# DuckDB Quack server

This package runs DuckDB 1.5.3 as a single Quack server. The DuckDB catalog is
stored on the Kubernetes PVC, while the `rustfs_events` view reads Parquet data
from the dedicated `duck-server` RustFS bucket.

Quack currently exposes the full SQL surface to every caller that has the
token. Treat the token as an administrator credential.

## Local client example

After running the Duck server setup, install the pinned client and obtain the
token from Kubernetes:

```sh
python3 -m venv .venv
. .venv/bin/activate
pip install -r packages/duck-server/requirements.txt
export QUACK_TOKEN="$(kubectl -n duck-server get secret duck-server-credentials -o jsonpath='{.data.QUACK_TOKEN}' | base64 -d)"
python packages/duck-server/example.py
```

The example connects to `quack:query.mizuumi.test:443`, queries the persistent
`hello` table, and then queries `rustfs_events`, which is backed by Parquet in
RustFS. The local gateway CA must already exist at `k8s/auth/tls/ca.crt`.

The explicit port is required because Quack URIs otherwise default to port
9494, even when a non-local hostname makes the client select HTTPS.
