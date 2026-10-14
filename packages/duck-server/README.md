# DuckDB Quack server

This package runs DuckDB 1.5.6 as a Quack server. The DuckDB catalog is
stored on the Kubernetes PVC, while the `rustfs_events` view reads Parquet data
from the dedicated `duck-server` RustFS bucket. A separate HTTP query endpoint
accepts a signed-in user's Keycloak bearer token and attaches the Mizuumi Unity
Catalog catalogs `unity`, `payments`, `vietjet`, `hdbank`, `vikki`, and `crm`
only for that request. The `unity` catalog defaults to the `demo` schema; the
other catalogs default to `bronze`. Unity Catalog therefore authorizes metadata
access and temporary S3 credentials as the user, not as Duck server or the Spark
writer. The per-request catalog attachment is read-only, so `SELECT` queries
request read-only temporary credentials and never require `MODIFY`.

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

Catalog attachments are configured by the `unityCatalog.catalogs` Helm value.
Each entry has a remote `name`, a DuckDB `alias`, and a `defaultSchema`. Direct
runs can provide the same array as JSON in `UNITY_CATALOGS`; the legacy
`UNITY_CATALOG_NAME` and `UNITY_CATALOG_DEFAULT_SCHEMA` variables remain the
single-catalog fallback when `UNITY_CATALOGS` is unset.

The explicit port is required because Quack URIs otherwise default to port
9494, even when a non-local hostname makes the client select HTTPS.

## SigNoz telemetry

The identity-aware `/api/query` path enables DuckDB's connection-local query
profiler with `no_output` and exports its root metrics to the in-cluster SigNoz
OpenTelemetry collector over OTLP/HTTP. The exported histograms cover query
latency, CPU and blocked-thread time, returned and scanned rows, result size,
operator cardinality, peak buffer and temporary-directory usage, allocated
memory, and bytes read and written. SQL text and operator names are
intentionally not exported.

The Helm chart configures the collector endpoint and a 30-second export
interval under `telemetry`. Set both `OTEL_EXPORTER_OTLP_ENDPOINT` and
`OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` to empty strings (or leave them unset) to
disable metrics for a direct run. Quack executes queries in extension-owned
connections whose completed profiles are not exposed to the Python host, so
these profiler metrics cover `/api/query`, not direct Quack clients. Kubernetes
container and pod metrics continue to come from `signoz-k8s-infra`.

The profiler also emits the engine-neutral `mizuumi.compute.operation.*`
contract used for side-by-side compute comparisons: duration, CPU time, input
and output rows/bytes, peak memory, spill, and operation count. Native
`duckdb.query.*` instruments remain available for DuckDB-specific diagnosis.

The histograms use unit-specific explicit buckets. Duration buckets are stored
in seconds with millisecond-scale boundaries; row counts and byte measurements
use separate ranges appropriate to their units. Keep these boundaries aligned
with the units above so percentile queries do not fall back to OpenTelemetry's
generic bucket layout.
