import { randomUUID } from "node:crypto"
import { Kafka, logLevel, type Producer, type SASLOptions } from "kafkajs"
import type { GeneratedPersonRecords, GeneratedSyntheticData } from "./generator.js"
import type { RegionCode } from "./generators/index.js"

export type PublishSummary = {
  region: RegionCode
  topics: {
    hdbank: string
    vikkiBank: string
    vietjetAir: string
  }
  messages: {
    hdbank: number
    vikkiBank: number
    vietjetAir: number
    total: number
  }
}

export interface SyntheticPublisher {
  publish(generated: GeneratedSyntheticData): Promise<PublishSummary>
  close(): Promise<void>
}

type OrganizationName = "hdbank" | "vikkiBank" | "vietjetAir"

type SyntheticCloudEvent<T> = {
  specversion: "1.0"
  id: string
  source: string
  type: string
  time: string
  datacontenttype: "application/json"
  subject: string
  data: T
}

export type RedpandaPublisherOptions = {
  brokers?: string[]
  clientId?: string
  ssl?: boolean
  sasl?: SASLOptions
  topics?: Partial<Record<RegionCode, Partial<Record<OrganizationName, string>>>>
}

function event<T>(input: {
  organization: OrganizationName
  region: RegionCode
  personId: string
  generatedAt: string
  data: T
}): SyntheticCloudEvent<T> {
  const organizationSlug = input.organization === "vikkiBank" ? "vikki" : input.organization
  return {
    specversion: "1.0",
    id: randomUUID(),
    source: `/mizuumi/synthetics/${input.region}/${organizationSlug}`,
    type: `${input.region}.mizuumi.synthetic.${organizationSlug}.customer.generated`,
    time: input.generatedAt,
    datacontenttype: "application/json",
    subject: input.personId,
    data: input.data,
  }
}

function kafkaMessage(
  organization: OrganizationName,
  region: RegionCode,
  record: GeneratedPersonRecords,
  generatedAt: string,
) {
  const payload = event({
    organization,
    region,
    personId: record.person.personId,
    generatedAt,
    data: record[organization],
  })
  return {
    key: record.person.personId,
    value: JSON.stringify(payload),
    headers: {
      "ce-id": payload.id,
      "ce-source": payload.source,
      "ce-specversion": payload.specversion,
      "ce-subject": payload.subject,
      "ce-time": payload.time,
      "ce-type": payload.type,
      "content-type": payload.datacontenttype,
    },
  }
}

export class RedpandaPublisher implements SyntheticPublisher {
  private readonly producer: Producer
  private readonly topics: Record<RegionCode, Record<OrganizationName, string>>
  private connection: Promise<void> | undefined

  constructor(options: RedpandaPublisherOptions = {}) {
    const brokers =
      options.brokers ??
      (process.env.REDPANDA_BROKERS ?? "redpanda-0.api.redpanda.mizuumi.test:30092")
        .split(",")
        .map((broker) => broker.trim())
        .filter(Boolean)
    const ssl =
      options.ssl ?? (process.env.REDPANDA_TLS ?? "false").toLowerCase() !== "false"
    const username = process.env.REDPANDA_SASL_USERNAME
    const password = process.env.REDPANDA_SASL_PASSWORD
    const mechanism = (process.env.REDPANDA_SASL_MECHANISM ?? "scram-sha-256") as
      | "plain"
      | "scram-sha-256"
      | "scram-sha-512"
    const sasl: SASLOptions | undefined =
      options.sasl ??
      (username && password
        ? ({ mechanism, username, password } as SASLOptions)
        : undefined)
    const kafka = new Kafka({
      clientId: options.clientId ?? process.env.REDPANDA_CLIENT_ID ?? "mizuumi-synthetics",
      brokers,
      ssl,
      sasl,
      logLevel: logLevel.NOTHING,
      retry: { retries: 5, initialRetryTime: 300 },
    })

    this.producer = kafka.producer({ allowAutoTopicCreation: true })
    this.topics = {
      vn: {
        hdbank:
          options.topics?.vn?.hdbank ??
          process.env.REDPANDA_TOPIC_VN_HDBANK ??
          "vn.synthetics.hdbank",
        vikkiBank:
          options.topics?.vn?.vikkiBank ??
          process.env.REDPANDA_TOPIC_VN_VIKKI ??
          "vn.synthetics.vikki",
        vietjetAir:
          options.topics?.vn?.vietjetAir ??
          process.env.REDPANDA_TOPIC_VN_VIETJETAIR ??
          "vn.synthetics.vietjetair",
      },
      th: {
        hdbank:
          options.topics?.th?.hdbank ??
          process.env.REDPANDA_TOPIC_TH_HDBANK ??
          "th.synthetics.hdbank",
        vikkiBank:
          options.topics?.th?.vikkiBank ??
          process.env.REDPANDA_TOPIC_TH_VIKKI ??
          "th.synthetics.vikki",
        vietjetAir:
          options.topics?.th?.vietjetAir ??
          process.env.REDPANDA_TOPIC_TH_VIETJETAIR ??
          "th.synthetics.vietjetair",
      },
    }
  }

  private async connect() {
    this.connection ??= this.producer.connect()
    try {
      await this.connection
    } catch (error) {
      this.connection = undefined
      throw error
    }
  }

  async publish(generated: GeneratedSyntheticData): Promise<PublishSummary> {
    await this.connect()
    const generatedAt = generated.dataset.generatedAt
    const region = generated.dataset.region
    const topics = this.topics[region]
    const hdbankMessages = generated.records.map((record) =>
      kafkaMessage("hdbank", region, record, generatedAt),
    )
    const vikkiMessages = generated.records.map((record) =>
      kafkaMessage("vikkiBank", region, record, generatedAt),
    )
    const vietjetMessages = generated.records.map((record) =>
      kafkaMessage("vietjetAir", region, record, generatedAt),
    )

    await this.producer.sendBatch({
      topicMessages: [
        { topic: topics.hdbank, messages: hdbankMessages },
        { topic: topics.vikkiBank, messages: vikkiMessages },
        { topic: topics.vietjetAir, messages: vietjetMessages },
      ],
    })

    return {
      region,
      topics,
      messages: {
        hdbank: hdbankMessages.length,
        vikkiBank: vikkiMessages.length,
        vietjetAir: vietjetMessages.length,
        total: hdbankMessages.length + vikkiMessages.length + vietjetMessages.length,
      },
    }
  }

  async close() {
    if (!this.connection) return
    try {
      await this.connection
      await this.producer.disconnect()
    } finally {
      this.connection = undefined
    }
  }
}
