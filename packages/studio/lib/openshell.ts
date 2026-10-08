import {
  type ChildProcessWithoutNullStreams,
  execFile,
  spawn,
} from "node:child_process"
import { promisify } from "node:util"

const execFileAsync = promisify(execFile)

const sandboxNamePattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/
const studioAgentSelector = "mizuumi.studio.agent=true"

export type AgentSandbox = {
  name: string
  status: string
}

function openshellArgs(args: string[]) {
  return ["--color", "never", ...args]
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

export async function runOpenShell(args: string[]) {
  try {
    const { stdout } = await execFileAsync("openshell", openshellArgs(args), {
      maxBuffer: 10 * 1024 * 1024,
      timeout: 120_000,
    })
    return stdout
  } catch (error) {
    if (
      typeof error === "object" &&
      error !== null &&
      "stderr" in error &&
      typeof error.stderr === "string" &&
      error.stderr.trim()
    ) {
      throw new Error(error.stderr.trim())
    }
    throw error
  }
}

export function spawnOpenShell(args: string[]): ChildProcessWithoutNullStreams {
  return spawn("openshell", openshellArgs(args), {
    stdio: ["pipe", "pipe", "pipe"],
  })
}

function objectValue(value: unknown, keys: string[]): unknown {
  if (typeof value !== "object" || value === null) return undefined

  for (const key of keys) {
    if (key in value) return (value as Record<string, unknown>)[key]
  }
}

export function parseAgentSandboxes(output: string): AgentSandbox[] {
  const parsed: unknown = JSON.parse(output)
  const sandboxes = Array.isArray(parsed)
    ? parsed
    : objectValue(parsed, ["sandboxes", "items"])

  if (!Array.isArray(sandboxes)) {
    throw new Error("OpenShell returned an unexpected sandbox list.")
  }

  return sandboxes.flatMap((sandbox) => {
    const name = objectValue(sandbox, ["name"])
    if (typeof name !== "string") return []

    const statusValue = objectValue(sandbox, ["status", "phase", "state"])
    const nestedPhase = objectValue(statusValue, ["phase", "state"])
    const status =
      typeof statusValue === "string"
        ? statusValue
        : typeof nestedPhase === "string"
          ? nestedPhase
          : "unknown"

    return [{ name, status }]
  })
}

export async function listAgentSandboxes() {
  const output = await runOpenShell([
    "sandbox",
    "list",
    "--selector",
    studioAgentSelector,
    "--page-size",
    "100",
    "--output",
    "json",
  ])
  return parseAgentSandboxes(output)
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
