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
