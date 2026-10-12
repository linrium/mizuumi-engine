# Mizuumi synthetic data

This TypeScript service generates localized, linked, and intentionally imperfect customer data for
HDBank, Vikki Bank, VietJet Air, and the group CRM. Vietnam records use Vietnamese identities,
CCCDs, `+84` phone numbers, VND, and Vietnamese addresses. Thailand records use Thai-script
identities, checksum-valid Thai national IDs, `+66` phone numbers, THB, and Thai addresses.

HDBank and Vikki records follow the Temenos Transact/T24 `CUSTOMER`, `ACCOUNT`,
`FUNDS.TRANSFER`, and `AA.ARRANGEMENT` shapes. VietJet uses reservation-native customers,
bookings, segments, and payments. CRM events contain poll metadata, contacts, and campaign
interactions.

## Generation model

Generation is a deterministic pipeline:

```text
canonical person
  -> economic and behavioural profile
  -> source coverage and product adoption
  -> financially valid source records
  -> controlled source-specific identity anomalies
  -> schema and reference validation
  -> Redpanda CloudEvents
```

Canonical `personId` values are generator truth and are not copied into source systems. Each source
receives its own local person/customer identifier. The hidden relationship is available only in the
in-memory truth records and optional anomaly manifest, so an entity matcher cannot join sources by a
shared identifier. This behavior is represented by synthetic dataset schema version `2.0.0`.

Random decisions use independent streams derived from `seed`, person sequence, and generation
domain. Consequently, changing VietJet output selection does not perturb an HDBank identity or
account. Supplying the same `seed`, `generatedAt`, scenario, and source mode reproduces the same
dataset. CloudEvent IDs remain unique publication identifiers.

### Source coverage

With `sourceMode: "sampled"`, each person is assigned a profile-conditioned source combination.
The base distribution is:

| Source combination | Base probability |
| --- | ---: |
| HDBank only | 12% |
| Vikki only | 12% |
| VietJet only | 10% |
| HDBank + Vikki | 18% |
| HDBank + VietJet | 15% |
| Vikki + VietJet | 13% |
| All three | 20% |

Customer segments adjust these weights: young digital customers favor Vikki, family borrowers favor
HDBank, and frequent travelers favor VietJet. CRM membership is sampled separately at 40–70%.
At least one of the three operating sources is present. Source absence is normal coverage, not an
anomaly.

Set `sourceMode: "all"` for deterministic demos in which every person exists in every source. The
`sources` request field filters which projections are published. This allows separate calls for
HDBank, Vikki, VietJet, or CRM while retaining stable records for the same seed and timestamp.

### Optional products and activity

Products are conditional rather than universal:

- A bank customer may have no active account.
- A Vikki account may have no wallet or wallet activity.
- An HDBank customer receives a loan only when adoption and affordability tests pass.
- A VietJet customer may have zero bookings; positive booking counts use a bounded,
  zero-inflated Poisson distribution.
- CRM contacts have a bounded Poisson number of campaign interactions.

Economic profiles contain monthly income, expenses, liquid assets, digital affinity, credit need,
and travel propensity. They are internal generation inputs and are not published as source fields.

### Financial and temporal invariants

Completed debits are generated against available funds. Account snapshots are calculated after
wallet payments, loan repayments, and linked ticket payments, so balances never become negative.

For a ledger balance `B`:

```text
B(t) = opening funds + credits through t - debits through t
debit(t) <= available balance immediately before t
```

Ticket affordability is capped by both funds and disposable income:

```text
maximum fare = min(25% of payment-account balance, 35% of monthly disposable income)
```

If no realistic fare is affordable, no paid booking is generated. Vikki wallet and HDBank card
payments carry the same reference, amount, and currency as the VietJet booking.

Loan principal is capped so the first monthly principal-plus-interest payment does not exceed 35%
of monthly income. The repayment schedule sums to the original principal, paid installments point
to real T24 transfers, and outstanding principal equals original principal minus paid principal.

The validator also enforces customer/product references, payment references, currency consistency,
and booking/departure/arrival ordering. Identity differences are treated separately because they can
be intentional anomalies.

### Anomaly model

Anomalies mutate one source projection while preserving valid regional syntax and all financial and
referential invariants. A selected entity receives one field 82% of the time, two fields 15%, and
three fields 3%, capped by `maxAnomaliesPerPerson`.

