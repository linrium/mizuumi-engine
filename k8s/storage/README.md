# RustFS on local Kubernetes

This installs the [official RustFS Helm chart](https://docs.rustfs.com/en/installation/cloud-native) (`rustfs/rustfs` 0.12.0, RustFS 1.0.0-beta.12) as one standalone instance with 8 Gi data and 1 Gi log PVCs. It exposes no Ingress. The Console and S3 API use local port-forwards.

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

The storage bootstrap creates the `rustfs-console` confidential Keycloak client in the `sovico` realm, a `groups` claim mapper and a `readonly` group for the initial `vault-user`. It also enables Vault Transit, creates a non-exportable `rustfs` key and a restricted token, and creates the `rustfs-credentials` and `rustfs-vault-ca` Kubernetes Secrets. The latter contains a CA bundle for Vault and Keycloak. The generated root access/secret keys and client/KMS secrets are held in ignored, mode-600 `k8s/storage/credentials.env`, never in Helm values. Back up this file securely. The Vault token has a 720-hour TTL; renew or rotate it before expiration.

After starting `forward.sh`, open the Console at `http://localhost:9001`. The S3 endpoint is `http://localhost:9000`; the OIDC callback is on port 9000. Keycloak must also remain forwarded at `https://localhost:8080` during login. The browser OIDC flow and Vault KMS connectivity should be verified on your cluster before storing important objects. `SSL_CERT_FILE` points RustFS at the local Vault and Keycloak CA bundle; do not enable RustFS's insecure KMS override. If you change either local TLS certificate, rerun the bootstrap and `setup_storage.sh` to update the Secret and restart RustFS.

On Docker Desktop, a `kubectl port-forward` tunnel may drop after a Vault HTTPS request even while the Vault pod stays healthy. `forward.sh` reconnects dropped tunnels; retry a browser request if it lands during the brief reconnect.

This is a local integration example, not a production RustFS topology. Its single pod and local `standard` StorageClass do not provide high availability, and the API/Console do not themselves use TLS. Use a trusted HTTPS ingress, durable storage, suitable replicas, backups, and operational token renewal for production.
