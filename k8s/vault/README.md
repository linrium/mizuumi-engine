# Local persistent Vault

This uses the official HashiCorp Vault Helm chart 0.34.1 in standalone server mode, not dev mode. Vault uses a 10 Gi PVC, TLS, and manual unseal. It is a single-node local setup, not an HA production deployment. The generated certificate is self-signed and only suitable for this local cluster.

## SigNoz telemetry

Vault exposes Prometheus metrics at its internal TLS endpoint, and the cluster-level `signoz-k8s-infra` OpenTelemetry Collector scrapes them every 30 seconds before exporting them to SigNoz. The collector assigns `service.name=vault`, the `vault` service and Kubernetes namespaces, the `local` deployment environment, and the `docker-desktop` cluster name. The shared Kubernetes log collector also sends Vault's structured operational logs to SigNoz. Vault does not natively emit OpenTelemetry traces, so this integration provides metrics and logs rather than synthetic traces.

The metrics API is unauthenticated so the in-cluster collector can scrape it without storing a Vault token. The public Caddy gateway returns `404` for `/v1/sys/metrics`; the endpoint remains available only through the internal `vault.vault.svc.cluster.local` service. This is a local development arrangement. Use authenticated scraping and a trusted metrics client CA for production.

Install or upgrade the components individually in this order:

```sh
./scripts/signoz/setup.sh
./scripts/vault/setup.sh
./scripts/vault/unseal.sh     # existing initialized Vault only
./scripts/signoz/setup-k8s-infra.sh
```

From the repository root, run `./scripts/bootstrap.sh` for the full Keycloak → Vault → RustFS setup (or `./scripts/bootstrap.sh --no-forward` to skip the final gateway check). Use `./scripts/bootstrap.sh --fresh-vault` only when intentionally replacing a local Vault volume; it backs up the old `init.json` before initializing the new empty Vault. To run Vault's steps individually:

```sh
./scripts/vault/setup.sh
./scripts/vault/init.sh       # once only
./scripts/auth/bootstrap-sovico.sh # Keycloak realm and Vault OIDC
sudo ./scripts/configure_workstation_dns.sh # once per workstation
./scripts/forward.sh
```

`scripts/vault/setup.sh` creates a local CA and CA-signed server certificate in `k8s/vault/tls/` (ignored by Git), creates the `vault-tls` Kubernetes Secret, and adds the CA to the shared gateway's upstream trust bundle. If the server certificate changes, it restarts Vault and unseals it when the local init file is available. `scripts/vault/init.sh` saves the unseal key and initial root token to ignored, mode-600 `k8s/vault/init.json`. Back up this file securely outside the machine. A later restart seals Vault; run `./scripts/vault/unseal.sh` before using it again. The UI is `https://vault.mizuumi.test/ui/`; using a dedicated hostname prevents cookies from the other local services from overflowing Vault's request headers. On macOS, `./scripts/forward.sh --trust --open` trusts the gateway CA and opens the browser UIs without certificate warnings.

If Vault reports that it is uninitialized while `init.json` already exists, the file belongs to another Vault volume. Restore the original volume to keep its data. For an intentional fresh local start, rerun `./scripts/bootstrap.sh --fresh-vault`; bootstrap backs up `init.json` before creating new unseal material. Do not use the old unseal key with a new volume.

The old dev-mode StatefulSet has no PVC, and Kubernetes cannot add one in place. If `scripts/vault/setup.sh` detects it, it stops before changing anything. After exporting any dev data you need, run `helm -n vault uninstall vault`, then rerun the setup, init, and bootstrap commands. The old in-memory Vault state is destroyed by that uninstall. Keycloak data remains in its PostgreSQL PVC.

`scripts/auth/bootstrap-sovico.sh` creates/updates the `sovico` Keycloak realm, `vault` confidential OIDC client, and `khaopad`. It grants only Vault's `default` policy. The local browser callback is `https://vault.mizuumi.test/ui/vault/auth/oidc/oidc/callback`. Vault discovers Keycloak through the canonical `https://auth.mizuumi.test` issuer; the stored `oidc_discovery_ca_pem` trusts the local Keycloak CA, and CoreDNS sends the connection to the in-cluster gateway. Keep the Vault root token for administration only; day-to-day login should use Keycloak.

Vault Transit is bootstrapped by `scripts/storage/bootstrap.sh` for RustFS. Back up the Vault PVC as well as the unseal material: losing either can make SSE-KMS objects permanently unreadable. For a real production cluster, replace local storage, self-signed TLS, manual procedures, and single-node Vault with managed durable storage, trusted TLS, audited access, and an HA/auto-unseal design.
