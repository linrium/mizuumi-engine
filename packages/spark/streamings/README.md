# Source-domain streaming jobs

Four Spark Structured Streaming applications consume the synthetic demo topics and write parsed
CloudEvents to source-owned Unity Catalog tables.

| Job | Redpanda topic | Managed Delta table |
| --- | --- | --- |
| CRM | `vn.crm.events` | `crm.bronze.events` |
| HDBank | `vn.hdbank.t24.events` | `hdbank.bronze.events` |
| Vikki | `vn.vikki.t24.events` | `vikki.bronze.events` |
| VietJet | `vn.vietjet.events` | `vietjet.bronze.events` |

Each bronze table retains the parsed CloudEvent identity, source, type, timestamp, subject, and data
map alongside the original JSON and Kafka key, topic, partition, offset, timestamp, and headers.
Each job has a separate checkpoint under its catalog storage root.

Deploy every job from the repository root:

```bash
./scripts/spark/deploy-streamings.sh
```

Deploy one job:

```bash
./scripts/spark/deploy-streaming-job.sh vikki
```

The deployment is idempotent: it ensures the topic, catalog, `bronze` schema, permissions, runtime
secrets, Spark Operator, and SparkApplication exist. Supported environment overrides include
`KAFKA_TOPIC_PARTITIONS`, `KAFKA_TOPIC_REPLICAS`, `SPARK_BUILD_IMAGE`, `SPARK_NAMESPACE`, and the
`SPARK_*`/`UNITYCATALOG_*` settings accepted by `scripts/spark/setup.sh`.

Runtime overrides may be passed as Helm values, for example:

```bash
./scripts/spark/deploy-streaming-job.sh crm \
  --set-string application.env.STREAMING_TRIGGER=availableNow
```

Inspect the applications and driver logs:

```bash
kubectl -n spark get sparkapplications
kubectl -n spark logs crm-events-bronze-driver --tail=-1
```
