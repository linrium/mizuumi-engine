# Keycloak Helm chart

This chart installs one Keycloak replica and, by default, a PostgreSQL StatefulSet with an 8 Gi persistent volume. It does not create an Ingress unless enabled. Kubernetes must have a default StorageClass, or set `postgresql.persistence.storageClassName`.

## Install

1. Copy `credentials.env.example` to `credentials.env` and replace both passwords. The local `credentials.env` file is ignored by Git. For a managed environment, create the Secret through your secret manager instead.
2. Run the local setup script from any directory:

   ```sh
   ./scripts/setup_auth.sh
   ```

The script uses your current Kubernetes context, creates the `auth` namespace and credentials Secret if needed, generates a local CA and CA-signed server certificate in the ignored `k8s/auth/tls/` directory, and installs or upgrades the Helm release. It leaves an existing credentials Secret unchanged, so editing `credentials.env` after the first install does not rotate the database password. If the server certificate changes, it restarts Keycloak after updating the TLS Secret.

To access Keycloak and Vault together after installation:

```sh
./scripts/forward.sh
```

Visit `https://auth.localhost` and sign in to the Administration Console with username `admin` and the `ADMIN_PASSWORD` Secret value. `forward.sh` keeps private Kubernetes tunnels on ports 8080, 8200, 9000, and 9001 while Caddy exposes the portless local HTTPS names. On macOS, run `./scripts/forward.sh --trust --open` once to trust the local CAs and open the browser UIs without certificate warnings; later runs only need `--open`. Dynamic backchannel URLs let in-cluster clients reach token and JWKS endpoints over HTTPS.

## Vault login through Keycloak

After installing Keycloak and initializing/unsealing Vault, run `./scripts/bootstrap_sovico.sh`. It creates the `sovico` realm, a confidential `vault` OIDC client, and an initial `khaopad` account. The script prompts for the account password when run interactively. In noninteractive runs it generates a password in the ignored, mode-600 file `k8s/auth/sovico-user-password`; you can instead set `SOVICO_USER_PASSWORD` in its environment. Existing user passwords are left unchanged. It then configures Vault's `oidc` auth method and `sovico` role. Run `./scripts/forward.sh` while signing in to Vault at `https://vault.localhost/ui/`.

## Public HTTPS ingress

Provide an ingress controller and TLS Secret, then set `ingress.enabled=true`, `ingress.host`, `ingress.className`, `ingress.tls.enabled=true`, `ingress.tls.secretName`, and `keycloak.hostname=https://<your-host>`. Configure the ingress controller to use HTTPS for its Keycloak backend (for ingress-nginx, `nginx.ingress.kubernetes.io/backend-protocol: "HTTPS"`). Use a certificate valid for both the public host and in-cluster access in a shared installation. Ensure the ingress controller overwrites incoming `X-Forwarded-*` headers. Do not expose Keycloak's management port 9000.

## External PostgreSQL

Set `postgresql.enabled=false`, `externalDatabase.url` to a JDBC URL such as `jdbc:postgresql://db.example.com:5432/keycloak`, and `externalDatabase.username`. The Secret's `DB_PASSWORD` key remains required. Back up the database and persistent volume before upgrades; uninstalling the release does not automatically delete the StatefulSet's PVC.
