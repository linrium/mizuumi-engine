import { createReadStream } from "node:fs"
import { access } from "node:fs/promises"
import path from "node:path"
import readline from "node:readline"

export interface SourceEvent {
  topic: string
  key: string
  source: string
  type: string
  subject: string
  data: Record<string, string>
}

export const Topics = {
  crm: process.env.REDPANDA_TOPIC_CRM ?? "vn.crm.events",
  hdbank: process.env.REDPANDA_TOPIC_HDBANK ?? "vn.hdbank.t24.events",
  vikki: process.env.REDPANDA_TOPIC_VIKKI ?? "vn.vikki.t24.events",
  vietjet: process.env.REDPANDA_TOPIC_VIETJET ?? "vn.vietjet.events",
} as const

const IDS = {
  vikkiCustomer: "VIK-CUST-000001",
  vikkiAccount: "VIK-ACC-000001",
  hdbankCustomer: "HDB-CUST-000001",
  hdbankLoan: "HDB-LOAN-000001",
  crmContact: "CRM-CONTACT-000001",
  vietjetPassenger: "VJ-PAX-000001",
} as const

function parseCsvLine(line: string): string[] {
  const fields: string[] = []
  let field = ""
  let quoted = false
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index]
    if (character === '"') {
      if (quoted && line[index + 1] === '"') {
        field += '"'
        index += 1
      } else {
        quoted = !quoted
      }
    } else if (character === "," && !quoted) {
      fields.push(field)
      field = ""
    } else {
      field += character
    }
  }
  if (quoted) throw new Error("Malformed CSV: unterminated quoted field")
  fields.push(field)
  return fields
}

async function selectRows(
  inputDirectory: string,
  relativePath: string,
  predicate: (row: Record<string, string>) => boolean,
): Promise<Record<string, string>[]> {
  const filePath = path.join(inputDirectory, relativePath)
  await access(filePath)
  const lines = readline.createInterface({ input: createReadStream(filePath), crlfDelay: Infinity })
  let headers: string[] | undefined
  const matches: Record<string, string>[] = []
  for await (const line of lines) {
    if (!headers) {
      headers = parseCsvLine(line)
      continue
    }
    if (!line) continue
    const values = parseCsvLine(line)
    const row = Object.fromEntries(headers.map((header, index) => [header, values[index] ?? ""]))
    if (predicate(row)) matches.push(row)
  }
  return matches
}

function eventsForRows(
  rows: Record<string, string>[],
  options: Omit<SourceEvent, "key" | "subject" | "data"> & { keyField: string },
): SourceEvent[] {
  return rows.map((data) => {
    const key = data[options.keyField]
    if (!key) throw new Error(`Missing ${options.keyField} in ${options.type}`)
    return { topic: options.topic, key, source: options.source, type: options.type, subject: key, data }
  })
}

export async function loadMinhAnhEvents(inputDirectory: string): Promise<SourceEvent[]> {
  const input = path.resolve(inputDirectory)
  const [vikkiCustomers, vikkiAccounts, vikkiTransactions] = await Promise.all([
    selectRows(input, "vikki/customers.csv", (row) => row.customer_id === IDS.vikkiCustomer),
    selectRows(input, "vikki/accounts.csv", (row) => row.account_id === IDS.vikkiAccount),
    selectRows(input, "vikki/wallet_transactions.csv", (row) => row.account_id === IDS.vikkiAccount),
  ])
  const [hdbankCustomers, hdbankLoans, hdbankRepayments] = await Promise.all([
    selectRows(input, "hdbank/customers.csv", (row) => row.customer_id === IDS.hdbankCustomer),
    selectRows(input, "hdbank/loans.csv", (row) => row.loan_id === IDS.hdbankLoan),
    selectRows(input, "hdbank/loan_repayments.csv", (row) => row.loan_id === IDS.hdbankLoan),
  ])
  const [crmContacts, crmInteractions] = await Promise.all([
    selectRows(input, "crm/contacts.csv", (row) => row.contact_id === IDS.crmContact),
    selectRows(input, "crm/campaign_interactions.csv", (row) => row.contact_id === IDS.crmContact),
  ])
  const [vietjetPassengers, vietjetBookings] = await Promise.all([
    selectRows(input, "vietjet/passengers.csv", (row) => row.passenger_id === IDS.vietjetPassenger),
    selectRows(input, "vietjet/bookings.csv", (row) => row.passenger_id === IDS.vietjetPassenger),
  ])

  const required = [vikkiCustomers, vikkiAccounts, hdbankCustomers, hdbankLoans, crmContacts, vietjetPassengers]
  if (required.some((rows) => rows.length !== 1)) {
    throw new Error("The input does not contain exactly one complete Trần Minh Anh identity in every source")
  }

  return [
    ...eventsForRows(vikkiCustomers, { topic: Topics.vikki, source: "/vikki/t24", type: "com.vikki.t24.customer.snapshot", keyField: "customer_id" }),
    ...eventsForRows(vikkiAccounts, { topic: Topics.vikki, source: "/vikki/t24", type: "com.vikki.t24.account.snapshot", keyField: "account_id" }),
    ...eventsForRows(vikkiTransactions, { topic: Topics.vikki, source: "/vikki/t24", type: "com.vikki.t24.wallet.transaction.completed", keyField: "transaction_id" }),
    ...eventsForRows(hdbankCustomers, { topic: Topics.hdbank, source: "/hdbank/t24", type: "com.hdbank.t24.customer.snapshot", keyField: "customer_id" }),
    ...eventsForRows(hdbankLoans, { topic: Topics.hdbank, source: "/hdbank/t24", type: "com.hdbank.t24.loan.snapshot", keyField: "loan_id" }),
    ...eventsForRows(hdbankRepayments, { topic: Topics.hdbank, source: "/hdbank/t24", type: "com.hdbank.t24.loan.repayment.completed", keyField: "repayment_id" }),
    ...eventsForRows(crmContacts, { topic: Topics.crm, source: "/crm", type: "com.crm.contact.snapshot", keyField: "contact_id" }),
    ...eventsForRows(crmInteractions, { topic: Topics.crm, source: "/crm", type: "com.crm.campaign.interaction.recorded", keyField: "interaction_id" }),
    ...eventsForRows(vietjetPassengers, { topic: Topics.vietjet, source: "/vietjet", type: "com.vietjet.passenger.snapshot", keyField: "passenger_id" }),
    ...eventsForRows(vietjetBookings, { topic: Topics.vietjet, source: "/vietjet", type: "com.vietjet.booking.recorded", keyField: "booking_id" }),
  ]
}
