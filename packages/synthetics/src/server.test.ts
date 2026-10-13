import assert from "node:assert/strict"
import { describe, it } from "node:test"
import type { GeneratedSyntheticData } from "./generator.js"
import type { IncrementalEvent } from "./events.js"
import { IncrementalStore } from "./incremental-store.js"
import type { SyntheticPublisher } from "./redpanda.js"
import { buildServer, type GenerateResponse } from "./server.js"

class TestPublisher implements SyntheticPublisher {
  published: GeneratedSyntheticData[] = []
  incremental: IncrementalEvent[] = []

  async publish(generated: GeneratedSyntheticData) {
    this.published.push(generated)
    const messages = {
      hdbank: generated.coverage.hdbank,
      vikkiBank: generated.coverage.vikkiBank,
      vietjetAir: generated.coverage.vietjetAir,
      crm: generated.coverage.crm,
    }
    return {
      region: generated.dataset.region,
      topics: {
        hdbank: `${generated.dataset.region}.test.hdbank`,
        vikkiBank: `${generated.dataset.region}.test.vikki`,
        vietjetAir: `${generated.dataset.region}.test.vietjetair`,
        crm: `${generated.dataset.region}.test.crm`,
      },
      messages: { ...messages, total: Object.values(messages).reduce((sum, count) => sum + count, 0) },
    }
  }

  async publishIncremental(events: IncrementalEvent[]) {
    this.incremental.push(...events)
    const region = events[0]!.region
    const messages = {
      hdbank: events.filter((event) => event.source === "hdbank").length,
      vikkiBank: events.filter((event) => event.source === "vikkiBank").length,
      vietjetAir: events.filter((event) => event.source === "vietjetAir").length,
      crm: events.filter((event) => event.source === "crm").length,
      total: events.length,
    }
    return {
      region,
      topics: {
        hdbank: `${region}.test.hdbank`,
        vikkiBank: `${region}.test.vikki`,
        vietjetAir: `${region}.test.vietjetair`,
        crm: `${region}.test.crm`,
      },
      messages,
    }
  }

  async close() {}
}

describe("POST /api/v1/generate", () => {
  it("generates selected scenarios and returns coverage and anomaly metadata", async () => {
    const publisher = new TestPublisher()
    const server = buildServer({ publisher, logger: false })

    const response = await server.inject({
      method: "POST",
      url: "/api/v1/generate",
      payload: {
        region: "th",
        count: 2,
        seed: 42,
        generatedAt: "2026-10-13T01:00:00.000Z",
        scenario: "clean",
        sourceMode: "all",
        includeManifest: true,
      },
    })
    await server.close()

    assert.equal(response.statusCode, 200)
    const body = response.json<GenerateResponse>()
    assert.equal(body.seed, 42)
    assert.equal(body.region, "th")
    assert.equal(body.peopleGenerated, 2)
    assert.equal(body.published.messages.total, 8)
    assert.deepEqual(body.coverage, { hdbank: 2, vikkiBank: 2, vietjetAir: 2, crm: 2 })
    assert.equal(body.anomalies.total, 0)
    assert.deepEqual(body.anomalies.manifest, [])
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

  it("initializes and advances a persistent incremental stream idempotently", async () => {
    const publisher = new TestPublisher()
    const server = buildServer({ publisher, store: new IncrementalStore(":memory:"), logger: false })
    const initialized = await server.inject({
      method: "POST",
      url: "/api/v1/generate",
      payload: {
        mode: "incremental",
        operation: "initialize",
        streamId: "vn-demo",
        region: "vn",
        asOf: "2026-10-13T01:00:00.000Z",
        initialPopulation: 5,
        seed: 42,
        scenario: "clean",
        sourceMode: "all",
        sources: ["hdbank"],
        newPeoplePerDay: 0,
      },
    })
    assert.equal(initialized.statusCode, 200)
    assert.equal(initialized.json().eventsGenerated, 5)
    assert.equal(publisher.incremental.length, 5)

    const advanced = await server.inject({
      method: "POST",
      url: "/api/v1/generate",
      payload: {
        mode: "incremental",
        operation: "advance",
        streamId: "vn-demo",
        asOf: "2026-10-14T01:00:00.000Z",
        sources: ["crm"],
        newPeople: 1,
      },
    })
    assert.equal(advanced.statusCode, 200)
    assert.equal(advanced.json().newPeople, 1)
    assert.equal(advanced.json().people, 6)
    assert.ok(advanced.json().eventsGenerated >= 6)

    const repeated = await server.inject({
      method: "POST",
      url: "/api/v1/generate",
      payload: {
        mode: "incremental",
        operation: "advance",
        streamId: "vn-demo",
        asOf: "2026-10-14T01:00:00.000Z",
      },
    })
    assert.equal(repeated.statusCode, 200)
    assert.equal(repeated.json().idempotent, true)
    assert.equal(repeated.json().eventsGenerated, 0)
    assert.equal(repeated.json().eventsPublished, 0)
    await server.close()
  })
})
