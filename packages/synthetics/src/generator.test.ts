import assert from "node:assert/strict"
import { describe, it } from "node:test"
import { generateSyntheticData } from "./generator.js"

describe("generateSyntheticData", () => {
  it("creates consistent Vikki, HDBank, and VietJet records", () => {
    const generated = generateSyntheticData({
      region: "vn",
      count: 2,
      seed: 42,
      generatedAt: new Date("2026-10-13T01:00:00.000Z"),
    })

    assert.equal(generated.records.length, 2)
    assert.equal(generated.dataset.region, "vn")
    assert.equal(generated.dataset.organizations.vikkiBank.customers.length, 2)
    assert.equal(generated.dataset.organizations.hdbank.loans.length, 2)
    assert.equal(generated.dataset.organizations.vietjetAir.bookings.length, 6)

    const first = generated.records[0]
    assert.ok(first)
    assert.equal(first.vikkiBank.customer.body.personId, first.person.personId)
    assert.equal(first.hdbank.customer.body.personId, first.person.personId)
    assert.equal(first.vietjetAir.customer.personId, first.person.personId)
    assert.equal(first.hdbank.loan.body.repaymentSchedule.filter((item) => item.status === "PAID").length, 3)
    assert.equal(first.vietjetAir.bookings.length, 3)
    assert.equal(first.vietjetAir.bookings[0]?.payment.method, "VIKKI_WALLET")
  })

  it("is deterministic for the same seed and timestamp", () => {
    const options = {
      region: "vn" as const,
      count: 1,
      seed: 7,
      generatedAt: new Date("2026-10-13T01:00:00.000Z"),
    }

    assert.deepEqual(generateSyntheticData(options), generateSyntheticData(options))
  })

  it("uses the Thailand engine for Thai identities and THB amounts", () => {
    const generated = generateSyntheticData({
      region: "th",
      count: 1,
      seed: 24,
      generatedAt: new Date("2026-10-13T01:00:00.000Z"),
    })

    const first = generated.records[0]
    assert.ok(first)
    assert.equal(generated.dataset.region, "th")
    assert.equal(first.person.nationalityCode, "TH")
    assert.equal(first.person.preferredLanguage, "th-TH")
    assert.match(first.person.phoneNumber, /^\+66/)
    assert.equal(first.person.nationalId.length, 13)
    assert.equal(first.hdbank.account.body.currency, "THB")
    assert.equal(first.vikkiBank.fundsTransfers[0]?.body.transactionAmount.currency, "THB")
  })
})
