import type { VietnamesePerson } from "../schemas/common.js"
import type { RegionGeneratorEngine } from "./types.js"
import { ascii, digits, integer, pick } from "./utils.js"

const familyNames = ["Nguyễn", "Trần", "Lê", "Phạm", "Hoàng", "Huỳnh", "Phan", "Vũ", "Võ", "Đặng"]
const middleNames = ["Minh", "Ngọc", "Thanh", "Quốc", "Thị", "Văn", "Hoài", "Gia"]
const femaleGivenNames = ["Anh", "Linh", "Trang", "Hương", "Thảo", "Mai", "Vy", "Ngân"]
const maleGivenNames = ["Anh", "Hùng", "Nam", "Dũng", "Khang", "Phúc", "Long", "Quân"]
const phonePrefixes = ["90", "91", "93", "94", "96", "97", "98"]
const locations = [
  {
    addressLine1: "12 Đường Nguyễn Huệ",
    ward: "Phường Bến Nghé",
    district: "Quận 1",
    province: "Thành phố Hồ Chí Minh",
    provinceCode: "079",
    airport: "SGN",
  },
  {
    addressLine1: "28 Phố Lý Thường Kiệt",
    ward: "Phường Trần Hưng Đạo",
    district: "Quận Hoàn Kiếm",
    province: "Hà Nội",
    provinceCode: "001",
    airport: "HAN",
  },
  {
    addressLine1: "105 Đường Bạch Đằng",
    ward: "Phường Hải Châu",
    district: "Quận Hải Châu",
    province: "Đà Nẵng",
    provinceCode: "048",
    airport: "DAD",
  },
] as const

export const vietnamGenerator: RegionGeneratorEngine = {
  region: "vn",
  countryCode: "VN",
  locale: "vi-VN",
  currency: "VND",
  timezone: "Asia/Ho_Chi_Minh",
  legalDocumentName: "CCCD",
  destinations: ["DAD", "CXR", "PQC"],
  financials: {
    accountBalance: [5_000_000, 300_000_000],
    walletPayment: [75_000, 1_500_000],
    loanUnit: 10_000_000,
    booking: [900_000, 2_500_000],
    monthlyInterest: 900_000,
  },
  createPerson(random, sequence, seed): VietnamesePerson {
    const gender = random() < 0.5 ? "FEMALE" : "MALE"
    const familyName = pick(random, familyNames)
    const middleName = pick(random, middleNames)
    const givenName = pick(random, gender === "FEMALE" ? femaleGivenNames : maleGivenNames)
    const fullName = `${familyName} ${middleName} ${givenName}`
    const birthYear = integer(random, 1970, 2000)
    const birthMonth = integer(random, 1, 12)
    const birthDay = integer(random, 1, 28)
    const dateOfBirth = `${birthYear}-${String(birthMonth).padStart(2, "0")}-${String(birthDay).padStart(2, "0")}`
    const location = pick(random, locations)
    const genderCenturyCode = gender === "FEMALE" ? "1" : "0"
    const nationalId = `${location.provinceCode}${genderCenturyCode}${String(birthYear).slice(-2)}${digits(random, 6)}`
    const phoneNumber = `+84${pick(random, phonePrefixes)}${digits(random, 7)}`
    const personSuffix = `vn-${seed}-${String(sequence).padStart(4, "0")}`
    const emailName = ascii(fullName).toLowerCase().replaceAll(" ", ".")

    return {
      personId: `PER-${personSuffix}`,
      fullName,
      fullNameAscii: ascii(fullName),
      familyName,
      middleName,
      givenName,
      dateOfBirth,
      gender,
      nationalId,
      phoneNumber,
      emailAddress: `${emailName}.${seed}.${sequence}@example.vn`,
      nationalityCode: "VN",
      residenceCountryCode: "VN",
      preferredLanguage: "vi-VN",
      address: {
        addressLine1: location.addressLine1,
        ward: location.ward,
        district: location.district,
        province: location.province,
        countryCode: "VN",
      },
    }
  },
  airportForProvince(province) {
    return locations.find((location) => location.province === province)?.airport ?? "SGN"
  },
  walletDescription(sequence) {
    return `Thanh toán ví Vikki ${sequence}`
  },
  repaymentDescription(sequence) {
    return `Kỳ trả nợ ${sequence}`
  },
  ticketDescription(bookingReference) {
    return `Thanh toán vé ${bookingReference}`
  },
}
