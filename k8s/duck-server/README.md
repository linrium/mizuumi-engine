# DuckDB Quack on local Kubernetes

This chart deploys one DuckDB 1.5.3 process with a persistent catalog and a
token-authenticated Quack endpoint. Caddy terminates TLS at
`https://query.mizuumi.test`; the pod itself serves HTTP on port 9494.

The setup provisions a dedicated `duck-server` IAM user and bucket in RustFS.
DuckDB verifies the RustFS certificate using the mounted local CA and exposes a
`rustfs_events` view over the example Parquet object.

```sh
./scripts/duck-server/bootstrap.sh
./scripts/duck-server/setup.sh
sudo ./scripts/configure_workstation_dns.sh
./scripts/forward.sh
```

Clients attach with `quack:query.mizuumi.test:443`; the explicit port selects
the shared HTTPS gateway instead of Quack's default port 9494. See
`packages/duck-server/README.md` for the client example. Quack is beta in
DuckDB 1.5.3 and provides full read/write SQL access, so protect its token and
do not expose this local deployment to an untrusted network.
