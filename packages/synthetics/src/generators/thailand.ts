import type { ThaiPerson } from "../schemas/common.js"
import type { RegionGeneratorEngine } from "./types.js"
import { digits, integer, pick } from "./utils.js"

const familyNames: readonly (readonly [string, string])[] = [
  ["ศรีสุข", "Srisuk"],
  ["วงศ์สวัสดิ์", "Wongsawat"],
  ["ตั้งใจ", "Tangjai"],
  ["บุญมี", "Boonmee"],
  ["แก้วกาญจน์", "Kaewkan"],
  ["ชัยวัฒน์", "Chaiwat"],
] as const
const femaleGivenNames: readonly (readonly [string, string])[] = [
  ["กัญญา", "Kanya"],
  ["พิมพ์ชนก", "Pimchanok"],
  ["ศิริพร", "Siriporn"],
  ["ณิชา", "Nicha"],
  ["วรัญญา", "Waranya"],
] as const
const maleGivenNames: readonly (readonly [string, string])[] = [
  ["อนันต์", "Anan"],
  ["ธนกร", "Thanakorn"],
  ["ณัฐวุฒิ", "Nattawut"],
  ["กิตติ", "Kitti"],
  ["ชัยพร", "Chaiporn"],
] as const
const phonePrefixes = ["6", "8", "9"]
const locations = [
  {
    addressLine1: "88 ถนนสุขุมวิท",
    ward: "แขวงคลองเตยเหนือ",
    district: "เขตวัฒนา",
    province: "กรุงเทพมหานคร",
    postalCode: "10110",
    airport: "BKK",
  },
  {
    addressLine1: "45 ถนนนิมมานเหมินท์",
    ward: "ตำบลสุเทพ",
    district: "อำเภอเมืองเชียงใหม่",
    province: "เชียงใหม่",
    postalCode: "50200",
    airport: "CNX",
  },
  {
    addressLine1: "21 ถนนเทพกระษัตรี",
    ward: "ตำบลตลาดใหญ่",
    district: "อำเภอเมืองภูเก็ต",
    province: "ภูเก็ต",
    postalCode: "83000",
    airport: "HKT",
  },
] as const

function thaiNationalId(random: () => number) {
  const firstTwelve = `${integer(random, 1, 8)}${digits(random, 11)}`
  const weightedSum = [...firstTwelve].reduce(
    (sum, digit, index) => sum + Number(digit) * (13 - index),
    0,
  )
  const checkDigit = (11 - (weightedSum % 11)) % 10
  return `${firstTwelve}${checkDigit}`
}

export const thailandGenerator: RegionGeneratorEngine = {
  region: "th",
  countryCode: "TH",
  locale: "th-TH",
  currency: "THB",
  timezone: "Asia/Bangkok",
  legalDocumentName: "THAI_NATIONAL_ID",
  destinations: ["CNX", "HKT", "KBV"],
  financials: {
    accountBalance: [10_000, 500_000],
    walletPayment: [100, 5_000],
    loanUnit: 25_000,
    booking: [1_200, 4_500],
    monthlyInterest: 750,
  },
  createPerson(random, sequence, seed): ThaiPerson {
    const gender = random() < 0.5 ? "FEMALE" : "MALE"
    const [familyName, familyNameAscii] = pick(random, familyNames)
    const [givenName, givenNameAscii] = pick(
      random,
      gender === "FEMALE" ? femaleGivenNames : maleGivenNames,
    )
    const fullName = `${givenName} ${familyName}`
    const fullNameAscii = `${givenNameAscii} ${familyNameAscii}`
    const birthYear = integer(random, 1970, 2000)
    const birthMonth = integer(random, 1, 12)
    const birthDay = integer(random, 1, 28)
    const dateOfBirth = `${birthYear}-${String(birthMonth).padStart(2, "0")}-${String(birthDay).padStart(2, "0")}`
    const location = pick(random, locations)

    return {
      personId: `PER-th-${seed}-${String(sequence).padStart(4, "0")}`,
      fullName,
      fullNameAscii,
      familyName,
      givenName,
      dateOfBirth,
      gender,
      nationalId: thaiNationalId(random),
      phoneNumber: `+66${pick(random, phonePrefixes)}${digits(random, 8)}`,
      emailAddress: `${givenNameAscii.toLowerCase()}.${familyNameAscii.toLowerCase()}.${seed}.${sequence}@example.co.th`,
      nationalityCode: "TH",
      residenceCountryCode: "TH",
      preferredLanguage: "th-TH",
      address: {
        addressLine1: location.addressLine1,
        ward: location.ward,
        district: location.district,
        province: location.province,
        postalCode: location.postalCode,
        countryCode: "TH",
      },
    }
  },
  airportForProvince(province) {
    return locations.find((location) => location.province === province)?.airport ?? "BKK"
  },
  walletDescription(sequence) {
    return `ชำระเงินผ่านกระเป๋า Vikki ${sequence}`
  },
  repaymentDescription(sequence) {
    return `ชำระคืนเงินกู้งวดที่ ${sequence}`
  },
  ticketDescription(bookingReference) {
    return `ชำระค่าตั๋ว ${bookingReference}`
  },
}
