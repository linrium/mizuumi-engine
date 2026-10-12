# Synthetics

This chart deploys the Fastify synthetic-data generator in the `synthetics` namespace. The service
publishes generated HDBank, Vikki Bank, and VietJet Air CloudEvents to the in-cluster Redpanda
broker.

```bash
./scripts/synthetics/bootstrap.sh
./scripts/synthetics/setup.sh
sudo ./scripts/configure_workstation_dns.sh
curl -X POST https://api.synthetics.mizuumi.test/api/v1/generate \
  -H 'content-type: application/json' \
  -d '{"region":"vn","count":10,"seed":42}'
```

Use `"region":"th"` to generate Thai identities and THB amounts. `bootstrap.sh` creates the
namespace and all six `vn.synthetics.*` and `th.synthetics.*` Redpanda topics. `setup.sh` builds and
loads the local image, installs the chart, and waits for the deployment and service endpoint.
It also updates the shared gateway and TLS certificate when the synthetics route is not installed.

Delete only the six regional synthetics topics with:

```bash
./scripts/synthetics/cleanup.sh --dry-run
./scripts/synthetics/cleanup.sh
```

The cleanup script does not uninstall the Helm release or delete the `synthetics` namespace.

For SASL, create a Secret containing `REDPANDA_SASL_USERNAME`, `REDPANDA_SASL_PASSWORD`, and
optionally `REDPANDA_SASL_MECHANISM`, then set `redpanda.saslSecretName`.
