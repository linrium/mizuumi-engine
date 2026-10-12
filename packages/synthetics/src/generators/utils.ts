import type { Random } from "./types.js"

export function pick<T>(random: Random, values: readonly T[]): T {
  return values[Math.floor(random() * values.length)] as T
}

export function integer(random: Random, minimum: number, maximum: number) {
  return Math.floor(random() * (maximum - minimum + 1)) + minimum
}

export function decimal(random: Random, minimum: number, maximum: number) {
  return minimum + random() * (maximum - minimum)
}

export function chance(random: Random, probability: number) {
  return random() < Math.max(0, Math.min(1, probability))
}

export function poisson(random: Random, lambda: number) {
  const threshold = Math.exp(-lambda)
  let product = 1
  let count = 0
  do {
    count += 1
    product *= random()
  } while (product > threshold)
  return count - 1
}

export function weightedPick<T>(
  random: Random,
  values: readonly { value: T; weight: number }[],
): T {
  const total = values.reduce((sum, item) => sum + Math.max(0, item.weight), 0)
  if (total <= 0) throw new RangeError("weightedPick requires a positive total weight")
  let cursor = random() * total
  for (const item of values) {
    cursor -= Math.max(0, item.weight)
    if (cursor <= 0) return item.value
  }
  return values[values.length - 1]!.value
}

export function digits(random: Random, length: number) {
  return Array.from({ length }, () => integer(random, 0, 9)).join("")
}

export function ascii(value: string) {
  return value
    .replaceAll("Đ", "D")
    .replaceAll("đ", "d")
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
}
