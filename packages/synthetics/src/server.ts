import { randomInt } from "node:crypto"
import { pathToFileURL } from "node:url"
import Fastify from "fastify"
import { z } from "zod"
import { generateSyntheticData } from "./generator.js"
import {
  RedpandaPublisher,
  type PublishSummary,
  type SyntheticPublisher,
} from "./redpanda.js"

const GenerateRequestSchema = z.object({
  region: z.enum(["vn", "th"]).default("vn"),
  count: z.number().int().min(1).max(100).default(1),
  seed: z.number().int().min(0).max(2_147_483_647).optional(),
})

export type GenerateResponse = {
  region: "vn" | "th"
  generatedAt: string
  seed: number
  peopleGenerated: number
  published: PublishSummary
}

export function buildServer(
  options: { publisher?: SyntheticPublisher; logger?: boolean } = {},
) {
  const server = Fastify({ logger: options.logger ?? true })
  const publisher = options.publisher ?? new RedpandaPublisher()

  server.post("/api/v1/generate", async (request, reply) => {
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
    })

    try {
      const published = await publisher.publish(generated)
      const response: GenerateResponse = {
        region: generated.dataset.region,
        generatedAt: generated.dataset.generatedAt,
        seed,
        peopleGenerated: generated.records.length,
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
