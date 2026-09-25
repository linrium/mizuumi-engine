# Unity Catalog Helm chart

This chart installs the Unity Catalog OSS `v0.6.0` API server in the `tower` namespace. No Unity Catalog UI is deployed. The local integration uses a persistent H2 metadata database, RustFS object storage, Vault-managed application credentials, Keycloak authentication, and the shared gateway at `https://uc.mizuumi.test`.

## Install

Install auth, Vault, and RustFS first, then bootstrap the dedicated identities and deploy:

```sh
./scripts/bootstrap_unitycatalog.sh
./scripts/setup_unitycatalog.sh
```

`bootstrap_unitycatalog.sh` creates/updates the confidential `unitycatalog` client in the `sovico` realm, assigns `khaopad@mizuumi.test` to the initial Keycloak user, provisions a dedicated RustFS IAM user with the built-in `readwrite` policy, creates the `unitycatalog` bucket, verifies RustFS STS, and writes the OAuth and S3 credentials to Vault KV v2 at `secret/unitycatalog`.

`setup_unitycatalog.sh` reads that Vault secret over TLS, syncs it to a namespaced Kubernetes Secret, creates a JVM truststore for the local Keycloak and RustFS CAs, creates a persistent JWT signing-key Secret on first run, installs the chart, registers a RustFS storage credential and external location, creates the RustFS-backed `unity.default` namespace and matching Unity Catalog user, and grants access to that namespace. Secrets are never placed in Helm values.

```sh
sudo ./scripts/configure_workstation_dns.sh # once per workstation
./scripts/forward.sh --trust --open
```

The shared gateway terminates TLS and forwards `https://uc.mizuumi.test` directly to the Unity Catalog API server in `tower`. API clients use the normal `/api/2.1/unity-catalog` and `/api/1.0/unity-control` paths.

For the upstream CLI, set `--server https://uc.mizuumi.test` and configure its local `etc/conf/server.properties` with the authorization URL, token URL, client ID `unitycatalog`, and the client secret stored at Vault `secret/data/unitycatalog`. The CLI uses a localhost OAuth callback, which is allowed by the Keycloak client. Keep the confidential client secret out of shell history and source control.

## RustFS client settings

Managed storage is rooted at `s3://unitycatalog`. Unity Catalog obtains scoped temporary credentials from RustFS's AWS-compatible `AssumeRole` endpoint. Engines consuming those credentials must also use the RustFS endpoint `https://api.storage.mizuumi.test`, region `us-east-1`, path-style addressing, and the local RustFS CA. The S3 endpoint is deployment-specific metadata and is not included in the standard Unity Catalog temporary-credentials response.

## Configuration

Override values with repeatable `--values` or `--set` arguments to the setup script, for example:

```sh
./scripts/setup_unitycatalog.sh --set server.persistence.size=10Gi
```

The local setup uses the repository's shared Caddy gateway, so the chart-level ingress remains disabled. For another environment, set `ingress.enabled=true`, configure its host and TLS, and replace all local issuer/endpoints/CAs.

The H2 database supports a single server replica and is intended for local/development use. Keep the generated `unitycatalog-jwt` Secret, Vault KV data, RustFS IAM state/bucket, and the retained PVC when upgrading. Back up both the PVC and RustFS data before changing versions. Uninstalling the Helm release deliberately retains the PVC.
