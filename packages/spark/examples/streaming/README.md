# Redpanda to bronze streaming example

This example runs a Spark Structured Streaming job that consumes the
`t24.payment.events` topic from the in-cluster Redpanda broker and appends the
raw Kafka records (key, value, topic, partition, offset, timestamp, headers)
into the managed Delta table `payments.bronze.t24_payment_events`. The payload
is kept as `BINARY` — no schema is parsed in the bronze zone.

Deploy from the repository root once RustFS, Keycloak, Unity Catalog, and
Redpanda are up:

```bash
./scripts/setup_spark_streaming.sh
```

The script creates the `payments` catalog with a `bronze` schema and the
required grants, uploads `main.py` to RustFS, and submits the
`t24-payment-events-bronze` SparkApplication. The Spark driver pulls the
`spark-sql-kafka-0-10_2.13:4.0.1` connector via `--packages` on startup.

Environment overrides consumed by `main.py`:

- `KAFKA_BOOTSTRAP_SERVERS` (default `redpanda.redpanda.svc.cluster.local:9092`)
- `KAFKA_TOPIC` (default `t24.payment.events`)
- `KAFKA_STARTING_OFFSETS` (default `earliest`)
- `STREAMING_TRIGGER` (default `30 seconds` processing-time micro-batches; set
  to `availableNow` for a one-shot batch over all pending Kafka records, or
  `once` for a single micro-batch)
- `STREAMING_CHECKPOINT` (default
  `s3a://unitycatalog/payments/_checkpoints/t24_payment_events`)

Watch the job and inspect its output with:

```bash
kubectl -n spark get sparkapplication t24-payment-events-bronze -w
kubectl -n spark logs t24-payment-events-bronze-driver --tail=-1
```

Re-running the setup script resubmits the application and keeps the existing
table and checkpoint, so new events are appended on the next trigger.
