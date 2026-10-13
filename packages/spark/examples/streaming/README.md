# Redpanda to bronze streaming example

This example runs a Spark Structured Streaming job that consumes the
`t24.payment.events` topic from the in-cluster Redpanda broker and appends the
raw Kafka records (key, value, topic, partition, offset, timestamp, headers)
into the managed Delta table `payments.bronze.t24_payment_events`. The payload
is kept as `BINARY` — no schema is parsed in the bronze zone.

Deploy from the repository root once RustFS, Keycloak, Unity Catalog, and
Redpanda are up:

```bash
./scripts/spark/setup-streaming.sh
```

The script creates the `payments` catalog with a `bronze` schema and the
required grants, uploads `main.py` to RustFS, and submits the
`t24-payment-events-bronze` SparkApplication. The reusable Spark image contains
the Kafka connector, Flare extension, and matching OpenTelemetry Java agent, so
the driver and executors do not resolve those dependencies at startup.

Flare exports Spark traces and metrics over OTLP/HTTP to the in-cluster SigNoz
collector. The streaming deployment uses job-level trace granularity: each
micro-batch keeps its SQL and job latency without creating a span for every
executor task in a continuously running application. Aggregate task duration,
throughput, shuffle, stage, JVM, and exporter metrics remain enabled. Driver
telemetry uses service `t24-payment-events-bronze-driver`; executor telemetry
uses `t24-payment-events-bronze-executor`. The existing Kubernetes collector
continues to collect pod logs, so Flare's OTLP log exporter remains disabled to
avoid duplicates.

Install SigNoz and its infrastructure collector before submitting the example:

```bash
./scripts/signoz/setup.sh
./scripts/signoz/setup-k8s-infra.sh
```

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
