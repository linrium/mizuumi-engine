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
