# Bronze-to-silver batch jobs

The four source-domain Spark batch jobs read the existing
`<catalog>.bronze.events` table and write each CloudEvent type to a typed table in the
same catalog's `silver` schema.

| Catalog | Silver tables |
| --- | --- |
| `crm` | `contacts`, `campaign_interactions` |
| `hdbank` | `customers`, `loans`, `loan_repayments` |
| `vikki` | `customers`, `accounts`, `wallet_transactions` |
| `vietjet` | `passengers`, `bookings` |

All fields from the synthetic source CSV schemas are cast to appropriate Spark SQL
types. Every silver table also contains `kafka_ts`, copied from the bronze Kafka record
timestamp, and `ingest_ts`, copied from the bronze ingestion timestamp. Every business
and ingestion column has a Unity Catalog comment when its table is created.

Before writing, each job publishes an ODCS v3.1.0 contract version to the Control Plane,
validates the outgoing DataFrame against it, and stamps the contract ID, version, document
hash, and schema hash into the Delta table properties. Contract versions are immutable: a
schema change must also increment the table's `contract_version`. After writing, the job
asks the Control Plane to reconcile the active contract with Unity Catalog and fails if the
table is still drifted.

Control Plane calls use a dedicated `spark-controlplane` Keycloak service account with the
`controlplane-writer` client role and `controlplane` audience. The driver exchanges its client
secret for a short-lived token and sends that token as a bearer credential; executors do not
receive the Control Plane client secret. Run `scripts/unitycatalog/bootstrap.sh` and
`scripts/unitycatalog/setup.sh` after auth changes so the runtime credential Secret contains
the service account, then redeploy the jobs.

The generated ODCS v3.1.0 document also declares the source-domain owner and steward,
operational frequency and latency targets, Vietnam residency constraints, and executable
schema- and property-level quality rules. The contract page in Studio can execute supported
library rules through the identity-aware DuckDB query service. Results are aggregate evidence
stored against the exact contract version; no failed customer rows or delegated tokens are
persisted.

Deploy every ETL job from the repository root:

```bash
./scripts/spark/deploy-jobs.sh
```

Each invocation reads the complete bronze table and atomically overwrites each derived
silver table. This makes scheduled retries idempotent and avoids streaming checkpoints.
Run the deployment script again, or submit the same Spark application from Dagster, to
trigger another batch run.

The SparkApplication uses `k8s/spark/job-values.yaml`, has a `Never` restart policy,
contains no streaming-specific Spark configuration, and uses the Unity Catalog `0.6.0`
connector required for the configured Keycloak OAuth scope.

The deployment accepts additional Helm options:

```bash
./scripts/spark/deploy-jobs.sh --set-string application.driver.memory=2g
```

Supported runtime environment overrides are `BRONZE_SCHEMA_NAME`, `BRONZE_TABLE_NAME`,
`SILVER_SCHEMA_NAME`, and `CONTROLPLANE_URL`. The chart supplies `CONTROLPLANE_CLIENT_ID`,
`CONTROLPLANE_CLIENT_SECRET`, `KEYCLOAK_TOKEN_URL`, and `KEYCLOAK_CA_CERT` from the managed
runtime credentials and trust bundle.

When OTLP metrics are enabled, each table write emits the shared
`mizuumi.compute.operation.*` comparison contract. The batch jobs currently
report wall-clock duration, output rows, and operation outcome. Output rows are
captured with a Spark `Observation` attached to the existing write, so metrics
collection does not launch a second Spark action or alter benchmark timing.
Flare continues to provide the Spark-specific task, stage, shuffle, and I/O
diagnostics.
