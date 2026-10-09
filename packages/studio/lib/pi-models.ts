export const PI_THINKING_LEVELS = [
  "off",
  "low",
  "medium",
  "high",
  "xhigh",
  "max",
] as const

export type PiThinkingLevel = (typeof PI_THINKING_LEVELS)[number]

type PiModelDefinition = {
  id: string
  label: string
  thinkingLevels: readonly PiThinkingLevel[]
}

const GPT_55_THINKING_LEVELS = [
  "off",
  "low",
  "medium",
  "high",
  "xhigh",
] as const satisfies readonly PiThinkingLevel[]

const GPT_56_THINKING_LEVELS = PI_THINKING_LEVELS

export const PI_MODELS = [
  {
    id: "gpt-5.5",
    label: "GPT-5.5",
    thinkingLevels: GPT_55_THINKING_LEVELS,
  },
  {
    id: "gpt-5.6-sol",
    label: "GPT-5.6 Sol",
    thinkingLevels: GPT_56_THINKING_LEVELS,
  },
  {
    id: "gpt-5.6-terra",
    label: "GPT-5.6 Terra",
    thinkingLevels: GPT_56_THINKING_LEVELS,
  },
  {
    id: "gpt-5.6-luna",
    label: "GPT-5.6 Luna",
    thinkingLevels: GPT_56_THINKING_LEVELS,
  },
] as const satisfies readonly PiModelDefinition[]

export type PiModel = (typeof PI_MODELS)[number]["id"]

export const DEFAULT_PI_MODEL: PiModel = "gpt-5.5"
export const DEFAULT_PI_THINKING_LEVEL: PiThinkingLevel = "off"

export function isPiModel(value: unknown): value is PiModel {
  return PI_MODELS.some((model) => model.id === value)
}

export function isPiThinkingLevel(value: unknown): value is PiThinkingLevel {
  return PI_THINKING_LEVELS.some((level) => level === value)
}

export function getPiThinkingLevels(model: PiModel) {
  return (
    PI_MODELS.find((candidate) => candidate.id === model)?.thinkingLevels ??
    GPT_56_THINKING_LEVELS
  )
}

export function supportsPiThinkingLevel(
  model: PiModel,
  thinkingLevel: PiThinkingLevel,
) {
  return getPiThinkingLevels(model).some((level) => level === thinkingLevel)
}

export const PI_MODELS_JSON = JSON.stringify(
  PI_MODELS.map(({ id, label, thinkingLevels }) => ({
    id,
    name: label,
    reasoning: true,
    input: ["text", "image"],
    contextWindow: 1_050_000,
    maxTokens: 128_000,
    thinkingLevelMap: Object.fromEntries(
      PI_THINKING_LEVELS.map((level) => [
        level,
        thinkingLevels.some((supported) => supported === level)
          ? level === "off"
            ? "none"
            : level
          : null,
      ]),
    ),
    compat: { supportsReasoningEffort: true },
  })),
)
