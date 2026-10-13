#!/usr/bin/env node

import { randomUUID } from "node:crypto"
import path from "node:path"
import { pathToFileURL } from "node:url"
import { Kafka, type Message } from "kafkajs"
import { loadMinhAnhEvents, type SourceEvent } from "./minh-anh-events.js"
import { connectionOptions, kafkaConfig, type RedpandaConnectionOptions } from "./redpanda-config.js"

interface Options extends RedpandaConnectionOptions {
  inputDirectory: string
  dryRun: boolean
}

const HELP = `Publish Trần Minh Anh's synthetic source records to Redpanda

Usage:
  npm run publish:minh-anh -- [options]

Options:
  --input <directory>   Generated dataset (default: ./data/minh-anh)
  --brokers <list>      Comma-separated brokers (default: REDPANDA_BROKERS or the Mizuumi broker)
  --client-id <id>      Kafka client ID (default: mizuumi-minh-anh-publisher)
  --tls                 Enable TLS (also REDPANDA_TLS=true)
  --no-tls              Disable TLS
  --dry-run             Print topic routing without connecting
  --help                Show this help
`

function argumentValue(args: string[], index: number, flag: string): string {
  const value = args[index + 1]
  if (!value || value.startsWith("--")) throw new Error(`${flag} requires a value`)
  return value
}

export function parsePublishArgs(args: string[]): Options | "help" {
  const options: Options = {
    ...connectionOptions("mizuumi-minh-anh-publisher"),
    inputDirectory: path.resolve("data/minh-anh"),
    dryRun: false,
  }
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index]
    if (argument === "--help" || argument === "-h") return "help"
    if (argument === "--tls") options.tls = true
    else if (argument === "--no-tls") options.tls = false
    else if (argument === "--dry-run") options.dryRun = true
    else if (argument === "--input") options.inputDirectory = path.resolve(argumentValue(args, index++, argument))
    else if (argument === "--brokers") options.brokers = argumentValue(args, index++, argument).split(",").map((broker) => broker.trim()).filter(Boolean)
    else if (argument === "--client-id") options.clientId = argumentValue(args, index++, argument)
    else throw new Error(`Unknown option: ${argument}`)
  }
  if (options.brokers.length === 0) throw new Error("At least one Redpanda broker is required")
  return options
}

function cloudEvent(event: SourceEvent): { value: string; headers: Record<string, string> } {
  const id = randomUUID()
  const time = new Date().toISOString()
  const value = JSON.stringify({
    specversion: "1.0",
    id,
    source: event.source,
    type: event.type,
    time,
    datacontenttype: "application/json",
    subject: event.subject,
    data: event.data,
  })
  return {
    value,
    headers: {
      "ce-id": id,
      "ce-type": event.type,
      "ce-source": event.source,
      "ce-time": time,
      "ce-specversion": "1.0",
      "content-type": "application/cloudevents+json",
    },
  }
}

export async function publishEvents(events: SourceEvent[], options: Options): Promise<void> {
  if (options.dryRun) {
    for (const event of events) process.stdout.write(`${event.topic}\t${event.key}\t${event.type}\n`)
    return
  }
  const producer = new Kafka(kafkaConfig(options)).producer({ allowAutoTopicCreation: true })
  await producer.connect()
  try {
    const topicMessages = new Map<string, Message[]>()
    for (const event of events) {
      const envelope = cloudEvent(event)
      const messages = topicMessages.get(event.topic) ?? []
      messages.push({ key: event.key, value: envelope.value, headers: envelope.headers })
      topicMessages.set(event.topic, messages)
    }
    await producer.sendBatch({
      topicMessages: [...topicMessages].map(([topic, messages]) => ({ topic, messages })),
    })
  } finally {
    await producer.disconnect()
  }
}

async function main(): Promise<void> {
  const options = parsePublishArgs(process.argv.slice(2))
  if (options === "help") {
    process.stdout.write(HELP)
    return
  }
  const events = await loadMinhAnhEvents(options.inputDirectory)
  await publishEvents(events, options)
  const topicCount = new Set(events.map((event) => event.topic)).size
  process.stdout.write(`${options.dryRun ? "Would publish" : "Published"} ${events.length} records to ${topicCount} Redpanda topics.\n`)
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error: unknown) => {
    const message = error instanceof Error ? error.message : String(error)
    process.stderr.write(`Error: ${message}\n`)
    process.exitCode = 1
  })
}
