import assert from "node:assert/strict"
import { describe, it } from "node:test"
import { generateSyntheticData } from "./generator.js"

const generatedAt = new Date("2026-10-13T01:00:00.000Z")

describe("generateSyntheticData", () => {
  it("creates clean source projections with source-local identifiers", () => {
    const generated = generateSyntheticData({
      region: "vn",
      count: 20,
      seed: 42,
      generatedAt,
      scenario: "clean",
      sourceMode: "all",
    })

    assert.deepEqual(generated.coverage, {
      hdbank: 20,
      vikkiBank: 20,
      vietjetAir: 20,
      crm: 20,
    })
    assert.equal(generated.manifest.length, 0)
    const first = generated.records[0]!
    assert.ok(first.hdbank)
    assert.ok(first.vikkiBank)
    assert.ok(first.vietjetAir)
    assert.ok(first.crm)
    assert.notEqual(first.hdbank.customer.body.personId, first.person.personId)
    assert.notEqual(first.vikkiBank.customer.body.personId, first.hdbank.customer.body.personId)
    assert.equal(
      first.vikkiBank.customer.body.displayNames[0]?.displayName,
      first.person.fullName,
    )
    assert.equal(first.hdbank.customer.body.displayNames[0]?.displayName, first.person.fullName)
    assert.equal(first.vietjetAir.customer.fullName, first.person.fullName)
    assert.equal(first.crm.contact.fullName, first.person.fullName)
  })

  it("is deterministic for the same seed and timestamp", () => {
    const options = {
      region: "vn" as const,
      count: 10,
      seed: 7,
      generatedAt,
      scenario: "matcher-demo" as const,
    }
    assert.deepEqual(generateSyntheticData(options), generateSyntheticData(options))
  })

  it("can publish sources in separate calls without changing their projections", () => {
    const options = {
      region: "vn" as const,
      count: 25,
      seed: 73,
      generatedAt,
      scenario: "matcher-demo" as const,
      sourceMode: "all" as const,
    }
    const complete = generateSyntheticData(options)
    const hdbankOnly = generateSyntheticData({ ...options, sources: ["hdbank"] })
    const vietjetOnly = generateSyntheticData({ ...options, sources: ["vietjetAir"] })

    assert.deepEqual(
      hdbankOnly.records.map((record) => record.hdbank),
      complete.records.map((record) => record.hdbank),
    )
    assert.deepEqual(
      vietjetOnly.records.map((record) => record.vietjetAir),
      complete.records.map((record) => record.vietjetAir),
    )
    assert.deepEqual(hdbankOnly.coverage, { hdbank: 25, vikkiBank: 0, vietjetAir: 0, crm: 0 })
    assert.deepEqual(vietjetOnly.coverage, { hdbank: 0, vikkiBank: 0, vietjetAir: 25, crm: 0 })
  })

  it("samples partial source coverage and optional products", () => {
    const generated = generateSyntheticData({
      region: "vn",
      count: 500,
      seed: 91,
      generatedAt,
      scenario: "clean",
    })

    for (const count of Object.values(generated.coverage)) {
      assert.ok(count > 0)
      assert.ok(count < 500)
    }
    assert.ok(generated.records.some((record) => !record.hdbank))
    assert.ok(generated.records.some((record) => !record.vikkiBank))
    assert.ok(generated.records.some((record) => !record.vietjetAir))
    assert.ok(generated.records.some((record) => record.hdbank && !record.hdbank.loan))
    assert.ok(generated.records.some((record) => record.vietjetAir?.bookings.length === 0))
  })

  it("creates declared, structurally valid identity anomalies", () => {
    const generated = generateSyntheticData({
      region: "vn",
      count: 200,
      seed: 19,
      generatedAt,
      scenario: "matcher-demo",
      sourceMode: "all",
    })

    assert.ok(generated.manifest.length > 25)
    for (const anomaly of generated.manifest) {
      assert.notEqual(anomaly.canonicalValue, anomaly.observedValue)
      assert.match(anomaly.sourceRecordId, /^(HDB|VIK|VJ|CRM)-C-/)
    }
  })

  it("keeps balances, payments, loans, and booking times coherent", () => {
    const generated = generateSyntheticData({
      region: "vn",
      count: 200,
      seed: 24,
      generatedAt,
      scenario: "stress",
      sourceMode: "all",
    })

    for (const record of generated.records) {
      for (const bank of [record.hdbank, record.vikkiBank]) {
        if (!bank?.account) continue
        const finalBalance = bank.account.body.workingBalance.amount
        assert.ok(finalBalance >= 0)
        const transfers = [...bank.fundsTransfers].sort((left, right) =>
          left.body.bookingDate.localeCompare(right.body.bookingDate),
        )
        let available =
          finalBalance +
          transfers.reduce((sum, transfer) => sum + transfer.body.transactionAmount.amount, 0)
        for (const transfer of transfers) {
          assert.ok(transfer.body.transactionAmount.amount <= available)
          available -= transfer.body.transactionAmount.amount
        }
        assert.equal(available, finalBalance)
      }

      if (record.hdbank?.loan) {
        const principal = record.hdbank.loan.body.originalPrincipal.amount
        const scheduledPrincipal = record.hdbank.loan.body.repaymentSchedule.reduce(
          (sum, item) => sum + item.principal.amount,
          0,
        )
        assert.equal(scheduledPrincipal, principal)
        const firstPayment = record.hdbank.loan.body.repaymentSchedule[0]!
        assert.ok(
          firstPayment.principal.amount + firstPayment.interest.amount <=
            record.profile.monthlyIncome * 0.35 + 1,
        )
      }

      for (const booking of record.vietjetAir?.bookings ?? []) {
        assert.ok(
          booking.totalAmount.amount <=
            (record.profile.monthlyIncome - record.profile.monthlyExpenses) * 0.35 + 1,
        )
        const segment = booking.segments[0]!
        assert.ok(booking.bookedAt < segment.scheduledDepartureAt)
        assert.ok(segment.scheduledDepartureAt < segment.scheduledArrivalAt)
        if (booking.payment.method === "CASH") continue
        const transfers =
          booking.payment.method === "VIKKI_WALLET"
            ? record.vikkiBank?.fundsTransfers
            : record.hdbank?.fundsTransfers
        const transfer = transfers?.find(
          (item) =>
            item.body.transactionReference === booking.payment.externalTransactionReference,
        )
        assert.equal(transfer?.body.transactionAmount.amount, booking.totalAmount.amount)
      }
    }
  })

  it("uses the Thailand engine for Thai identities and THB amounts", () => {
    const generated = generateSyntheticData({
      region: "th",
      count: 20,
      seed: 24,
      generatedAt,
      scenario: "clean",
      sourceMode: "all",
    })
    const first = generated.records[0]!
    assert.equal(first.person.nationalityCode, "TH")
    assert.equal(first.person.preferredLanguage, "th-TH")
    assert.match(first.person.phoneNumber, /^\+66/)
    assert.equal(first.person.nationalId.length, 13)
    assert.equal(first.profile.currency, "THB")
    assert.ok(
      generated.dataset.organizations.hdbank.accounts.every(
        (account) => account.body.currency === "THB",
      ),
    )
  })
})
