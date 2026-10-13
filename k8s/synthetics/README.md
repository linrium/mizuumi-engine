# Synthetics

This chart deploys the Fastify synthetic-data generator in the `synthetics` namespace. The service
publishes generated HDBank, Vikki Bank, VietJet Air, and CRM CloudEvents to the in-cluster
Redpanda broker.

```bash
./scripts/synthetics/bootstrap.sh
./scripts/synthetics/setup.sh
sudo ./scripts/configure_workstation_dns.sh
curl -X POST https://api.synthetics.mizuumi.test/api/v1/generate \
  -H 'content-type: application/json' \
  -d '{"region":"vn","count":10,"seed":42,"scenario":"realistic"}'
```

Use `"region":"th"` to generate Thai identities and THB amounts. `bootstrap.sh` creates the
namespace and all eight `vn.synthetics.*` and `th.synthetics.*` Redpanda topics. `setup.sh` builds and
loads the local image, installs the chart, and waits for the deployment and service endpoint.
It also updates the shared gateway and TLS certificate when the synthetics route is not installed.

Delete only the eight regional synthetics topics with:

```bash
./scripts/synthetics/cleanup.sh --dry-run
./scripts/synthetics/cleanup.sh
```

The cleanup script does not uninstall the Helm release or delete the `synthetics` namespace.

Generate one persistent user once, or continuously, with:

```bash
./scripts/synthetics/generate-one-user.sh
./scripts/synthetics/generate-one-user.sh --watch
```

The script publishes one source projection per iteration and rotates through all four sources by
default. It keeps `initialPopulation: 1`, `newPeoplePerDay: 0`, and `newPeople: 0`, so the stream
never grows beyond the original person.

For SASL, create a Secret containing `REDPANDA_SASL_USERNAME`, `REDPANDA_SASL_PASSWORD`, and
optionally `REDPANDA_SASL_MECHANISM`, then set `redpanda.saslSecretName`.
