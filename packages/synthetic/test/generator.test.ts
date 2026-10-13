import assert from "node:assert/strict"
import { mkdtemp, readFile } from "node:fs/promises"
import os from "node:os"
import path from "node:path"
import test from "node:test"
import { generateDataset } from "../src/generator.js"

test("generates the featured customer with linked histories and intentional CRM drift", async () => {
  const outputDirectory = await mkdtemp(path.join(os.tmpdir(), "mizuumi-synthetic-"))
  const counts = await generateDataset({
    outputDirectory,
    customerCount: 20,
    seed: 42,
    asOf: "2026-10-13",
    force: false,
  })

  assert.equal(counts["hdbank/loan_repayments.csv"] >= 3, true)
  assert.equal(counts["vietjet/bookings.csv"] >= 3, true)

  const vikki = await readFile(path.join(outputDirectory, "vikki/customers.csv"), "utf8")
  const hdbank = await readFile(path.join(outputDirectory, "hdbank/customers.csv"), "utf8")
  const crm = await readFile(path.join(outputDirectory, "crm/contacts.csv"), "utf8")
  const bookings = await readFile(path.join(outputDirectory, "vietjet/bookings.csv"), "utf8")
  const wallet = await readFile(path.join(outputDirectory, "vikki/wallet_transactions.csv"), "utf8")
  const repayments = await readFile(path.join(outputDirectory, "hdbank/loan_repayments.csv"), "utf8")
  const interactions = await readFile(path.join(outputDirectory, "crm/campaign_interactions.csv"), "utf8")

  assert.match(vikki, /Trần Minh Anh,1992-04-18,079192700001,\+84902204567,minh\.anh\.tran@example\.vn/)
  assert.match(hdbank, /Trần Minh Anh,1992-04-18,079192700001,\+84902204567,minh\.anh\.tran@example\.vn/)
  assert.match(crm, /Tran Minh Anh,1992-04-18,,\+84912204567,minh\.anh\.tran@example\.vn/)
  assert.match(bookings, /VJ-8K2M4Q.*vikki_wallet,VIK-TXN-VJ-8K2M4Q/)
  assert.match(wallet, /VIK-TXN-VJ-8K2M4Q.*VietJet Air,2186000,completed,VJ-8K2M4Q/)
  assert.equal(repayments.split("\n").filter((row) => row.includes("HDB-LOAN-000001")).length, 3)
  assert.equal(bookings.split("\n").filter((row) => row.includes("VJ-PAX-000001")).length, 3)
  assert.equal(interactions.split("\n").filter((row) => row.includes("CRM-CONTACT-000001")).length, 1)

  const manifest = JSON.parse(await readFile(path.join(outputDirectory, "manifest.json"), "utf8")) as { synthetic_people: number }
  assert.equal(manifest.synthetic_people, 20)
})

test("same seed and reference date produce identical CSV data", async () => {
  const first = await mkdtemp(path.join(os.tmpdir(), "mizuumi-synthetic-a-"))
  const second = await mkdtemp(path.join(os.tmpdir(), "mizuumi-synthetic-b-"))
  const options = { customerCount: 10, seed: 7, asOf: "2026-10-13", force: false }
  await generateDataset({ ...options, outputDirectory: first })
  await generateDataset({ ...options, outputDirectory: second })
  assert.equal(
    await readFile(path.join(first, "hdbank/customers.csv"), "utf8"),
    await readFile(path.join(second, "hdbank/customers.csv"), "utf8"),
  )
})
