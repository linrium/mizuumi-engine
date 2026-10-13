#!/usr/bin/env node

import path from "node:path"
import { generateDataset, type GenerateOptions } from "./generator.js"

const HELP = `Mizuumi synthetic customer data generator

Usage:
  npm run generate -- [options]

Options:
  --output <directory>    Destination (default: ./data/demo-customer)
  --customers <number>   Total synthetic population (default: 250000)
  --seed <number>        Deterministic random seed (default: 20261013)
  --as-of <YYYY-MM-DD>   Dataset reference date (default: 2026-10-13)
  --featured-only        Generate only Trần Minh Anh's linked history
  --force                Replace files already in the output directory
  --help                  Show this help
`

function valueAfter(args: string[], index: number, flag: string): string {
  const value = args[index + 1]
  if (!value || value.startsWith("--")) throw new Error(`${flag} requires a value`)
  return value
}

export function parseArgs(args: string[]): GenerateOptions | "help" {
  const options: GenerateOptions = {
    outputDirectory: path.resolve("data/demo-customer"),
    customerCount: 250_000,
    seed: 20_261_013,
    asOf: "2026-10-13",
    force: false,
  }
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index]
    if (argument === "--help" || argument === "-h") return "help"
    if (argument === "--force") options.force = true
    else if (argument === "--featured-only") options.customerCount = 1
    else if (argument === "--output") options.outputDirectory = path.resolve(valueAfter(args, index++, argument))
    else if (argument === "--customers") options.customerCount = Number(valueAfter(args, index++, argument))
    else if (argument === "--seed") options.seed = Number(valueAfter(args, index++, argument))
    else if (argument === "--as-of") options.asOf = valueAfter(args, index++, argument)
    else throw new Error(`Unknown option: ${argument}`)
  }
  return options
}

async function main(): Promise<void> {
  const options = parseArgs(process.argv.slice(2))
  if (options === "help") {
    process.stdout.write(HELP)
    return
  }
  process.stdout.write(`Generating ${options.customerCount.toLocaleString("en-US")} synthetic people in ${options.outputDirectory}\n`)
  const counts = await generateDataset(options)
  const totalRows = Object.values(counts).reduce((sum, count) => sum + count, 0)
  process.stdout.write(`Generated ${totalRows.toLocaleString("en-US")} rows across ${Object.keys(counts).length} CSV files.\n`)
}

main().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error)
  process.stderr.write(`Error: ${message}\n`)
  process.exitCode = 1
})
