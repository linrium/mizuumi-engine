import { logLevel, type KafkaConfig } from "kafkajs"

export const DEFAULT_REDPANDA_BROKERS = ["redpanda-0.api.redpanda.mizuumi.test:30092"]

export interface RedpandaConnectionOptions {
  brokers: string[]
  clientId: string
  tls: boolean
}

export function connectionOptions(clientId: string): RedpandaConnectionOptions {
  return {
    brokers: (process.env.REDPANDA_BROKERS ?? DEFAULT_REDPANDA_BROKERS.join(","))
      .split(",")
      .map((broker) => broker.trim())
      .filter(Boolean),
    clientId: process.env.REDPANDA_CLIENT_ID ?? clientId,
    tls: (process.env.REDPANDA_TLS ?? "false").toLowerCase() === "true",
  }
}

export function kafkaConfig(options: RedpandaConnectionOptions): KafkaConfig {
  const username = process.env.REDPANDA_SASL_USERNAME
  const password = process.env.REDPANDA_SASL_PASSWORD
  const mechanism = process.env.REDPANDA_SASL_MECHANISM ?? "scram-sha-256"
  let sasl: KafkaConfig["sasl"]
  if (username && password) {
    if (mechanism === "plain") sasl = { mechanism: "plain", username, password }
    else if (mechanism === "scram-sha-256") sasl = { mechanism: "scram-sha-256", username, password }
    else if (mechanism === "scram-sha-512") sasl = { mechanism: "scram-sha-512", username, password }
    else throw new Error(`Unsupported REDPANDA_SASL_MECHANISM: ${mechanism}`)
  }
  return {
    clientId: options.clientId,
    brokers: options.brokers,
    ssl: options.tls,
    sasl,
    logLevel: logLevel.NOTHING,
    retry: { retries: 5, initialRetryTime: 300 },
  }
}
