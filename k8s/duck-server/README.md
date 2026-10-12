# DuckDB Quack on local Kubernetes

This chart deploys one DuckDB 1.5.6 process with a persistent local catalog, a
token-authenticated Quack endpoint, and an identity-aware Unity Catalog query
API. Caddy terminates TLS at `https://query.mizuumi.test`; the pod serves Quack
on port 9494 and HTTP queries on port 9495.

The setup provisions a dedicated `duck-server` IAM user and bucket in RustFS.
DuckDB verifies the RustFS certificate using the mounted local CA and exposes a
`rustfs_events` view over the example Parquet object. Its Unity Catalog and
Delta extensions are built from `packages/duck-uc-extension`, allowing vended
credentials to use the in-cluster RustFS S3-compatible endpoint.

```sh
./scripts/duck-server/bootstrap.sh
./scripts/duck-server/setup.sh
sudo ./scripts/configure_workstation_dns.sh
./scripts/forward.sh
```

The bootstrap creates a least-privilege Keycloak client used for token
introspection. It receives no Unity Catalog grants. Studio forwards each user's
access token to `/api/query`, and DuckDB uses it for a request-scoped Unity
Catalog attachment. A user with only `BROWSE` cannot query table data.

Clients attach with `quack:query.mizuumi.test:443`; the explicit port selects
the shared HTTPS gateway instead of Quack's default port 9494. See
`packages/duck-server/README.md` for the local client example. Quack provides
full read/write SQL access to the local DuckDB database, so protect its token
and do not expose this local deployment to an untrusted network.
