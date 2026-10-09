import { db } from "./client.js";
import { runMigrations } from "./migrate.js";
import { accounts, customers, dataRights, products } from "./schema.js";

const CUSTOMERS = [
  {
    id: "CUST-1001",
    firstName: "Lan",
    lastName: "Nguyễn Thị",
    email: "lan.nguyen@example.vn",
    phone: "+84-90-123-0101",
    dateOfBirth: "1988-03-14",
    nationalId: "CCCD-***-***-1234",
    address: "100 Đường Nguyễn Huệ, Phường Bến Nghé",
    city: "Hồ Chí Minh",
    country: "VN",
    segment: "premium",
    kycStatus: "verified",
    riskRating: "low",
  },
  {
    id: "CUST-1002",
    firstName: "Minh",
    lastName: "Trần Văn",
    email: "minh.tran@example.vn",
    phone: "+84-91-234-0102",
    dateOfBirth: "1975-11-02",
    nationalId: "CCCD-***-***-5678",
    address: "22 Phố Lý Thường Kiệt, Quận Hoàn Kiếm",
    city: "Hà Nội",
    country: "VN",
    segment: "retail",
    kycStatus: "verified",
    riskRating: "medium",
  },
];

const ACCOUNTS = [
  {
    id: "ACC-2001",
    customerId: "CUST-1001",
    accountNumber: "VN-0001-2001",
    type: "thanh_toán",
    currency: "VND",
    balance: 285_000_000,
    availableBalance: 285_000_000,
  },
  {
    id: "ACC-2002",
    customerId: "CUST-1001",
    accountNumber: "VN-0001-2002",
    type: "tiết_kiệm",
    currency: "VND",
    balance: 1_245_000_000,
    availableBalance: 1_245_000_000,
  },
  {
    id: "ACC-2003",
    customerId: "CUST-1002",
    accountNumber: "VN-0002-2003",
    type: "thanh_toán",
    currency: "VND",
    balance: 22_500_000,
    availableBalance: 22_500_000,
  },
];

const PRODUCTS = [
  {
    id: "PROD-3001",
    customerId: "CUST-1001",
    productCode: "CC-GOLD",
    productName: "Thẻ tín dụng Hạng Vàng",
    category: "credit_card",
  },
  {
    id: "PROD-3002",
    customerId: "CUST-1001",
    productCode: "MORTGAGE-30",
    productName: "Vay thế chấp mua nhà 30 năm",
    category: "mortgage",
  },
  {
    id: "PROD-3003",
    customerId: "CUST-1002",
    productCode: "CC-STANDARD",
    productName: "Thẻ tín dụng Tiêu Chuẩn",
    category: "credit_card",
  },
];

export function seedDatabase() {
  db.insert(customers).values(CUSTOMERS).onConflictDoNothing().run();
  db.insert(dataRights)
    .values(
      CUSTOMERS.map((c) => ({
        customerId: c.id,
        canAccessData: true,
        canExportData: true,
        canDeleteData: false,
        marketingOptIn: c.segment === "premium",
        thirdPartySharingOptIn: false,
      })),
    )
    .onConflictDoNothing()
    .run();
  db.insert(accounts).values(ACCOUNTS).onConflictDoNothing().run();
  db.insert(products).values(PRODUCTS).onConflictDoNothing().run();
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runMigrations();
  seedDatabase();
  console.log("Seeded Temenos simulator database (vi-VN fixtures).");
}
