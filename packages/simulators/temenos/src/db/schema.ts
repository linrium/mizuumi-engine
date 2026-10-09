import { sql } from "drizzle-orm";
import { integer, real, sqliteTable, text } from "drizzle-orm/sqlite-core";

export const customers = sqliteTable("customers", {
  id: text("id").primaryKey(),
  firstName: text("first_name").notNull(),
  lastName: text("last_name").notNull(),
  email: text("email").notNull(),
  phone: text("phone"),
  dateOfBirth: text("date_of_birth"),
  nationalId: text("national_id"),
  address: text("address"),
  city: text("city"),
  country: text("country").default("US"),
  segment: text("segment").default("retail"),
  kycStatus: text("kyc_status").default("verified"),
  riskRating: text("risk_rating").default("low"),
  createdAt: text("created_at").default(sql`CURRENT_TIMESTAMP`),
});

export const dataRights = sqliteTable("data_rights", {
  customerId: text("customer_id")
    .primaryKey()
    .references(() => customers.id),
  canAccessData: integer("can_access_data", { mode: "boolean" }).default(true),
  canExportData: integer("can_export_data", { mode: "boolean" }).default(true),
  canDeleteData: integer("can_delete_data", { mode: "boolean" }).default(false),
  marketingOptIn: integer("marketing_opt_in", { mode: "boolean" }).default(false),
  thirdPartySharingOptIn: integer("third_party_sharing_opt_in", { mode: "boolean" }).default(false),
  consentVersion: text("consent_version").default("v1.0"),
  consentUpdatedAt: text("consent_updated_at").default(sql`CURRENT_TIMESTAMP`),
});

export const accounts = sqliteTable("accounts", {
  id: text("id").primaryKey(),
  customerId: text("customer_id")
    .notNull()
    .references(() => customers.id),
  accountNumber: text("account_number").notNull(),
  type: text("type").notNull(),
  currency: text("currency").notNull().default("USD"),
  balance: real("balance").notNull().default(0),
  availableBalance: real("available_balance").notNull().default(0),
  status: text("status").notNull().default("active"),
  openedAt: text("opened_at").default(sql`CURRENT_TIMESTAMP`),
});

export const products = sqliteTable("products", {
  id: text("id").primaryKey(),
  customerId: text("customer_id")
    .notNull()
    .references(() => customers.id),
  productCode: text("product_code").notNull(),
  productName: text("product_name").notNull(),
  category: text("category").notNull(),
  status: text("status").notNull().default("active"),
  enrolledAt: text("enrolled_at").default(sql`CURRENT_TIMESTAMP`),
});

export const transfers = sqliteTable("transfers", {
  id: text("id").primaryKey(),
  fromAccountId: text("from_account_id").references(() => accounts.id),
  toAccountId: text("to_account_id").references(() => accounts.id),
  amount: real("amount").notNull(),
  currency: text("currency").notNull().default("USD"),
  type: text("type").notNull().default("transfer"),
  reference: text("reference"),
  description: text("description"),
  status: text("status").notNull().default("pending"),
  createdAt: text("created_at").default(sql`CURRENT_TIMESTAMP`),
  completedAt: text("completed_at"),
});
