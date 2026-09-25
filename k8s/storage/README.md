# RustFS on local Kubernetes

This installs the [official RustFS Helm chart](https://docs.rustfs.com/en/installation/cloud-native) (`rustfs/rustfs` 1.0.0, RustFS 1.0.0) in the `storage` namespace as one standalone instance with 8 Gi data and 1 Gi log PVCs. It exposes no Ingress of its own. The shared in-cluster Caddy gateway serves the Console at `https://storage.mizuumi.test/rustfs/console/` and the S3 API at `https://api.storage.mizuumi.test`. On the Console origin, `/rustfs/console/*` goes to the console listener while all other paths—including `/`—go directly to the API listener; this preserves the host used by the Console's signed Admin and S3 requests.

For the full local setup from the repository root, run `./scripts/bootstrap.sh`. It runs the steps below and verifies the gateway. Use `./scripts/bootstrap.sh --no-forward` to skip that final check.

The individual steps are:

```sh
./scripts/setup_auth.sh
./scripts/setup_vault.sh
./scripts/init_vault.sh        # first Vault install only; otherwise unseal_vault.sh
./scripts/bootstrap_sovico.sh
./scripts/bootstrap_storage.sh
./scripts/setup_storage.sh
sudo ./scripts/configure_workstation_dns.sh # once per workstation
./scripts/forward.sh
```

On macOS, use `./scripts/forward.sh --trust --open` the first time. This
trusts the gateway CA in your keychain and opens the browser UIs without
certificate warnings. Trust persists, so later runs only need
`./scripts/forward.sh --open`. The local Kubernetes environment must provide
LoadBalancer integration; Docker Desktop does this automatically.

The storage bootstrap creates the `rustfs-console` confidential Keycloak client in the `sovico` realm, a `groups` claim mapper and a `readonly` group for the initial `khaopad`. It also enables Vault Transit, creates a non-exportable `rustfs` key and a restricted token, and creates the `rustfs-credentials` and `rustfs-vault-ca` Kubernetes Secrets. The Vault policy permits encrypt/decrypt for the `rustfs` key and listing Transit key names for RustFS's startup health check. The CA Secret contains a bundle for Vault and Keycloak. The generated root access/secret keys and client/KMS secrets are held in ignored, mode-600 `k8s/storage/credentials.env`, never in Helm values. Back up this file securely. The Vault token has a 720-hour TTL; renew or rotate it before expiration.

`setup_storage.sh` generates a local CA and a CA-signed server certificate in ignored `k8s/storage/tls/`, then creates the `rustfs-tls` Secret. RustFS uses TLS on both listeners. The pinned chart renders HTTP health probes, so the setup script patches the probes to HTTPS after each Helm upgrade and waits for the rollout. CoreDNS sends RustFS's requests for the canonical `https://auth.mizuumi.test` issuer to the in-cluster Keycloak gateway.

After running `forward.sh`, open the Console at `https://storage.mizuumi.test/rustfs/console/` and use `https://api.storage.mizuumi.test` as the S3 endpoint. The OIDC callback is `https://api.storage.mizuumi.test/rustfs/admin/v3/oidc/callback/default`, and Keycloak remains available at `https://auth.mizuumi.test` during login. `RUSTFS_EXTRA_CA_CERT` supplies the local Keycloak CA for OIDC discovery, and `RUSTFS_KMS_VAULT_CA_CERT` supplies Vault's CA for KMS. `SSL_CERT_FILE` points to the same bundle for other clients. Do not enable RustFS's insecure KMS override. If you change either Keycloak or Vault certificate, rerun `bootstrap_storage.sh` and `setup_storage.sh`. If you replace the RustFS certificate, rerun `setup_storage.sh`.

This is a local integration example, not a production RustFS topology. Its single pod and local `standard` StorageClass do not provide high availability. Use a publicly trusted certificate, durable storage, suitable replicas, backups, and operational token renewal for production.
