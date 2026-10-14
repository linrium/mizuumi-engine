# Control plane on local Kubernetes

This chart deploys the Rust control plane server and a single-node PostgreSQL 18 StatefulSet.

```sh
./scripts/controlplane/bootstrap.sh
```

To install or upgrade without the post-deployment smoke test, run:

```sh
./scripts/controlplane/setup.sh
```

The default release is installed in the `tower` namespace and is available through
`https://api.controlplane.mizuumi.test`. The server exposes
`/health/livez` for process liveness and `/health/readyz` for PostgreSQL-backed readiness.
All `/api/...` routes require an RS256 bearer token issued by the `sovico` realm for the
`controlplane` audience. Read-only requests require `controlplane-reader` or
`controlplane-writer`, mutations require `controlplane-writer`, and `controlplane-admin` grants
full access. `/api/auth/me` returns the authenticated principal.

`GET /api/duck-server/metrics` reads the fixed DuckDB profiler metrics from the
internal SigNoz API. Before using it, run
`scripts/controlplane/bootstrap.sh` as described in
`packages/controlplane/server/README.md`; the resulting `controlplane-signoz`
Secret contains a read-only SigNoz service-account API key under `api-key`.
The same key backs `/api/computes/{compute_id}/metrics`, which selects DuckDB
profiler metrics for Duck Server pods and Flare task/stage metrics for
SparkApplication computes.
Use `CONTROLPLANE_SKIP_IMAGE_BUILD=1` when the configured image is already available to the cluster.
The deployment consumes the `CONTROLPLANE_CLIENT_ID` and `CONTROLPLANE_CLIENT_SECRET` keys from the
`unitycatalog-credentials` Secret created by `scripts/unitycatalog/bootstrap.sh` and
`scripts/unitycatalog/setup.sh` so approved requests can update Unity Catalog permissions. The setup
script also creates `controlplane-trust` from the local Keycloak CA so the server can obtain its
Unity Catalog service token over HTTPS.

At first startup the server writes a 256-bit bootstrap token to
`/var/run/controlplane/bootstrap-token` with mode `0600`. Set
`CONTROLPLANE__AUTH__BOOTSTRAP_ENABLED=false` after Keycloak clients and role assignments are in
place.

Bootstrap idempotently seeds the `vietjet-grow-table-read` policy template. To seed or test an
already-running deployment directly:

```sh
./scripts/controlplane/seed.sh
./scripts/controlplane/test-access-request.sh
```

The access-request smoke test creates a one-hour table `SELECT` request, verifies its required
reason and duration, and confirms that it appears in the `vietjetGrow` approval queue.

