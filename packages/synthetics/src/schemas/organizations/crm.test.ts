import assert from "node:assert/strict"
import { describe, it } from "node:test"
import { CrmDatasetSchema } from "./crm.js"

const crmDataset = {
  organization: {
    organizationId: "group-crm",
    legalName: "Sovico Group CRM",
    countryCode: "VN",
    timezone: "Asia/Ho_Chi_Minh",
  },
  sourceSystem: "GROUP_CRM",
  pollBatches: [
    {
      pollId: "crm-poll-001",
      startedAt: "2026-10-13T01:00:00.000Z",
      completedAt: "2026-10-13T01:00:02.000Z",
      cursor: "contact-001",
      status: "COMPLETED",
      recordsRead: 1,
    },
  ],
  contacts: [
    {
      contactId: "CRM-C-001",
      personId: "person-minh-anh",
      sourceRecordId: "salesforce-contact-001",
      fullName: "Trần Minh Anh",
      fullNameAscii: "Tran Minh Anh",
      dateOfBirth: "1992-04-18",
      nationalId: "079192001234",
      phoneNumber: "+84901234567",
      emailAddress: "minh.anh@example.vn",
      preferredLanguage: "vi-VN",
      countryCode: "VN",
      lifecycleStage: "CUSTOMER",
      status: "ACTIVE",
      createdAt: "2025-08-20T03:30:00.000Z",
      updatedAt: "2026-10-13T01:00:00.000Z",
    },
  ],
  campaignInteractions: [
    {
      interactionId: "CRM-I-001",
      contactId: "CRM-C-001",
      campaignId: "CAMPAIGN-TET-2026",
      campaignName: "Ưu đãi Tết 2026",
      channel: "SMS",
      direction: "OUTBOUND",
      outcome: "DELIVERED",
      occurredAt: "2026-01-15T02:00:00.000Z",
    },
  ],
}

describe("CrmDatasetSchema", () => {
  it("accepts Vietnamese CRM contacts, poll metadata, and campaign interactions", () => {
    assert.equal(CrmDatasetSchema.safeParse(crmDataset).success, true)
  })

  it("allows source identity values to differ for future anomaly generation", () => {
    const input = structuredClone(crmDataset)
    input.contacts[0].fullName = "Tran Minh Anh"
    input.contacts[0].phoneNumber = "+84908888888"

    assert.equal(CrmDatasetSchema.safeParse(input).success, true)
  })
})
