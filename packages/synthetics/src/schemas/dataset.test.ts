import assert from "node:assert/strict"
import { describe, it } from "node:test"
import { SyntheticDatasetSchema } from "./dataset.js"

const person = {
  personId: "person-minh-anh",
  fullName: "Trần Minh Anh",
  fullNameAscii: "Tran Minh Anh",
  familyName: "Trần",
  middleName: "Minh",
  givenName: "Anh",
  dateOfBirth: "1992-04-18",
  gender: "FEMALE" as const,
  nationalId: "079192001234",
  phoneNumber: "+84901234567",
  emailAddress: "minh.anh@example.vn",
  nationalityCode: "VN" as const,
  residenceCountryCode: "VN" as const,
  preferredLanguage: "vi-VN" as const,
  address: {
    addressLine1: "12 Nguyễn Huệ",
    ward: "Bến Nghé",
    district: "Quận 1",
    province: "Thành phố Hồ Chí Minh",
    countryCode: "VN" as const,
  },
}

function t24Customer(id: string, company: string) {
  return {
    application: "CUSTOMER" as const,
    header: {
      id,
      status: "success" as const,
      transactionStatus: "Live" as const,
      versionNumber: "1",
      recordedAt: "2026-10-12T01:00:00.000Z",
    },
    body: {
      personId: person.personId,
      customerMnemonic: "MINHANH",
      sectorId: 1001,
      gender: person.gender,
      nationalityId: "VN" as const,
      residenceId: "VN" as const,
      displayNames: [{ displayName: person.fullName }],
      customerNames: [{ customerName: person.fullName }],
      language: "vi-VN" as const,
      dateOfBirth: person.dateOfBirth,
      communicationDevices: [
        { phoneNumber: person.phoneNumber, email: person.emailAddress },
      ],
      legalIdentifiers: [
        {
          legalId: person.nationalId,
          legalDocumentName: "CCCD" as const,
          countryOfIssue: "VN" as const,
          issueDate: "2021-06-01",
          expiryDate: "2047-04-18",
        },
      ],
      addresses: [{ addressType: "HOME" as const, ...person.address }],
      target: 1,
      customerStatus: 1,
      industryId: 1000,
      accountOfficerId: "AO-01",
      customerCompany: company,
      amlCheck: "PASSED" as const,
    },
  }
}

function dataset() {
  return {
    schemaVersion: "2.0.0",
    region: "vn",
    generatedAt: "2026-10-12T01:00:00.000Z",
    seed: 42,
    people: [person],
    organizations: {
      vikkiBank: {
        organization: {
          organizationId: "vikki-bank",
          legalName: "Ngân hàng TNHH MTV Số Vikki",
          countryCode: "VN",
          timezone: "Asia/Ho_Chi_Minh",
        },
        sourceSystem: "TEMENOS_T24",
        customers: [t24Customer("VIK-C-001", "VN0010001")],
        accounts: [],
        wallets: [],
        fundsTransfers: [],
      },
      hdbank: {
        organization: {
          organizationId: "hdbank",
          legalName: "Ngân hàng TMCP Phát triển Thành phố Hồ Chí Minh",
          countryCode: "VN",
          timezone: "Asia/Ho_Chi_Minh",
        },
        sourceSystem: "TEMENOS_T24",
        customers: [t24Customer("HDB-C-001", "VN0020001")],
        accounts: [],
        loans: [],
        fundsTransfers: [],
      },
      crm: {
        organization: {
          organizationId: "group-crm",
          legalName: "Sovico Group CRM",
          countryCode: "VN",
          timezone: "Asia/Ho_Chi_Minh",
        },
        sourceSystem: "GROUP_CRM",
        pollBatches: [],
        contacts: [],
        campaignInteractions: [],
      },
      vietjetAir: {
        organization: {
          organizationId: "vietjet-air",
          legalName: "Công ty Cổ phần Hàng không VietJet",
          countryCode: "VN",
          timezone: "Asia/Ho_Chi_Minh",
        },
        sourceSystem: "VIETJET_RESERVATION",
        customers: [
          {
            customerId: "VJ-C-001",
            personId: person.personId,
            fullName: person.fullName,
            dateOfBirth: person.dateOfBirth,
            gender: person.gender,
            nationalId: person.nationalId,
            nationalityCode: "VN",
            phoneNumber: person.phoneNumber,
            emailAddress: person.emailAddress,
            preferredLanguage: "vi-VN",
          },
        ],
        bookings: [],
      },
    },
  }
}

describe("SyntheticDatasetSchema", () => {
  it("accepts one consistent Vietnamese identity across the organization datasets", () => {
    assert.equal(SyntheticDatasetSchema.safeParse(dataset()).success, true)
  })

  it("allows structurally valid identity drift between source systems", () => {
    const input = dataset()
    input.organizations.vietjetAir.customers[0].phoneNumber = "+84909999999"

    assert.equal(SyntheticDatasetSchema.safeParse(input).success, true)
  })
})
