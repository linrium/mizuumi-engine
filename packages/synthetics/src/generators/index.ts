import { thailandGenerator } from "./thailand.js"
import type { RegionCode, RegionGeneratorEngine } from "./types.js"
import { vietnamGenerator } from "./vietnam.js"

const generators: Record<RegionCode, RegionGeneratorEngine> = {
  vn: vietnamGenerator,
  th: thailandGenerator,
}

export function getRegionGenerator(region: RegionCode) {
  return generators[region]
}

export { thailandGenerator, vietnamGenerator }
export type { RegionCode, RegionGeneratorEngine } from "./types.js"
