# DuckDB Quack server

This package runs DuckDB 1.5.6 as a Quack server. The DuckDB catalog is
stored on the Kubernetes PVC, while the `rustfs_events` view reads Parquet data
from the dedicated `duck-server` RustFS bucket. A separate HTTP query endpoint
accepts a signed-in user's Keycloak bearer token and attaches Mizuumi Unity
Catalog as `unity` only for that request. Unity Catalog therefore authorizes
metadata access and temporary S3 credentials as the user, not as Duck server or
the Spark writer.

The image builds `unity_catalog` and its matching `delta` extension from the
fork in `packages/duck-uc-extension`. It does not install DuckDB's official
`unity_catalog` binary, because that binary cannot configure the RustFS
S3-compatible endpoint used here.

Quack exposes the local persistent database to callers with its shared token,
but it has no Unity Catalog attachment. Studio uses `POST /api/query`, which
accepts one read-only statement and never uses the Quack token.

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

The example connects to `quack:query.mizuumi.test:443` and queries the local
`hello` table and `rustfs_events`. The local gateway CA must already exist at
`k8s/auth/tls/ca.crt`.

`./scripts/duck-server/bootstrap.sh` creates a confidential `duck-server`
Keycloak client. Its credentials are used only to introspect incoming user
tokens. The user's access token is passed to Unity Catalog for each query and is
never stored in Kubernetes or attached to the persistent Quack database.

The explicit port is required because Quack URIs otherwise default to port
9494, even when a non-local hostname makes the client select HTTPS.
