# Mizuumi Studio

Studio signs in through the local Keycloak realm. From the repository root, set up
the local gateway and create the Studio client before starting the app:

```bash
./scripts/forward.sh
./scripts/studio/bootstrap.sh
cd packages/studio
npm install
npm run dev
```

Open [http://localhost:3000/login](http://localhost:3000/login). The development
script gives Node the local gateway CA at `k8s/auth/tls/ca.crt` so Better Auth can
fetch Keycloak's discovery document. Set `NODE_EXTRA_CA_CERTS` before running the
script if your Keycloak issuer uses a different CA. Restart the dev server after
changing this variable; Node reads it only at startup.

The `/groups` page requests the Keycloak `organization:*` scope and displays only
the signed-in user's organization and organization-group memberships. The Studio
bootstrap adds the Organization Group Membership mapper to Keycloak's built-in
`organization` client scope. After bootstrapping or changing memberships, sign
out and sign in again so Keycloak issues a token with the current memberships.

The bootstrap also adds the `controlplane` audience to Studio access tokens,
creates the Control Plane reader/writer/admin client roles, and grants the local
demo users their configured roles. Studio forwards this user token to all
Control Plane requests. After changing these mappings, sign out and sign in
again before using contracts, access reviews, or pipelines.

If discovery fails, check the issuer in `.env.local` and verify it is reachable
with the local CA:

```bash
curl --fail --cacert ../../k8s/auth/tls/ca.crt \
  https://auth.mizuumi.test/realms/sovico/.well-known/openid-configuration
```

For `npm run start` or another deployment, configure Node's TLS trust for that
environment before starting Studio.

## Pi agents

The `/chat` page is a minimal OpenShell agent runner. It uses the OpenShell
TypeScript SDK to list, create, delete, and execute in sandboxes, and streams
Pi's JSONL events to the browser while a prompt runs. By default, the SDK uses
the active gateway and mTLS credentials from the local OpenShell configuration.
It obtains renewable Keycloak access tokens with the Studio service account;
`scripts/studio/bootstrap.sh` creates the required audience and role mapping.

First initialize the Vault KV secret, custom provider, and image as described in
`../openshell/examples/pi-custom-openai/README.md`. Studio reads `base_url`
from that KV v2 secret; users select the GPT model and thinking level on the
chat page. Configure the secret's location in `.env.local`:

```sh
OPENSHELL_PI_IMAGE=pi-custom-openai:local-1
OPENSHELL_PI_PROVIDER=pi-custom-openai
OPENSHELL_OIDC_ISSUER=https://auth.mizuumi.test/realms/sovico
OPENSHELL_OIDC_CLIENT_ID=studio
OPENSHELL_OIDC_CLIENT_SECRET=<studio-client-secret>
OPENSHELL_OIDC_AUDIENCE=openshell-cli
VAULT_ADDR=https://vault.mizuumi.test
VAULT_CACERT=../../k8s/auth/tls/ca.crt
VAULT_KV_MOUNT=pi-custom-openai
VAULT_KV_SECRET_PATH=config
```

Authenticate the Vault CLI (`vault login`) before starting Studio, or provide a
Vault token through the process environment. Studio reads only `base_url`; it
never reads or returns `api_key`. New sandboxes attach the existing OpenShell
provider, and OpenShell injects its endpoint-bound credential into Pi.

The chat model selector supports GPT-5.5 and GPT-5.6 Sol, Terra, and Luna. The
thinking selector is validated for the selected family. Recreate sandboxes made
with an older Studio image so their Pi model registry includes these models.

The public `vault.mizuumi.test` endpoint terminates TLS at the shared gateway,
so `VAULT_CACERT` must point to the gateway CA under `k8s/auth/tls`. The separate
CA under `k8s/vault/tls` is only for the gateway-to-Vault upstream connection.

For a non-local deployment, give the Studio workload a read-only Vault identity
for `<mount>/data/<secret-path>` instead of using a developer token.

## DuckDB SQL

The `/sql` page and Unity Catalog previews execute SQL through the Duck server's
identity-aware HTTP endpoint. Studio forwards the signed-in user's Keycloak
access token, so Unity Catalog evaluates the user's own grants. The setting is:

```sh
DUCK_QUERY_URL=https://query.mizuumi.test/api/query
```

The access token stays in the Studio server process and is never sent to the
browser. The endpoint accepts one read-only statement. The Duck server validates
the token with Keycloak and uses it for a request-scoped Unity Catalog attach.
