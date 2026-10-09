import { randomUUID } from "node:crypto";
import { Kafka, logLevel, type Producer } from "kafkajs";

const BROKERS = (process.env.REDPANDA_BROKERS ?? "redpanda-0.api.redpanda.mizuumi.test:30092")
  .split(",")
  .map((s) => s.trim())
  .filter(Boolean);
const CLIENT_ID = process.env.REDPANDA_CLIENT_ID ?? "temenos-t24-simulator";
const ENABLED = (process.env.REDPANDA_ENABLED ?? "true").toLowerCase() !== "false";
const USE_TLS = (process.env.REDPANDA_TLS ?? "false").toLowerCase() !== "false";
const SASL_USER = process.env.REDPANDA_SASL_USERNAME;
const SASL_PASS = process.env.REDPANDA_SASL_PASSWORD;
const SASL_MECHANISM = (process.env.REDPANDA_SASL_MECHANISM ?? "scram-sha-256") as
  | "plain"
  | "scram-sha-256"
  | "scram-sha-512";
const COOLDOWN_MS = Number(process.env.REDPANDA_COOLDOWN_MS ?? 60_000);
const SOURCE = "/temenos/T24";

export const Topics = {
  Customer: process.env.REDPANDA_TOPIC_CUSTOMER ?? "t24.customer.events",
  Account: process.env.REDPANDA_TOPIC_ACCOUNT ?? "t24.account.events",
  Payment: process.env.REDPANDA_TOPIC_PAYMENT ?? "t24.payment.events",
  Eligibility: process.env.REDPANDA_TOPIC_ELIGIBILITY ?? "t24.eligibility.events",
} as const;

export const EventTypes = {
  CustomerRead: "com.temenos.t24.customer.read",
  CustomerDataRightsAccessed: "com.temenos.t24.customer.datarights.accessed",
  CustomerAccountsListed: "com.temenos.t24.customer.accounts.listed",
  CustomerProductsListed: "com.temenos.t24.customer.products.listed",
  AccountRead: "com.temenos.t24.account.read",
  PaymentInitiated: "com.temenos.t24.payment.initiated",
  PaymentCompleted: "com.temenos.t24.payment.completed",
  PaymentFailed: "com.temenos.t24.payment.failed",
  PaymentRead: "com.temenos.t24.payment.read",
  EligibilityEvaluated: "com.temenos.t24.eligibility.evaluated",
} as const;

export type EventType = (typeof EventTypes)[keyof typeof EventTypes];

export type CloudEvent<T> = {
  specversion: "1.0";
  id: string;
  source: string;
  type: EventType;
  time: string;
  datacontenttype: "application/json";
  subject?: string;
  data: T;
};

let kafka: Kafka | null = null;
let producer: Producer | null = null;
let connecting: Promise<void> | null = null;
let cooldownUntil = 0;

function getKafka() {
  if (!kafka) {
    kafka = new Kafka({
      clientId: CLIENT_ID,
      brokers: BROKERS,
      logLevel: logLevel.NOTHING,
      ssl: USE_TLS,
      sasl:
        SASL_USER && SASL_PASS
          ? { mechanism: SASL_MECHANISM, username: SASL_USER, password: SASL_PASS }
          : undefined,
      retry: { retries: 3, initialRetryTime: 200 },
    });
  }
  return kafka;
}

async function ensureConnected(): Promise<Producer | null> {
  if (!ENABLED) return null;
  if (producer) return producer;
  if (Date.now() < cooldownUntil) return null;
  if (!connecting) {
    const p = getKafka().producer({ allowAutoTopicCreation: true });
    connecting = p
      .connect()
      .then(() => {
        producer = p;
        console.log(`[redpanda] connected brokers=${BROKERS.join(",")} tls=${USE_TLS}`);
      })
      .catch((err) => {
        connecting = null;
        cooldownUntil = Date.now() + COOLDOWN_MS;
        console.warn(
          `[redpanda] connect failed: ${err instanceof Error ? err.message : err} (cooling down ${COOLDOWN_MS}ms)`,
        );
        throw err;
      });
  }
  try {
    await connecting;
    return producer;
  } catch {
    return null;
  }
}

export function buildEvent<T>(
  type: EventType,
  data: T,
  opts: { subject?: string; source?: string } = {},
): CloudEvent<T> {
  return {
    specversion: "1.0",
    id: randomUUID(),
    source: opts.source ?? SOURCE,
    type,
    time: new Date().toISOString(),
    datacontenttype: "application/json",
    subject: opts.subject,
    data,
  };
}

export async function publish<T>(
  topic: string,
  key: string,
  type: EventType,
  data: T,
  opts: { subject?: string } = {},
) {
  if (!ENABLED) return;
  const p = await ensureConnected();
  if (!p) return;
  const event = buildEvent(type, data, { subject: opts.subject ?? key });
  try {
    await p.send({
      topic,
      messages: [
        {
          key,
          value: JSON.stringify(event),
          headers: {
            "ce-id": event.id,
            "ce-type": event.type,
            "ce-source": event.source,
            "ce-time": event.time,
            "ce-specversion": event.specversion,
            "content-type": "application/cloudevents+json",
          },
        },
      ],
    });
  } catch (err) {
    console.warn(
      `[redpanda] publish failed topic=${topic} type=${type}: ${
        err instanceof Error ? err.message : err
      }`,
    );
  }
}

export function emit<T>(
  topic: string,
  key: string,
  type: EventType,
  data: T,
  opts: { subject?: string } = {},
) {
  void publish(topic, key, type, data, opts);
}

export async function shutdownProducer() {
  if (producer) {
    try {
      await producer.disconnect();
    } catch {
      // ignore
    }
    producer = null;
    connecting = null;
  }
}
