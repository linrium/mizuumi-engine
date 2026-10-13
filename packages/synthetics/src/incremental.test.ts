import assert from "node:assert/strict"
import { describe, it } from "node:test"
import {
  advanceIncrementalStream,
  IncrementalRequestError,
  initializeIncrementalStream,
} from "./incremental.js"
import { IncrementalStore } from "./incremental-store.js"

describe("incremental synthetic streams", () => {
  it("evolves persisted state without violating balances or anomaly limits", () => {
    const store = new IncrementalStore(":memory:")
    initializeIncrementalStream(store, {
      streamId: "th-lifecycle",
      region: "th",
      asOf: new Date("2026-01-01T00:00:00.000Z"),
      initialPopulation: 20,
      seed: 7,
      scenario: "stress",
      sourceMode: "all",
      maxAnomaliesPerPerson: 2,
      newPeoplePerDay: 0,
      maxEvents: 1_000,
    })

    const result = advanceIncrementalStream(store, {
      streamId: "th-lifecycle",
      asOf: new Date("2026-04-01T00:00:00.000Z"),
      newPeople: 2,
      maxEvents: 10_000,
    })

    assert.equal(result.state.people.length, 22)
    assert.equal(new Set(result.events.map((event) => event.id)).size, result.events.length)
    for (const person of result.state.people) {
      assert.ok((person.record.hdbank?.account?.body.workingBalance.amount ?? 0) >= 0)
      assert.ok((person.record.vikkiBank?.account?.body.workingBalance.amount ?? 0) >= 0)
      const active = result.state.anomalies.filter(
        (anomaly) => anomaly.status === "ACTIVE" && anomaly.truthId === person.record.person.personId,
      )
      assert.ok(active.length <= 2)
    }

    const repeated = advanceIncrementalStream(store, {
      streamId: "th-lifecycle",
      asOf: new Date("2026-04-01T00:00:00.000Z"),
      maxEvents: 10_000,
    })
    assert.equal(repeated.idempotent, true)
    assert.deepEqual(repeated.events, [])
    store.close()
  })

  it("rejects time travel", () => {
    const store = new IncrementalStore(":memory:")
    initializeIncrementalStream(store, {
      streamId: "no-time-travel",
      region: "vn",
      asOf: new Date("2026-10-13T00:00:00.000Z"),
      initialPopulation: 1,
      seed: 42,
      scenario: "clean",
      sourceMode: "all",
      maxAnomaliesPerPerson: 1,
      newPeoplePerDay: 0,
      maxEvents: 100,
    })
    assert.throws(
      () => advanceIncrementalStream(store, {
        streamId: "no-time-travel",
        asOf: new Date("2026-10-12T00:00:00.000Z"),
        maxEvents: 100,
      }),
      IncrementalRequestError,
    )
    store.close()
  })

  it("coalesces one selected source into one current record", () => {
    const store = new IncrementalStore(":memory:")
    initializeIncrementalStream(store, {
      streamId: "one-record",
      region: "vn",
      asOf: new Date("2026-01-01T00:00:00.000Z"),
      initialPopulation: 1,
      seed: 42,
      scenario: "stress",
      sourceMode: "all",
      sources: ["hdbank"],
      maxAnomaliesPerPerson: 3,
      newPeoplePerDay: 0,
      maxEvents: 100,
    })
    const result = advanceIncrementalStream(store, {
      streamId: "one-record",
      asOf: new Date("2026-04-01T00:00:00.000Z"),
      sources: ["hdbank"],
      newPeople: 0,
      emitCurrentRecord: true,
      maxEvents: 100,
    })
    assert.equal(result.events.length, 1)
    assert.equal(result.events[0]?.source, "hdbank")
    store.close()
  })
})
