# Rust Unity Catalog on local Kubernetes

This chart deploys the Rust Unity Catalog server from `packages/unitycatalog/server` in the `tower` namespace, with an in-cluster PostgreSQL database using the `postgres:20-alpine` image by default. The shared gateway in `k8s/auth` routes `https://unitycatalog.mizuumi.test` to `unitycatalog-server.tower.svc.cluster.local:8080`.

Install or upgrade with:

```sh
./scripts/bootstrap_unitycatalog.sh
./scripts/setup_unitycatalog.sh
```

`setup_unitycatalog.sh` builds `packages/unitycatalog/server/Dockerfile` before the Helm upgrade. By default it tags the image as `mizuumi/unitycatalog-server:latest`, loads it into common local clusters (`kind`, `k3d`, and `minikube`), and restarts the server deployment so rebuilding the same tag is picked up. Override the image or skip the build with:

```sh
UNITYCATALOG_IMAGE_REPOSITORY=example/unitycatalog-server \
UNITYCATALOG_IMAGE_TAG=dev \
  ./scripts/setup_unitycatalog.sh

UNITYCATALOG_SKIP_IMAGE_BUILD=1 ./scripts/setup_unitycatalog.sh
```

`bootstrap_unitycatalog.sh` creates the RustFS IAM credential and `unitycatalog` bucket, then stores `S3_ACCESS_KEY` and `S3_SECRET_KEY` in Vault. `setup_unitycatalog.sh` copies that Vault data into the `unitycatalog-credentials` Kubernetes Secret and creates the `unitycatalog-trust` CA bundle Secret. This chart reads those keys and uses RustFS STS to vend temporary S3 credentials.

`vending.accessKeySecretKey` and `vending.secretKeySecretKey` in `values.yaml` are the names of keys inside `server.credentialsSecretName`; they are not credential values. The defaults expect:

```yaml
server:
  credentialsSecretName: unitycatalog-credentials
vending:
  accessKeySecretKey: S3_ACCESS_KEY
  secretKeySecretKey: S3_SECRET_KEY
```

To create a non-root RustFS IAM user and populate that Secret directly, run:

```sh
./scripts/create_unitycatalog_rustfs_user.sh
```

The script reads the RustFS root/admin key from `storage/rustfs-credentials` only to call RustFS admin APIs during provisioning. It creates a separate IAM user, attaches the `readwrite` policy by default, creates the `unitycatalog` bucket, validates STS with the new user key, then stores only the non-root key in `tower/unitycatalog-credentials`.

The default in-cluster RustFS endpoint is:

```text
https://rustfs-svc.storage.svc.cluster.local:9000
```

For local host access after `./scripts/forward.sh`, the public URLs are:

```text
Unity Catalog API: https://unitycatalog.mizuumi.test
RustFS S3/API:    https://api.storage.mizuumi.test
```

The checked-in `packages/unitycatalog/server/config.toml` uses the public RustFS URL for local runs. The chart overrides runtime config through `UNITYCATALOG__...` environment variables and the generated ConfigMap.

## Keycloak authentication and authorization

All routes except `/health/livez` and `/health/readyz` require a bearer token. The server accepts RS256 access tokens issued by the `sovico` realm for the `unitycatalog` audience and refreshes Keycloak's JWKS when it sees an unknown signing key. The Keycloak bootstrap creates these client roles:

| Role | Access |
| --- | --- |
| `unitycatalog-reader` | `GET`, `HEAD`, and `OPTIONS` API requests |
| `unitycatalog-writer` | Read access and API mutations except permission management |
| `unitycatalog-admin` | Full access, including `/permissions/...` |

Roles may be present as realm roles or as client roles under `resource_access.unitycatalog`. Human principals use their verified token email (then username, then subject) as their ID. Keycloak service-account tokens use the service client ID. Inspect the current mapping with `GET /api/auth/me`.

At first startup the server generates a 256-bit bootstrap token in `/var/run/unitycatalog/bootstrap-token` with mode `0600`. It has admin access and is intended only for initial provisioning. Retrieve it locally when needed:

```sh
kubectl -n tower exec deployment/unitycatalog-server -c server -- \
  cat /var/run/unitycatalog/bootstrap-token
```

The token changes when the pod is replaced because the runtime volume is ephemeral. For a controlled external secret, set `UNITYCATALOG__AUTH__BOOTSTRAP_TOKEN`. After provisioning Keycloak administrators, disable bootstrap access with `UNITYCATALOG__AUTH__BOOTSTRAP_ENABLED=false`.

Run the end-to-end authentication check after bootstrapping and deploying:

```sh
./scripts/test_unitycatalog_auth.sh
```

The test creates temporary human-reader and service-writer principals in Keycloak, verifies authentication, authorization, and bootstrap access, then deletes them.
