# Local persistent Vault

This uses the official HashiCorp Vault Helm chart 0.34.1 in standalone server mode, not dev mode. Vault uses a 10 Gi PVC, TLS, and manual unseal. It is a single-node local setup, not an HA production deployment. The generated certificate is self-signed and only suitable for this local cluster.

From the repository root, run `./scripts/bootstrap.sh` for the full Keycloak → Vault → RustFS setup (or `./scripts/bootstrap.sh --no-forward` to skip the final gateway check). To run Vault's steps individually:

```sh
./scripts/setup_vault.sh
./scripts/init_vault.sh       # once only
./scripts/bootstrap_sovico.sh # Keycloak realm and Vault OIDC
sudo ./scripts/configure_workstation_dns.sh # once per workstation
./scripts/forward.sh
```

`setup_vault.sh` creates a local CA and CA-signed server certificate in `k8s/vault/tls/` (ignored by Git), creates the `vault-tls` Kubernetes Secret, and adds the CA to the shared gateway's upstream trust bundle. If the server certificate changes, it restarts Vault and unseals it when the local init file is available. `init_vault.sh` saves the unseal key and initial root token to ignored, mode-600 `k8s/vault/init.json`. Back up this file securely outside the machine. A later restart seals Vault; run `./scripts/unseal_vault.sh` before using it again. The UI is `https://vault.mizuumi.test/ui/`; using a dedicated hostname prevents cookies from the other local services from overflowing Vault's request headers. On macOS, `./scripts/forward.sh --trust --open` trusts the gateway CA and opens the browser UIs without certificate warnings.

The old dev-mode StatefulSet has no PVC, and Kubernetes cannot add one in place. If `setup_vault.sh` detects it, it stops before changing anything. After exporting any dev data you need, run `helm -n vault uninstall vault`, then rerun the setup, init, and bootstrap commands. The old in-memory Vault state is destroyed by that uninstall. Keycloak data remains in its PostgreSQL PVC.

`bootstrap_sovico.sh` creates/updates the `sovico` Keycloak realm, `vault` confidential OIDC client, and `khaopad`. It grants only Vault's `default` policy. The local browser callback is `https://vault.mizuumi.test/ui/vault/auth/oidc/oidc/callback`. Vault discovers Keycloak through the canonical `https://auth.mizuumi.test` issuer; the stored `oidc_discovery_ca_pem` trusts the local Keycloak CA, and CoreDNS sends the connection to the in-cluster gateway. Keep the Vault root token for administration only; day-to-day login should use Keycloak.

Vault Transit is bootstrapped by `scripts/bootstrap_storage.sh` for RustFS. Back up the Vault PVC as well as the unseal material: losing either can make SSE-KMS objects permanently unreadable. For a real production cluster, replace local storage, self-signed TLS, manual procedures, and single-node Vault with managed durable storage, trusted TLS, audited access, and an HA/auto-unseal design.
