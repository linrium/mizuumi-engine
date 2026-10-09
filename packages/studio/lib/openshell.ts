import { readFile } from "node:fs/promises"
import { homedir } from "node:os"
import { join } from "node:path"
import {
  type ExecOptions,
  type ExecStreamEvent,
  OpenShellClient,
  type SandboxSpec,
} from "@nvidia/openshell-sdk"

const sandboxNamePattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/
const studioAgentSelector = "mizuumi.studio.agent=true"

export type AgentSandbox = {
  name: string
  status: string
}

type GatewayMetadata = {
  gateway_endpoint?: unknown
}

let clientPromise: Promise<OpenShellClient> | undefined

function environmentPath(name: string, fallback: string) {
  const value = process.env[name]?.trim()
  return value || fallback
}

async function connectOpenShell() {
  const configHome =
    process.env.XDG_CONFIG_HOME?.trim() || join(homedir(), ".config")
  const configRoot = join(configHome, "openshell")
  const gatewayName =
    process.env.OPENSHELL_GATEWAY_NAME?.trim() ||
    (await readFile(join(configRoot, "active_gateway"), "utf8")).trim()

  if (!gatewayName) {
    throw new Error("OpenShell has no active gateway.")
  }

  const gatewayRoot = join(configRoot, "gateways", gatewayName)
  const metadata = JSON.parse(
    await readFile(join(gatewayRoot, "metadata.json"), "utf8"),
  ) as GatewayMetadata
  const configuredGateway =
    typeof metadata.gateway_endpoint === "string"
      ? metadata.gateway_endpoint.trim()
      : ""
  const gateway =
    process.env.OPENSHELL_GATEWAY_ENDPOINT?.trim() || configuredGateway

  if (!gateway) {
    throw new Error(`OpenShell gateway "${gatewayName}" has no endpoint.`)
  }

  const mtlsRoot = join(gatewayRoot, "mtls")
  return OpenShellClient.connect({
    gateway,
    caCert: await readFile(
      environmentPath("OPENSHELL_CA_CERT", join(mtlsRoot, "ca.crt")),
    ),
    clientCert: await readFile(
      environmentPath("OPENSHELL_CLIENT_CERT", join(mtlsRoot, "tls.crt")),
    ),
    clientKey: await readFile(
      environmentPath("OPENSHELL_CLIENT_KEY", join(mtlsRoot, "tls.key")),
    ),
  })
}

function getOpenShellClient() {
  clientPromise ??= connectOpenShell()
  return clientPromise
}

export function assertSandboxName(name: unknown): asserts name is string {
  if (
    typeof name !== "string" ||
    name.length > 63 ||
    !sandboxNamePattern.test(name)
  ) {
    throw new Error(
      "Agent name must be lowercase kebab-case and at most 63 characters.",
    )
  }
}

export async function createSandbox(spec: SandboxSpec) {
  const client = await getOpenShellClient()
  return client.sandbox.create(spec)
}

export async function deleteSandbox(name: string) {
  const client = await getOpenShellClient()
  return client.sandbox.delete(name)
}

export async function* execSandboxStream(
  name: string,
  command: string[],
  options?: ExecOptions,
): AsyncGenerator<ExecStreamEvent, void, void> {
  const client = await getOpenShellClient()
  yield* client.sandbox.execStream(name, command, options)
}

export async function listAgentSandboxes() {
  const client = await getOpenShellClient()
  const sandboxes = await client.sandbox.listAll({
    labelSelector: studioAgentSelector,
    pageSize: 100,
  })

  return sandboxes.map(({ name, phase }) => ({ name, status: phase }))
}

export async function assertManagedAgentSandbox(name: string) {
  const agents = await listAgentSandboxes()
  if (!agents.some((agent) => agent.name === name)) {
    throw new Error(`Agent sandbox "${name}" is not managed by Studio.`)
  }
}

export function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : "Unexpected server error."
}
