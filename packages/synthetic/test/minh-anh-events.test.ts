import assert from "node:assert/strict"
import { mkdtemp } from "node:fs/promises"
import os from "node:os"
import path from "node:path"
import test from "node:test"
import { generateDataset } from "../src/generator.js"
import { loadMinhAnhEvents, Topics } from "../src/minh-anh-events.js"
import { parsePublishArgs } from "../src/publish-minh-anh.js"
import { cleanupTopics, demoTopics, parseCleanupArgs } from "../src/cleanup-topics.js"

test("uses the Mizuumi external Redpanda listener by default", () => {
  const previous = process.env.REDPANDA_BROKERS
  delete process.env.REDPANDA_BROKERS
  try {
    const options = parsePublishArgs([])
    assert.notEqual(options, "help")
    if (options !== "help") assert.deepEqual(options.brokers, ["redpanda-0.api.redpanda.mizuumi.test:30092"])
  } finally {
    if (previous === undefined) delete process.env.REDPANDA_BROKERS
    else process.env.REDPANDA_BROKERS = previous
  }
})

test("routes the complete Minh Anh history to source-appropriate topics", async () => {
  const outputDirectory = await mkdtemp(path.join(os.tmpdir(), "mizuumi-publish-"))
  await generateDataset({ outputDirectory, customerCount: 1, seed: 42, asOf: "2026-10-13", force: false })

  const events = await loadMinhAnhEvents(outputDirectory)
  assert.equal(events.length, 32)
  assert.deepEqual(new Set(events.map((event) => event.topic)), new Set(Object.values(Topics)))
  assert.deepEqual(demoTopics(), ["vn.crm.events", "vn.hdbank.t24.events", "vn.vikki.t24.events", "vn.vietjet.events"])
  assert.equal(events.filter((event) => event.type === "com.hdbank.t24.loan.repayment.completed").length, 3)
  assert.equal(events.filter((event) => event.type === "com.vietjet.booking.recorded").length, 3)
  assert.equal(events.filter((event) => event.source === "/crm").length, 2)
  assert.equal(events.find((event) => event.key === "VIK-TXN-VJ-8K2M4Q")?.topic, Topics.vikki)
  assert.equal(events.find((event) => event.key === "HDB-CUST-000001")?.topic, Topics.hdbank)
})

test("topic cleanup is non-destructive without explicit confirmation", async () => {
  const options = parseCleanupArgs([])
  assert.notEqual(options, "help")
  if (options === "help") return
  await assert.rejects(cleanupTopics(options), /Re-run with --yes/)

  const preview = parseCleanupArgs(["--dry-run"])
  assert.notEqual(preview, "help")
  if (preview !== "help") assert.deepEqual(await cleanupTopics(preview), demoTopics())
})
