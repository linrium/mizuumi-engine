# Mizuumi synthetic data

TypeScript schemas for generating a consistent Vietnamese customer population across Vikki Bank,
HDBank, the group CRM, and VietJet Air, plus a Thai regional generator with localized identities.

The package keeps a canonical `people` collection and requires every organization-specific customer
record to reference a `personId`. `SyntheticDatasetSchema` also validates the copied identity fields
(name, birth date, gender, CCCD, phone, and email), preventing generators from silently producing
different identities for the same person.

Vikki Bank and HDBank records use Temenos Transact/T24 application boundaries:

- `CUSTOMER` for customer master data
- `ACCOUNT` for settlement and deposit accounts
- `FUNDS.TRANSFER` for payments and repayments
- `AA.ARRANGEMENT` for HDBank consumer loans

Vikki adds its digital wallet on top of the T24 records. HDBank adds lending arrangements and their
repayment schedules. VietJet uses reservation-native customer, booking, flight segment, and payment
records; a booking paid with `VIKKI_WALLET` can carry the matching T24 transaction reference.

The CRM schema contains contacts, campaign interactions, and poll-batch metadata. CRM identity
fields are source observations and may differ from the canonical person, allowing a later anomaly
engine to introduce controlled drift without changing the schema.

## Commands

```bash
npm install
npm run dev
npm run check
npm test
npm run build
npm start
```

The Fastify server listens on port `4020` by default and exposes one endpoint:

```bash
curl -X POST http://localhost:4020/api/v1/generate \
  -H 'content-type: application/json' \
  -d '{"region":"vn","count":10,"seed":42}'
```

When deployed locally with `scripts/synthetics/setup.sh`, the same endpoint is available at
`https://api.synthetics.mizuumi.test/api/v1/generate` through the shared gateway.

`region` accepts `vn` or `th` and defaults to `vn`. The Vietnam engine generates Vietnamese names,
CCCDs, `+84` phone numbers, Vietnamese addresses, and VND amounts. The Thailand engine generates
Thai-script names, Thai national IDs, `+66` phone numbers, Thai addresses, and THB amounts. `count`
defaults to `1` and is limited to `100` per request. `seed` is optional; provide it when the records
need to be reproducible.

The server validates the complete linked dataset and publishes one CloudEvent per person to each
organization topic for the selected region:

- `vn.synthetics.hdbank`, `vn.synthetics.vikki`, `vn.synthetics.vietjetair`
- `th.synthetics.hdbank`, `th.synthetics.vikki`, `th.synthetics.vietjetair`

Configure Redpanda with these environment variables:

| Variable | Default |
| --- | --- |
| `REDPANDA_BROKERS` | `redpanda-0.api.redpanda.mizuumi.test:30092` |
| `REDPANDA_CLIENT_ID` | `mizuumi-synthetics` |
| `REDPANDA_TLS` | `false` |
| `REDPANDA_SASL_USERNAME` | unset |
| `REDPANDA_SASL_PASSWORD` | unset |
| `REDPANDA_SASL_MECHANISM` | `scram-sha-256` |
| `REDPANDA_TOPIC_VN_HDBANK` | `vn.synthetics.hdbank` |
| `REDPANDA_TOPIC_VN_VIKKI` | `vn.synthetics.vikki` |
| `REDPANDA_TOPIC_VN_VIETJETAIR` | `vn.synthetics.vietjetair` |
| `REDPANDA_TOPIC_TH_HDBANK` | `th.synthetics.hdbank` |
| `REDPANDA_TOPIC_TH_VIKKI` | `th.synthetics.vikki` |
| `REDPANDA_TOPIC_TH_VIETJETAIR` | `th.synthetics.vietjetair` |
| `HOST` | `0.0.0.0` |
| `PORT` | `4020` |

Example response:

```json
{
  "region": "vn",
  "generatedAt": "2026-10-13T01:00:00.000Z",
  "seed": 42,
  "peopleGenerated": 10,
  "published": {
    "region": "vn",
    "topics": {
      "hdbank": "vn.synthetics.hdbank",
      "vikkiBank": "vn.synthetics.vikki",
      "vietjetAir": "vn.synthetics.vietjetair"
    },
    "messages": {
      "hdbank": 10,
      "vikkiBank": 10,
      "vietjetAir": 10,
      "total": 30
    }
  }
}
```

Import schemas from the package root:

```ts
import { SyntheticDatasetSchema, type SyntheticDataset } from "@mizuumi/synthetics"

const dataset: SyntheticDataset = SyntheticDatasetSchema.parse(input)
```
