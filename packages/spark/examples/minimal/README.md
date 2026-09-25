# Minimal PySpark medallion example

This example runs a Bronze → Silver → Gold Delta Lake pipeline on Kubernetes.
It uses:

- Kubeflow Spark Operator to create the Spark driver and executor pods.
- Unity Catalog OSS for user authentication, authorization, and medallion
  schema discovery.
- RustFS as the S3-compatible storage backend.
- A short-lived RustFS STS session for Spark data access.
- Vault as the source of the permanent RustFS service credentials used by the
  deployment scripts.

## Data flow

```text
generated order rows
        |
        v
s3://unitycatalog/bronze/orders_raw
        |
        | remove invalid rows and duplicate order IDs
        v
s3://unitycatalog/silver/orders
        |
        | aggregate orders and revenue by date
        v
s3://unitycatalog/gold/daily_sales
```

The example overwrites these paths, so rerunning it does not append duplicate
records.

## Authentication and credential flow

There are two independent credentials: a Unity Catalog user token and a RustFS
STS session.

```text
User
  |
  | bin/uc auth login
  v
Keycloak ── authenticated user token ──► setup_spark.sh
                                           |
                                           v
                              spark-runtime-credentials
                                           |
                                           v
                                  Spark driver ──► Unity Catalog

Vault
  |
  | setup_unitycatalog.sh
  v
tower/unitycatalog-credentials
  |
  | setup_spark.sh signs AssumeRole request
  v
RustFS STS ── temporary key + secret + session token
  |
  v
spark/spark-runtime-credentials ──► driver and executors ──► RustFS S3 API
```

### Unity Catalog user token

The Unity Catalog CLI starts an interactive OAuth login with Keycloak. After
the user authenticates, the resulting token identifies that user to Unity
Catalog. Unity Catalog then applies its own grants for the user's email.

Obtain the token and pass it to setup:

```sh
export UNITYCATALOG_USER_TOKEN="$(
  bin/uc auth login --output jsonPretty | jq -r .access_token
)"
./scripts/setup_spark.sh
```

Alternatively, write only the raw token to a protected file:

```sh
chmod 600 /path/to/uc-token
UNITYCATALOG_USER_TOKEN_FILE=/path/to/uc-token ./scripts/setup_spark.sh
```

The setup script stores it as `UNITY_CATALOG_TOKEN` in the
`spark-runtime-credentials` Kubernetes Secret. The PySpark application reads
that environment variable and configures:

```text
spark.sql.catalog.unity.token
```

The driver then connects to:

```text
http://unitycatalog-server.tower.svc.cluster.local:8080
```

The token is not baked into the image or Helm values. It is also not
automatically refreshed. Authenticate and rerun setup when it expires.

### RustFS temporary session

Permanent RustFS credentials are originally stored in Vault. The Unity Catalog
setup copies them into the `tower/unitycatalog-credentials` Secret for the UC
server.

`setup_spark.sh` uses those credentials only on the setup workstation to sign a
RustFS `AssumeRole` request. The request includes a session policy restricted
to the `unitycatalog` bucket. RustFS returns:

```text
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
AWS_SESSION_TOKEN
```

Although the first two names resemble permanent AWS credentials, they are
temporary when accompanied by `AWS_SESSION_TOKEN`. All three values belong to
one STS session and expire together. The default lifetime is 3,600 seconds:

```sh
SPARK_CREDENTIAL_TTL=3600 ./scripts/setup_spark.sh
```

Only the temporary values are copied into the Spark namespace. The permanent
RustFS access key and secret are never mounted into driver or executor pods.
Rerun setup to replace the Kubernetes Secret with a new session.

## Unity Catalog initialization

Before submitting the application, `scripts/setup_spark.sh` invokes:

```sh
./scripts/init_spark_catalog.sh
```

The initializer uses the UC server's local administrative token only for
control-plane provisioning. It idempotently creates:

```text
unity.bronze
unity.silver
unity.gold
```

`setup_unitycatalog.sh` grants `USE CATALOG` on `unity`. The initializer adds:

- `USE SCHEMA` and `CREATE TABLE` on each medallion schema
- `READ FILES`, `WRITE FILES`, and `CREATE EXTERNAL TABLE` on the
  `rustfs_unitycatalog` external location

The Spark job itself never receives the administrative token.

## Spark runtime

The Helm chart injects the user token and temporary STS session through
Secret-backed environment variables. It also configures:

- `io.unitycatalog.spark.UCSingleCatalog`
- Delta Lake's Spark session extension and catalog
- Hadoop S3A with path-style addressing
- `https://rustfs-svc.storage.svc.cluster.local:9000`
- A Java truststore containing the local RustFS CA

The job verifies access to the three Unity Catalog schemas. Delta reads and
writes use their corresponding RustFS paths and the temporary STS session.

## Current vending limitation

The ideal flow is:

```text
Spark user token ──► Unity Catalog ──► RustFS STS ──► scoped credentials
```

That direct flow is configured by the Unity Catalog Spark connector, but it is
not currently usable with this local combination of Unity Catalog OSS `0.6.0`
and RustFS `1.0.0`: RustFS rejects the Java STS request emitted by the UC
server with `403 invalid security token`. Equivalent `AssumeRole` requests
from standard CLI clients succeed.

For that reason, `setup_spark.sh` performs the RustFS STS exchange before
submitting the job. This preserves short-lived storage credentials, but they
are valid for the whole Spark application rather than being renewed by Unity
Catalog during execution.

## Run and inspect

```sh
export UNITYCATALOG_USER_TOKEN="$(
  bin/uc auth login --output jsonPretty | jq -r .access_token
)"
./scripts/setup_spark.sh

kubectl -n spark get sparkapplication medallion-minimal
kubectl -n spark logs -l spark-role=driver --tail=-1
```

Successful output includes the three Unity Catalog schemas and:

```text
+----------+-----------+-------------+
|order_date|order_count|gross_revenue|
+----------+-----------+-------------+
|2026-09-24|2          |138.9        |
|2026-09-25|1          |25.5         |
+----------+-----------+-------------+
```
