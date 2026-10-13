# Mizuumi synthetic engine

This CLI creates the source-shaped dataset used by the EAB demo spine. Every record is synthetic.
It always places the featured customer, Trần Minh Anh, first and surrounds her with a deterministic
Vietnamese population suitable for ingestion and entity-matching demonstrations.

The output contains:

- Vikki customers, digital accounts, and 90 days of wallet transactions.
- HDBank customers, consumer loans, and repayments.
- CRM contacts and campaign interactions, including deliberate identity drift.
- VietJet passengers and bookings, including a booking linked to its Vikki wallet payment.
- A manifest with row counts and the featured record identifiers.

Generate a small fixture:

```bash
npm install
npm run generate -- --featured-only --output ./data/minh-anh
```

Generate the default 250,000-person demo population:

```bash
npm run generate -- --output ./data/eab-demo
```

CSV generation is deterministic for a given `--seed` and `--as-of` date. Existing output is protected;
pass `--force` explicitly to replace it. Run `npm run generate -- --help` for all options.

## Publish Minh Anh to Redpanda

Generate her records, inspect the topic routing, and publish them:

```bash
npm run generate -- --featured-only --output ./data/minh-anh
npm run publish:minh-anh -- --input ./data/minh-anh --dry-run
npm run publish:minh-anh -- --input ./data/minh-anh
```

Records are separated into four source-owned topics:

- `vn.crm.events`
- `vn.hdbank.t24.events`
- `vn.vikki.t24.events`
- `vn.vietjet.events`

Topic names can be overridden with `REDPANDA_TOPIC_CRM`, `REDPANDA_TOPIC_HDBANK`,
`REDPANDA_TOPIC_VIKKI`, and `REDPANDA_TOPIC_VIETJET`.

The default Kafka bootstrap server is `redpanda-0.api.redpanda.mizuumi.test:30092`, matching the
external listener advertised by the local Redpanda deployment. Override it with `--brokers` or
`REDPANDA_BROKERS` when using a port-forward or another environment.

TLS and SASL use `REDPANDA_TLS`, `REDPANDA_SASL_USERNAME`, `REDPANDA_SASL_PASSWORD`, and
`REDPANDA_SASL_MECHANISM`, matching the Temenos simulator.

Delete only these four demo topics, including all records in them:

```bash
npm run cleanup:topics -- --dry-run
npm run cleanup:topics -- --yes
```

Cleanup checks which topics exist before deleting them. Publishing again recreates missing topics.
