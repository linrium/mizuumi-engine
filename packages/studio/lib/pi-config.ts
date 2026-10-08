import { execFile } from "node:child_process"
import { promisify } from "node:util"

const execFileAsync = promisify(execFile)

export type PiConfig = {
  baseUrl: string
  model: string
}

function vaultError(error: unknown) {
  if (
    typeof error === "object" &&
    error !== null &&
    "stderr" in error &&
    typeof error.stderr === "string" &&
    error.stderr.trim()
  ) {
    return error.stderr.trim()
  }
  return error instanceof Error ? error.message : "Unknown Vault error."
}

function validateVaultLocation(mount: string, secretPath: string) {
  if (!/^[A-Za-z0-9_-]+$/.test(mount)) {
    throw new Error(
      "VAULT_KV_MOUNT must contain only letters, numbers, underscores, or hyphens.",
    )
  }
  if (
    !/^[A-Za-z0-9._/-]+$/.test(secretPath) ||
    secretPath.startsWith("/") ||
    `/${secretPath}/`.includes("/../")
  ) {
    throw new Error(
      "VAULT_KV_SECRET_PATH must be a relative Vault path without '..' segments.",
    )
  }
}

function parseVaultPiConfig(output: string): PiConfig {
  const response: unknown = JSON.parse(output)
  const data =
    typeof response === "object" && response !== null && "data" in response
      ? response.data
      : undefined
  const secret =
    typeof data === "object" && data !== null && "data" in data
      ? data.data
      : undefined

  const baseUrl =
    typeof secret === "object" &&
    secret !== null &&
    "base_url" in secret &&
    typeof secret.base_url === "string"
      ? secret.base_url
      : ""
  const model =
    typeof secret === "object" &&
    secret !== null &&
    "model" in secret &&
    typeof secret.model === "string"
      ? secret.model
      : ""

  if (!baseUrl || !model) {
    throw new Error(
      "The Pi Vault secret must contain non-empty base_url and model fields.",
    )
  }

  return { baseUrl, model }
}

export async function getPiConfig(): Promise<PiConfig> {
  const mount = process.env.VAULT_KV_MOUNT ?? "pi-custom-openai"
  const secretPath = process.env.VAULT_KV_SECRET_PATH ?? "config"
  validateVaultLocation(mount, secretPath)

  try {
    const { stdout } = await execFileAsync(
      "vault",
      ["kv", "get", `-mount=${mount}`, "-format=json", secretPath],
      { maxBuffer: 1024 * 1024, timeout: 15_000 },
    )
    return parseVaultPiConfig(stdout)
  } catch (error) {
    throw new Error(
      `Could not read Vault KV ${mount}/${secretPath}: ${vaultError(error)}`,
    )
  }
}