| Field | Selection weight | Mutation |
| --- | ---: | --- |
| Phone | 25% | Change one subscriber digit, preserving prefix and length |
| Name | 25% | Omit a middle name, remove diacritics, or introduce a small typo |
| Email | 20% | Change the local part while retaining a valid email |
| Date of birth | 12% | Shift by one day, rarely one year |
| National ID | 10% | Change a serial digit; recompute Thai checksum |
| Address | 8% | Produce a plausible stale street number |

Each mutation creates an `AnomalyManifestEntry` containing the truth ID, source record ID, field,
kind, canonical value, observed value, and severity. It is returned only when `includeManifest` is
true and is never included in normal source events.

Scenario defaults are:

| Scenario | Entity anomaly rate |
| --- | ---: |
| `clean` | 0% |
| `realistic` | 12% |
| `matcher-demo` | 35% |
| `stress` | 60% |

`anomalyRate` can override the selected scenario. An anomaly is assigned to one source, so the
observed rate on a request filtered to a single source can be lower than the entity rate.

## API

The Fastify server listens on port `4020` and exposes `POST /api/v1/generate`.

```bash
curl -X POST http://localhost:4020/api/v1/generate \
  -H 'content-type: application/json' \
  -d '{
    "region": "vn",
    "count": 100,
    "seed": 42,
    "generatedAt": "2026-10-13T01:00:00.000Z",
    "scenario": "matcher-demo",
    "sourceMode": "sampled",
    "sources": ["hdbank", "vikkiBank", "vietjetAir", "crm"],
    "maxAnomaliesPerPerson": 2,
    "includeManifest": true
  }'
```

Request fields:

| Field | Default | Meaning |
| --- | --- | --- |
| `region` | `vn` | `vn` or `th` localization engine |
| `count` | `1` | Canonical people, from 1 to 100 |
| `seed` | random | Reproducible random seed |
| `generatedAt` | current time | Reproducible logical generation time |
| `scenario` | `realistic` | `clean`, `realistic`, `matcher-demo`, or `stress` |
| `sourceMode` | `sampled` | Probabilistic coverage or `all` sources |
| `sources` | all | Projections/topics to publish |
| `anomalyRate` | scenario rate | Optional value from 0 to 1 |
| `maxAnomaliesPerPerson` | `3` | Mutation cap from 1 to 3 |
| `includeManifest` | `false` | Include protected evaluation truth in the response |

To land sources at different times, reuse the same deterministic inputs and change only `sources`:

```bash
# First T24 source
curl -X POST http://localhost:4020/api/v1/generate \
  -H 'content-type: application/json' \
  -d '{"region":"vn","count":100,"seed":42,"generatedAt":"2026-10-13T01:00:00.000Z","sources":["hdbank"]}'

# Later CRM poll for the same hidden population
curl -X POST http://localhost:4020/api/v1/generate \
  -H 'content-type: application/json' \
  -d '{"region":"vn","count":100,"seed":42,"generatedAt":"2026-10-13T01:00:00.000Z","sources":["crm"]}'
```

The deployed endpoint is
`https://api.synthetics.mizuumi.test/api/v1/generate`.

## Redpanda topics

The service publishes one CloudEvent for each source record that exists:

- `vn.synthetics.hdbank`, `vn.synthetics.vikki`, `vn.synthetics.vietjetair`, `vn.synthetics.crm`
- `th.synthetics.hdbank`, `th.synthetics.vikki`, `th.synthetics.vietjetair`, `th.synthetics.crm`

Messages use source-local customer/contact IDs as Kafka keys and CloudEvent subjects. Empty source
projections do not produce messages.

Environment overrides are `REDPANDA_BROKERS`, `REDPANDA_CLIENT_ID`, `REDPANDA_TLS`, the optional
SASL variables, and `REDPANDA_TOPIC_<REGION>_<SOURCE>` for all eight topics.

## Commands

```bash
npm install
npm run dev
npm run check
npm test
npm run build
npm start
```

Import schemas and the generator from the package root:

```ts
import {
  SyntheticDatasetSchema,
  generateSyntheticData,
  type SyntheticDataset,
} from "@mizuumi/synthetics"
```

SQLite is not required. Stable substream seeds make source-specific generation reproducible without
persisting canonical people. A database can be added later if runs need mutable lifecycle state or
scheduled replay checkpoints.
