# OpenShell examples

`examples/minimal-agent` is a dependency-free sandbox smoke test. It accepts a
small goal, records basic runtime facts in `/sandbox/minimal-agent-result.txt`,
and prints the result. It deliberately needs no model provider, credentials, or
network access, so it also confirms that the restrictive default OpenShell
policy is sufficient for local computation and workspace writes.

`examples/pi-custom-openai` runs the Pi coding agent against a custom
OpenAI-compatible API. It builds a Pi image, creates an endpoint-bound OpenShell
provider for the API key, and starts Pi with the requested model. See the
example README for the required environment variables and setup commands.

Run it through the Kubernetes gateway with:

```sh
./scripts/openshell/bootstrap.sh
```

To run it again in an existing retained sandbox:

```sh
openshell sandbox upload minimal-agent packages/openshell/examples/minimal-agent
openshell sandbox exec -n minimal-agent \
  --workdir /sandbox/minimal-agent \
  -- sh ./agent.sh "verify this sandbox"
```
