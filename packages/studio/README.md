# Mizuumi Studio

Studio signs in through the local Keycloak realm. From the repository root, set up
the local gateway and create the Studio client before starting the app:

```bash
./scripts/forward.sh
./scripts/bootstrap_studio.sh
cd packages/studio
npm install
npm run dev
```

Open [http://localhost:3000/login](http://localhost:3000/login). The development
script gives Node the local gateway CA at `k8s/auth/tls/ca.crt` so Better Auth can
fetch Keycloak's discovery document. Set `NODE_EXTRA_CA_CERTS` before running the
script if your Keycloak issuer uses a different CA. Restart the dev server after
changing this variable; Node reads it only at startup.

If discovery fails, check the issuer in `.env.local` and verify it is reachable
with the local CA:

```bash
curl --fail --cacert ../../k8s/auth/tls/ca.crt \
  https://auth.mizuumi.test/realms/sovico/.well-known/openid-configuration
```

For `npm run start` or another deployment, configure Node's TLS trust for that
environment before starting Studio.

## Pi agents

The `/chat` page is a minimal OpenShell agent runner. It uses the local
`openshell` CLI to list, create, and delete sandboxes, and streams Pi's JSONL
events to the browser while a prompt runs.

First initialize the Vault KV secret, custom provider, and image as described in
`../openshell/examples/pi-custom-openai/README.md`. Studio reads `base_url` and
`model` from that KV v2 secret. Configure its location in `.env.local`:

```sh
OPENSHELL_PI_IMAGE=pi-custom-openai:local
OPENSHELL_PI_PROVIDER=pi-custom-openai
VAULT_ADDR=https://vault.mizuumi.test
VAULT_CACERT=../../k8s/auth/tls/ca.crt
VAULT_KV_MOUNT=pi-custom-openai
VAULT_KV_SECRET_PATH=config
```

Authenticate the Vault CLI (`vault login`) before starting Studio, or provide a
Vault token through the process environment. Studio reads only `base_url` and
`model`; it never reads or returns `api_key`. New sandboxes attach the existing
OpenShell provider, and OpenShell injects its endpoint-bound credential into Pi.

The public `vault.mizuumi.test` endpoint terminates TLS at the shared gateway,
so `VAULT_CACERT` must point to the gateway CA under `k8s/auth/tls`. The separate
CA under `k8s/vault/tls` is only for the gateway-to-Vault upstream connection.

For a non-local deployment, give the Studio workload a read-only Vault identity
for `<mount>/data/<secret-path>` instead of using a developer token.
