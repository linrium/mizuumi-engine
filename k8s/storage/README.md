# RustFS on local Kubernetes

This installs the [official RustFS Helm chart](https://docs.rustfs.com/en/installation/cloud-native) (`rustfs/rustfs` 1.0.0, RustFS 1.0.0) as one standalone instance with 8 Gi data and 1 Gi log PVCs. It exposes no Ingress. The Console and S3 API use local port-forwards.

For the full local setup from the repository root, run `./scripts/bootstrap.sh`. It runs the steps below and then keeps the local port-forwards open. Use `./scripts/bootstrap.sh --no-forward` for a non-blocking setup.

The individual steps are:

```sh
./scripts/setup_auth.sh
./scripts/setup_vault.sh
./scripts/init_vault.sh        # first Vault install only; otherwise unseal_vault.sh
./scripts/bootstrap_sovico.sh
./scripts/bootstrap_storage.sh
./scripts/setup_storage.sh
./scripts/forward.sh
```

For on-demand forwarding with [autotunnel](https://github.com/atas/autotunnel), install it with `brew install atas/tap/autotunnel`, then run `./scripts/forward.sh --autotunnel`. This uses the current Kubernetes context and the same local ports and URLs as the default script. Autotunnel starts each tunnel when a connection arrives and closes it after 60 minutes idle. Stop it with Ctrl-C; do not run both forwarding modes at once because they use the same ports.

The storage bootstrap creates the `rustfs-console` confidential Keycloak client in the `sovico` realm, a `groups` claim mapper and a `readonly` group for the initial `vault-user`. It also enables Vault Transit, creates a non-exportable `rustfs` key and a restricted token, and creates the `rustfs-credentials` and `rustfs-vault-ca` Kubernetes Secrets. The Vault policy permits encrypt/decrypt for the `rustfs` key and listing Transit key names for RustFS's startup health check. The CA Secret contains a bundle for Vault and Keycloak. The generated root access/secret keys and client/KMS secrets are held in ignored, mode-600 `k8s/storage/credentials.env`, never in Helm values. Back up this file securely. The Vault token has a 720-hour TTL; renew or rotate it before expiration.

`setup_storage.sh` generates a local CA and a CA-signed server certificate in ignored `k8s/storage/tls/`, then creates the `rustfs-tls` Secret. RustFS uses TLS on both listeners. The pinned chart renders HTTP health probes and does not expose sidecars, so the setup script patches the probes to HTTPS and adds a pod-local Keycloak tunnel after each Helm upgrade, then waits for the rollout. The tunnel lets RustFS use Keycloak's exact `https://localhost:8080` issuer from inside its pod.

After starting `forward.sh`, open the Console at `https://localhost:9001`. The S3 endpoint is `https://localhost:9000`; the OIDC callback is on port 9000. Trust `k8s/storage/tls/ca.crt` in your browser or S3 client. Keycloak must also remain forwarded at `https://localhost:8080` during login. `RUSTFS_EXTRA_CA_CERT` supplies the local Keycloak CA for OIDC discovery, and `RUSTFS_KMS_VAULT_CA_CERT` supplies Vault's CA for KMS. `SSL_CERT_FILE` points to the same bundle for other clients. Do not enable RustFS's insecure KMS override. If you change either Keycloak or Vault certificate, rerun `bootstrap_storage.sh` and `setup_storage.sh`. If you replace the RustFS certificate, rerun `setup_storage.sh`.

On Docker Desktop, a `kubectl port-forward` tunnel may drop after a Vault HTTPS request even while the Vault pod stays healthy. `forward.sh` reconnects dropped tunnels; retry a browser request if it lands during the brief reconnect.

This is a local integration example, not a production RustFS topology. Its single pod and local `standard` StorageClass do not provide high availability. Use a publicly trusted certificate, durable storage, suitable replicas, backups, and operational token renewal for production.
