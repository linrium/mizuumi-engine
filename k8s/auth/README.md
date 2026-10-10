# Keycloak Helm chart

This chart installs one Keycloak replica and, by default, a PostgreSQL StatefulSet with an 8 Gi persistent volume. It does not create an Ingress unless enabled. Kubernetes must have a default StorageClass, or set `postgresql.persistence.storageClassName`.

## Install

1. Copy `credentials.env.example` to `credentials.env` and replace both passwords. The local `credentials.env` file is ignored by Git. For a managed environment, create the Secret through your secret manager instead.
2. Run the local setup script from any directory:

   ```sh
   ./scripts/auth/setup.sh
   ```

The script uses your current Kubernetes context, creates the `auth` namespace and credentials Secret if needed, generates a local CA and CA-signed server certificate in the ignored `k8s/auth/tls/` directory, and installs or upgrades the Helm release. It leaves an existing credentials Secret unchanged, so editing `credentials.env` after the first install does not rotate the database password. If the server certificate changes, it restarts Keycloak after updating the TLS Secret.

To access Keycloak and Vault together after installation:

```sh
sudo ./scripts/configure_workstation_dns.sh # once per workstation
./scripts/forward.sh
```

Visit `https://auth.mizuumi.test` and sign in to the Administration Console with username `admin` and the `ADMIN_PASSWORD` Secret value. The auth chart runs the shared Caddy gateway in Kubernetes. `scripts/auth/setup.sh` maps the local `mizuumi.test` service names to the gateway's ClusterIP in CoreDNS, while `configure_workstation_dns.sh` maps them to `127.0.0.1` on the workstation. The gateway's LoadBalancer Service exposes every endpoint on port 443 without host-side proxies or persistent port-forwards. One gateway certificate covers all local service names, and Caddy verifies each HTTPS backend with its local CA. OpenShell is available at `https://openshell.mizuumi.test` after its setup script runs; that route requires the chart-generated client certificate. SigNoz is available at `https://console.signoz.mizuumi.test`, with OTLP/HTTP and OTLP/gRPC ingestion at `https://otlp.signoz.mizuumi.test`. On macOS, run `./scripts/forward.sh --trust --open` once to trust the gateway CA and open the browser UIs; later runs only need `--open`.

## SigNoz telemetry

Keycloak exports traces, metrics, and logs through OpenTelemetry to the local
`k8s-infra` agent, which enriches them with Kubernetes metadata and forwards
them to SigNoz. Traces use full sampling in this local environment. Native
OpenTelemetry logs and metrics are preview features in Keycloak 26; keep that
in mind when upgrading Keycloak. The regular container log stream remains
available through `kubectl logs`, but `k8s-infra` excludes the `keycloak`
container from file-log collection to avoid duplicating its native OTLP logs.

Install SigNoz and `k8s-infra` before applying the auth chart:

```sh
./scripts/signoz/setup.sh
./scripts/signoz/setup-k8s-infra.sh
./scripts/auth/setup.sh
```

## Vault login through Keycloak

After installing Keycloak and initializing/unsealing Vault, run `./scripts/auth/bootstrap-sovico.sh`. It creates the `sovico` realm, a confidential `vault` OIDC client, and an initial `khaopad` account. Bootstrap runs without prompting, writes the default password `123` to the ignored, mode-600 `k8s/auth/sovico-user-password` file, and resets the Keycloak password on rerun; set `SOVICO_USER_PASSWORD` to override it. The default profile is `Khao Pad <khaopad@mizuumi.test>`; override it with `SOVICO_USER_FIRST_NAME`, `SOVICO_USER_LAST_NAME`, and `SOVICO_USER_EMAIL`. It then configures Vault's `oidc` auth method and `sovico` role. Run `./scripts/forward.sh` before signing in to Vault at `https://vault.mizuumi.test/ui/`.

## Public HTTPS ingress

Provide an ingress controller and TLS Secret, then set `ingress.enabled=true`, `ingress.host`, `ingress.className`, `ingress.tls.enabled=true`, `ingress.tls.secretName`, and `keycloak.hostname=https://<your-host>`. Configure the ingress controller to use HTTPS for its Keycloak backend (for ingress-nginx, `nginx.ingress.kubernetes.io/backend-protocol: "HTTPS"`). Use a certificate valid for both the public host and in-cluster access in a shared installation. Ensure the ingress controller overwrites incoming `X-Forwarded-*` headers. Do not expose Keycloak's management port 9000.

## External PostgreSQL

Set `postgresql.enabled=false`, `externalDatabase.url` to a JDBC URL such as `jdbc:postgresql://db.example.com:5432/keycloak`, and `externalDatabase.username`. The Secret's `DB_PASSWORD` key remains required. Back up the database and persistent volume before upgrades; uninstalling the release does not automatically delete the StatefulSet's PVC.
