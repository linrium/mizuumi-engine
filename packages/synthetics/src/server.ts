import { randomInt } from "node:crypto"
import { pathToFileURL } from "node:url"
import Fastify from "fastify"
import { z } from "zod"
import { generateSyntheticData } from "./generator.js"
import { sourceNames } from "./generator.js"
import {
  advanceIncrementalStream,
  IncrementalEventLimitError,
  IncrementalRequestError,
  initializeIncrementalStream,
} from "./incremental.js"
import {
  IncrementalStore,
  StreamAlreadyExistsError,
  StreamConflictError,
} from "./incremental-store.js"
import {
  RedpandaPublisher,
  type PublishSummary,
  type SyntheticPublisher,
} from "./redpanda.js"

const GenerateRequestSchema = z.object({
  mode: z.literal("snapshot").optional(),
  region: z.enum(["vn", "th"]).default("vn"),
  count: z.number().int().min(1).max(100).default(1),
  seed: z.number().int().min(0).max(2_147_483_647).optional(),
  generatedAt: z.string().datetime().optional(),
  scenario: z.enum(["clean", "realistic", "matcher-demo", "stress"]).default("realistic"),
  sourceMode: z.enum(["sampled", "all"]).default("sampled"),
  sources: z.array(z.enum(sourceNames)).min(1).max(sourceNames.length).optional(),
  anomalyRate: z.number().min(0).max(1).optional(),
  maxAnomaliesPerPerson: z.number().int().min(1).max(3).default(3),
  includeManifest: z.boolean().default(false),
})

const IncrementalBaseSchema = z.object({
  mode: z.literal("incremental"),
  streamId: z.string().min(1).max(100).regex(/^[a-zA-Z0-9._-]+$/),
  asOf: z.string().datetime(),
  sources: z.array(z.enum(sourceNames)).min(1).max(sourceNames.length).optional(),
  maxEvents: z.number().int().min(1).max(50_000).default(10_000),
})

const InitializeIncrementalSchema = IncrementalBaseSchema.extend({
  operation: z.literal("initialize"),
  region: z.enum(["vn", "th"]).default("vn"),
  initialPopulation: z.number().int().min(1).max(1_000).default(100),
  seed: z.number().int().min(0).max(2_147_483_647).optional(),
  scenario: z.enum(["clean", "realistic", "matcher-demo", "stress"]).default("realistic"),
  sourceMode: z.enum(["sampled", "all"]).default("sampled"),
  anomalyRate: z.number().min(0).max(1).optional(),
  maxAnomaliesPerPerson: z.number().int().min(1).max(3).default(3),
  newPeoplePerDay: z.number().min(0).max(100).default(1),
})

const AdvanceIncrementalSchema = IncrementalBaseSchema.extend({
  operation: z.literal("advance"),
  newPeople: z.number().int().min(0).max(100).optional(),
  emitCurrentRecord: z.boolean().default(false),
})

const IncrementalRequestSchema = z.discriminatedUnion("operation", [
  InitializeIncrementalSchema,
  AdvanceIncrementalSchema,
])

export type GenerateResponse = {
  region: "vn" | "th"
  generatedAt: string
  seed: number
  peopleGenerated: number
  coverage: Record<(typeof sourceNames)[number], number>
  anomalies: {
    total: number
    manifest?: ReturnType<typeof generateSyntheticData>["manifest"]
  }
  published: PublishSummary
}

