# Unity Catalog OSS container

This package pins the Unity Catalog OSS server to `v0.6.0`. The H2 metadata database is kept in a named volume. No Unity Catalog UI is deployed.

The Compose stack is the unauthenticated upstream-style API quickstart. Use the Helm deployment in `k8s/unitycatalog` for the integrated Keycloak, Vault, RustFS, and `uc.mizuumi.test` environment.

## Run with Docker Compose

```sh
docker compose -f packages/unitycatalog/compose.yaml up --build -d
```

The REST API is available at <http://localhost:8080/api/2.1/unity-catalog>.

Verify the quickstart catalog:

```sh
curl --fail http://localhost:8080/api/2.1/unity-catalog/catalogs
```

Stop the containers without deleting data:

```sh
docker compose -f packages/unitycatalog/compose.yaml down
```

Appending `--volumes` deletes the local H2 database. Set `UNITYCATALOG_VERSION` or `UNITYCATALOG_API_PORT` to override the defaults.
