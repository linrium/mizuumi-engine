# DuckDB Quack server

This package runs DuckDB 1.5.3 as a single Quack server. The DuckDB catalog is
stored on the Kubernetes PVC, while the `rustfs_events` view reads Parquet data
from the dedicated `duck-server` RustFS bucket. The server also attaches the
Mizuumi Unity Catalog as `unity`, using the `unitycatalog-spark` Keycloak
service principal and temporary S3 credentials vended by Unity Catalog.

The image builds `unity_catalog` and its matching `delta` extension from the
fork in `packages/duck-uc-extension`. It does not install DuckDB's official
`unity_catalog` binary, because that binary cannot configure the RustFS
S3-compatible endpoint used here.

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
`hello` table, `rustfs_events`, and the Spark-managed
`unity.demo.orders` Delta table through Unity Catalog. Run
`./scripts/spark/setup-managed.sh` first if that managed table does not exist.
The local gateway CA must already exist at `k8s/auth/tls/ca.crt`.

`./scripts/duck-server/bootstrap.sh` copies the existing Unity Catalog
service-principal credentials into the Duck server secret. The server exchanges
them for a principal access token when the pod starts and refreshes/re-attaches
the catalog before that token expires; no access token is stored in Kubernetes.

The explicit port is required because Quack URIs otherwise default to port
9494, even when a non-local hostname makes the client select HTTPS.
