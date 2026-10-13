# Bronze-to-silver ETL jobs

The four source-domain Spark Structured Streaming jobs read the existing
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

The jobs use independent table checkpoints under
`s3a://unitycatalog/<catalog>/_checkpoints/silver/`. The deployment is idempotent and
accepts Helm options, for example:

```bash
./scripts/spark/deploy-jobs.sh --set-string application.env.ETL_TRIGGER=availableNow
```

Supported runtime environment overrides are `BRONZE_SCHEMA_NAME`, `BRONZE_TABLE_NAME`,
`SILVER_SCHEMA_NAME`, `ETL_TRIGGER`, and `ETL_CHECKPOINT_ROOT`.
