import type { Random } from "./types.js"

export function pick<T>(random: Random, values: readonly T[]): T {
  return values[Math.floor(random() * values.length)] as T
}

export function integer(random: Random, minimum: number, maximum: number) {
  return Math.floor(random() * (maximum - minimum + 1)) + minimum
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
