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
and ingestion column has a Unity Catalog comment when its table is created. Because the
jobs use `CREATE TABLE IF NOT EXISTS`, redeploying does not change comments on tables
that already exist.

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
and `SILVER_SCHEMA_NAME`.