export function buildServer(
  options: { publisher?: SyntheticPublisher; logger?: boolean; store?: IncrementalStore; databasePath?: string } = {},
) {
  const server = Fastify({ logger: options.logger ?? true })
  const publisher = options.publisher ?? new RedpandaPublisher()
  let store = options.store
  const getStore = () => {
    store ??= new IncrementalStore(options.databasePath ?? process.env.SYNTHETICS_DB_PATH ?? "/data/synthetics.db")
    return store
  }

  server.post("/api/v1/generate", async (request, reply) => {
    const body = request.body as Record<string, unknown> | undefined
    if (body?.mode === "incremental") {
      const parsed = IncrementalRequestSchema.safeParse(body)
      if (!parsed.success) {
        return reply.status(400).send({ error: "invalid_request", details: parsed.error.flatten() })
      }
      try {
        const data = parsed.data
        const result = data.operation === "initialize"
          ? initializeIncrementalStream(getStore(), {
              streamId: data.streamId,
              region: data.region,
              asOf: new Date(data.asOf),
              initialPopulation: data.initialPopulation,
              seed: data.seed ?? randomInt(0, 2_147_483_648),
              scenario: data.scenario,
              sourceMode: data.sourceMode,
              sources: data.sources,
              anomalyRate: data.anomalyRate,
              maxAnomaliesPerPerson: data.maxAnomaliesPerPerson,
              newPeoplePerDay: data.newPeoplePerDay,
              maxEvents: data.maxEvents,
            })
          : advanceIncrementalStream(getStore(), {
              streamId: data.streamId,
              asOf: new Date(data.asOf),
              sources: data.sources,
              newPeople: data.newPeople,
              emitCurrentRecord: data.emitCurrentRecord,
              maxEvents: data.maxEvents,
            })
        const pending = getStore().pendingEvents(data.streamId)
        const published = pending.length > 0 ? await publisher.publishIncremental(pending) : undefined
        if (published) getStore().markPublished(pending.map((event) => event.id), new Date().toISOString())
        return reply.send({
          mode: "incremental",
          operation: data.operation,
          streamId: data.streamId,
          region: result.state.region,
          previousCursor: result.previousCursor,
          cursor: result.state.cursor,
          seed: result.state.seed,
          people: result.state.people.length,
          newPeople: result.newPeople,
          eventsGenerated: result.events.length,
          eventsPublished: pending.length,
          activeAnomalies: result.state.anomalies.filter((anomaly) => anomaly.status === "ACTIVE").length,
          idempotent: result.idempotent,
          published,
        })
      } catch (error) {
        if (error instanceof StreamAlreadyExistsError || error instanceof StreamConflictError) {
          return reply.status(409).send({ error: "stream_conflict", message: error.message })
        }
        if (error instanceof IncrementalRequestError || error instanceof IncrementalEventLimitError) {
          return reply.status(400).send({ error: "invalid_incremental_request", message: error.message })
        }
        request.log.error({ err: error }, "failed to generate or publish incremental synthetic data")
        return reply.status(502).send({
          error: "incremental_publish_failed",
          message: error instanceof Error ? error.message : "Unknown incremental error",
        })
      }
    }

    const parsed = GenerateRequestSchema.safeParse(request.body ?? {})
    if (!parsed.success) {
      return reply.status(400).send({
        error: "invalid_request",
        details: parsed.error.flatten(),
      })
    }

    const seed = parsed.data.seed ?? randomInt(0, 2_147_483_648)
    const generated = generateSyntheticData({
      region: parsed.data.region,
      count: parsed.data.count,
      seed,
      generatedAt: parsed.data.generatedAt ? new Date(parsed.data.generatedAt) : undefined,
      scenario: parsed.data.scenario,
      sourceMode: parsed.data.sourceMode,
      sources: parsed.data.sources,
      anomalyRate: parsed.data.anomalyRate,
      maxAnomaliesPerPerson: parsed.data.maxAnomaliesPerPerson,
    })

    try {
      const published = await publisher.publish(generated)
      const response: GenerateResponse = {
        region: generated.dataset.region,
        generatedAt: generated.dataset.generatedAt,
        seed,
        peopleGenerated: generated.records.length,
        coverage: generated.coverage,
        anomalies: {
          total: generated.manifest.length,
          manifest: parsed.data.includeManifest ? generated.manifest : undefined,
        },
        published,
      }
      return reply.send(response)
    } catch (error) {
      request.log.error({ err: error }, "failed to publish synthetic data to Redpanda")
      return reply.status(502).send({
        error: "redpanda_publish_failed",
        message: error instanceof Error ? error.message : "Unknown Redpanda error",
      })
    }
  })

  server.addHook("onClose", async () => {
    await publisher.close()
    store?.close()
  })

  return server
}

async function main() {
  const server = buildServer()
  const port = Number(process.env.PORT ?? 4020)
  const host = process.env.HOST ?? "0.0.0.0"

  const close = async () => {
    await server.close()
    process.exit(0)
  }
  process.once("SIGINT", close)
  process.once("SIGTERM", close)

  await server.listen({ port, host })
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main()
}
