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
Use `CONTROLPLANE_SKIP_IMAGE_BUILD=1` when the configured image is already available to the cluster.
The deployment consumes the `CONTROLPLANE_CLIENT_ID` and `CONTROLPLANE_CLIENT_SECRET` keys from the
`unitycatalog-credentials` Secret created by `scripts/unitycatalog/bootstrap.sh` and
`scripts/unitycatalog/setup.sh` so approved requests can update Unity Catalog permissions. The setup
script also creates `controlplane-trust` from the local Keycloak CA so the server can obtain its
Unity Catalog service token over HTTPS.

Bootstrap idempotently seeds the `vietjet-grow-table-read` policy template. To seed or test an
already-running deployment directly:

```sh
./scripts/controlplane/seed.sh
./scripts/controlplane/test-access-request.sh
```

The access-request smoke test creates a one-hour table `SELECT` request, verifies its required
reason and duration, and confirms that it appears in the `vietjetGrow` approval queue.

