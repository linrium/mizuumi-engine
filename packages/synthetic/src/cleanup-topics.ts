#!/usr/bin/env node

import { pathToFileURL } from "node:url"
import { Kafka } from "kafkajs"
import { Topics } from "./minh-anh-events.js"
import { connectionOptions, kafkaConfig, type RedpandaConnectionOptions } from "./redpanda-config.js"

interface CleanupOptions extends RedpandaConnectionOptions {
  confirmed: boolean
  dryRun: boolean
}

const HELP = `Delete the four Mizuumi synthetic demo topics from Redpanda

Usage:
  npm run cleanup:topics -- [options]

Options:
  --brokers <list>   Comma-separated brokers (default: REDPANDA_BROKERS or the Mizuumi broker)
  --client-id <id>   Kafka client ID (default: mizuumi-topic-cleanup)
  --tls              Enable TLS (also REDPANDA_TLS=true)
  --no-tls           Disable TLS
  --dry-run          Print the four topics without connecting or deleting
  --yes              Confirm permanent topic deletion
  --help             Show this help
`

function argumentValue(args: string[], index: number, flag: string): string {
  const value = args[index + 1]
  if (!value || value.startsWith("--")) throw new Error(`${flag} requires a value`)
  return value
}

export function parseCleanupArgs(args: string[]): CleanupOptions | "help" {
  const options: CleanupOptions = {
    ...connectionOptions("mizuumi-topic-cleanup"),
    confirmed: false,
    dryRun: false,
  }
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index]
    if (argument === "--help" || argument === "-h") return "help"
    if (argument === "--yes") options.confirmed = true
    else if (argument === "--dry-run") options.dryRun = true
    else if (argument === "--tls") options.tls = true
    else if (argument === "--no-tls") options.tls = false
    else if (argument === "--brokers") options.brokers = argumentValue(args, index++, argument).split(",").map((broker) => broker.trim()).filter(Boolean)
    else if (argument === "--client-id") options.clientId = argumentValue(args, index++, argument)
    else throw new Error(`Unknown option: ${argument}`)
  }
  if (options.brokers.length === 0) throw new Error("At least one Redpanda broker is required")
  return options
}

export const demoTopics = (): string[] => Object.values(Topics)

export async function cleanupTopics(options: CleanupOptions): Promise<string[]> {
  const targets = demoTopics()
  if (options.dryRun) return targets
  if (!options.confirmed) throw new Error("Topic deletion is permanent. Re-run with --yes, or inspect with --dry-run.")

  const admin = new Kafka(kafkaConfig(options)).admin()
  await admin.connect()
  try {
    const existing = new Set(await admin.listTopics())
    const deletions = targets.filter((topic) => existing.has(topic))
    if (deletions.length > 0) await admin.deleteTopics({ topics: deletions, timeout: 30_000 })
    return deletions
  } finally {
    await admin.disconnect()
  }
}

async function main(): Promise<void> {
  const options = parseCleanupArgs(process.argv.slice(2))
  if (options === "help") {
    process.stdout.write(HELP)
    return
  }
  const topics = await cleanupTopics(options)
  const action = options.dryRun ? "Would delete" : "Deleted"
  process.stdout.write(`${action} ${topics.length} topic${topics.length === 1 ? "" : "s"}:\n${topics.map((topic) => `- ${topic}`).join("\n")}\n`)
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error: unknown) => {
    const message = error instanceof Error ? error.message : String(error)
    process.stderr.write(`Error: ${message}\n`)
    process.exitCode = 1
  })
}
