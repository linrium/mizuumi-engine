import assert from "node:assert/strict"
import { describe, it } from "node:test"
import type { GeneratedSyntheticData } from "./generator.js"
import type { SyntheticPublisher } from "./redpanda.js"
import { buildServer, type GenerateResponse } from "./server.js"

class TestPublisher implements SyntheticPublisher {
  published: GeneratedSyntheticData[] = []

  async publish(generated: GeneratedSyntheticData) {
    this.published.push(generated)
    const count = generated.records.length
    return {
      region: generated.dataset.region,
      topics: {
        hdbank: `${generated.dataset.region}.test.hdbank`,
        vikkiBank: `${generated.dataset.region}.test.vikki`,
        vietjetAir: `${generated.dataset.region}.test.vietjetair`,
      },
      messages: { hdbank: count, vikkiBank: count, vietjetAir: count, total: count * 3 },
    }
  }

  async close() {}
}

describe("POST /api/v1/generate", () => {
  it("generates and publishes one message per person per organization", async () => {
    const publisher = new TestPublisher()
    const server = buildServer({ publisher, logger: false })

    const response = await server.inject({
      method: "POST",
      url: "/api/v1/generate",
      payload: { region: "th", count: 2, seed: 42 },
    })
    await server.close()

    assert.equal(response.statusCode, 200)
    const body = response.json<GenerateResponse>()
    assert.equal(body.seed, 42)
    assert.equal(body.region, "th")
    assert.equal(body.peopleGenerated, 2)
    assert.equal(body.published.messages.total, 6)
    assert.equal(publisher.published.length, 1)
    assert.equal(publisher.published[0]?.dataset.region, "th")
  })

  it("rejects an invalid generation count", async () => {
    const server = buildServer({ publisher: new TestPublisher(), logger: false })
    const response = await server.inject({
      method: "POST",
      url: "/api/v1/generate",
      payload: { count: 101 },
    })
    await server.close()

    assert.equal(response.statusCode, 400)
  })
})
