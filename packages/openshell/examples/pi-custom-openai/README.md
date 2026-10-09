# Pi with a custom OpenAI-compatible endpoint

This example adapts NVIDIA's
[Run Pi with OpenRouter](https://docs.nvidia.com/openshell/latest/tutorials/run-pi-with-openrouter)
tutorial for any endpoint that implements the OpenAI Chat Completions API.

Like the NVIDIA tutorial, it builds Pi from its npm package, includes `fd` and
`ripgrep`, stores Pi's writable state in a runtime-owned directory under
`/tmp`, and gives only the Node.js binary access to the inference endpoint. The
real API key is stored by OpenShell. Pi receives an opaque placeholder, which
OpenShell replaces only for requests to the configured host, port, and path.

## Prerequisites

- OpenShell 0.1.2 or later with an active gateway
- Docker or Podman
- HashiCorp Vault and its CLI
- `jq` for initializing the Vault KV mount
- An OpenAI-compatible Chat Completions endpoint, API key, and model ID

The endpoint URL should include the API prefix expected by the service, usually
`/v1`. For a model server running on the host, bind it to a reachable interface
and use `host.openshell.internal` instead of `localhost`.

## Initialize HashiCorp Vault

`init-vault-kv.sh` creates a dedicated KV v2 secrets-engine mount and writes
the three values from the current environment:

```sh
export VAULT_ADDR=https://vault.example.com
export VAULT_TOKEN=your-vault-token

export CUSTOM_OPENAI_BASE_URL=https://api.example.com/v1
export CUSTOM_OPENAI_API_KEY=your-key
export CUSTOM_OPENAI_MODEL=your-model-id

./init-vault-kv.sh
```

## Run Pi

Unset the seed values and launch Pi. `run.sh` reads the existing Vault secret,
builds `pi-custom-openai:local-1`, creates an endpoint-bound provider, and starts
Pi in a sandbox:

```sh
unset CUSTOM_OPENAI_BASE_URL CUSTOM_OPENAI_API_KEY CUSTOM_OPENAI_MODEL
./run.sh
```

Run `init-vault-kv.sh` again with all three values to write a new KV v2 secret
version. `run.sh` is read-only with respect to Vault.

The default mount is `pi-custom-openai/` and the secret path is `config`.
Override either location when needed:

```sh
export VAULT_KV_MOUNT=agents
export VAULT_KV_SECRET_PATH=pi/custom-openai
```

The Vault token must be able to inspect and enable secret-engine mounts and to
create, update, and read the selected KV v2 path. Existing mounts are reused
only when they are KV v2; the script will not replace a different engine. In
production, authenticate the Vault CLI with your normal login method instead
of exporting a long-lived root token.

The initialization script creates only the dedicated KV v2 secrets-engine
mount. It does not initialize or unseal the Vault server itself.

The API key is read from Vault by `run.sh` and submitted to OpenShell's provider
store. Inside the sandbox, Pi still receives only OpenShell's opaque,
endpoint-bound credential placeholder. The plaintext key is not passed with
`sandbox create --env`, written into the image, or written into Pi's
configuration. The endpoint and model are non-secret values and are passed
explicitly to the sandbox environment.

For Podman, use an image name visible to the gateway:

```sh
CONTAINER_ENGINE=podman \
CUSTOM_OPENAI_IMAGE=localhost/pi-custom-openai:local-1 \
./run.sh
```

If the gateway runs on another machine, build and push the image separately,
then skip the local build and use the registry image:

```sh
CUSTOM_OPENAI_SKIP_BUILD=true \
CUSTOM_OPENAI_IMAGE=registry.example.com/team/pi-custom-openai:latest \
./run.sh
```

Optional names can prevent collisions with an existing setup:

```sh
export CUSTOM_OPENAI_PROFILE=pi-team-api
export CUSTOM_OPENAI_PROVIDER=pi-team-api
export CUSTOM_OPENAI_SANDBOX=pi-team-api
```

OpenShell profiles are deliberately not overwritten. If a selected profile,
provider, or sandbox already exists, the script exits instead of potentially
reusing credentials or endpoint policy from another setup.

## Inspect and clean up

In another terminal, inspect the sandbox while Pi is running:

```sh
openshell sandbox provider list pi-custom-openai
openshell policy get pi-custom-openai --full
openshell logs pi-custom-openai --tail
```

Quit Pi with `/quit`, then remove the OpenShell resources when they are no
longer needed:

```sh
./cleanup.sh
```

The cleanup script uses the same `CUSTOM_OPENAI_PROFILE`,
`CUSTOM_OPENAI_PROVIDER`, and `CUSTOM_OPENAI_SANDBOX` overrides as `run.sh`.
It is idempotent and waits for asynchronous sandbox deletion before removing
the provider and profile. It intentionally preserves the Vault KV mount and
the locally built container image.

## Compatibility

The generated Pi configuration uses `openai-completions`. If the service only
implements the OpenAI Responses API, change `api` in `pi-custom-openai` to
`openai-responses` before building the image.
