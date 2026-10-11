export type StepStatus = "running" | "completed" | "error"

export type PiEvent = Record<string, unknown>

export type UiStep = {
  id: string
  title: string
  status: StepStatus
  detail?: string
  meta?: string
  streamKey?: string
  runIndex?: number
  event: PiEvent
}

function object(value: unknown): Record<string, unknown> | undefined {
  return typeof value === "object" && value !== null
    ? (value as Record<string, unknown>)
    : undefined
}

function string(value: unknown) {
  return typeof value === "string" ? value : undefined
}

function number(value: unknown) {
  return typeof value === "number" ? value : undefined
}

function messageText(messageValue: unknown) {
  const message = object(messageValue)
  if (!message) return undefined
  if (typeof message.content === "string") return message.content
  if (!Array.isArray(message.content)) return undefined

  const text = message.content
    .map((item) => {
      const content = object(item)
      return content?.type === "text" ? string(content.text) : undefined
    })
    .filter((item): item is string => item !== undefined)
    .join("\n")

  return text || undefined
}

function truncate(value: unknown, length = 500) {
  let text: string
  if (typeof value === "string") {
    text = value
  } else {
    try {
      text = JSON.stringify(value, null, 2) ?? String(value)
    } catch {
      text = String(value)
    }
  }
  return text.length > length ? `${text.slice(0, length)}…` : text
}

function assistantMeta(message: Record<string, unknown>) {
  const model = string(message.model)
  const usage = object(message.usage)
  const totalTokens = number(usage?.totalTokens)
  const durationMs = number(message.durationMs)
  return [
    model,
    totalTokens === undefined ? undefined : `${totalTokens} tokens`,
    durationMs === undefined ? undefined : `${durationMs} ms`,
  ]
    .filter(Boolean)
    .join(" · ")
}

function titleCase(value: string) {
  return value
    .split("_")
    .map((word) => `${word.charAt(0).toUpperCase()}${word.slice(1)}`)
    .join(" ")
}

function previousStreamText(steps: UiStep[], streamKey: string) {
  for (let index = steps.length - 1; index >= 0; index -= 1) {
    const step = steps[index]
    if (step.streamKey === streamKey) return step.detail ?? ""
    if (step.event.type === "run_start") break
  }
  return ""
}

function describeEvent(
  steps: UiStep[],
  event: PiEvent,
): Omit<UiStep, "id" | "event"> {
  const type = string(event.type) ?? "unknown"

  if (type === "run_start") {
    const agent = string(event.agent)
    return {
      title: agent ? `Run started for ${agent}` : "Run started",
      status: "running",
    }
  }

  if (type === "session") {
    return {
      title: "Session started",
      status: "completed",
      detail: string(event.cwd),
      meta: string(event.id),
    }
  }

  if (type === "agent_start")
    return { title: "Agent started", status: "running" }
  if (type === "turn_start") return { title: "Turn started", status: "running" }

  if (type === "message_start" || type === "message_end") {
    const message = object(event.message)
    const role = string(message?.role) ?? "unknown"
    const status = type === "message_end" ? "completed" : "running"

    if (role === "system") {
      return {
        title: `System message ${type === "message_end" ? "ended" : "started"}`,
        status,
      }
    }
    if (role === "user") {
      return { title: "User message", status, detail: messageText(message) }
    }
    if (role === "assistant") {
      const stopReason = string(message?.stopReason)
      return {
        title: "Assistant message",
        status:
          stopReason === "error" || stopReason === "aborted" ? "error" : status,
        detail: messageText(message),
        meta: message ? assistantMeta(message) || undefined : undefined,
      }
    }

    return { title: `${titleCase(role)} message`, status }
  }

  if (type === "message_update") {
    const messageEvent = object(event.assistantMessageEvent)
    const updateType = string(messageEvent?.type) ?? "update"
    const contentIndex = number(messageEvent?.contentIndex) ?? 0

    if (updateType.startsWith("text_")) {
      const streamKey = `text-${contentIndex}`
      const delta = string(messageEvent?.delta)
      const content = string(messageEvent?.content)
      const detail =
        delta === undefined
          ? content
          : `${previousStreamText(steps, streamKey)}${delta}`
      return {
        title:
          updateType === "text_start"
            ? "Assistant text started"
            : updateType === "text_end"
              ? "Assistant text completed"
              : "Assistant text updated",
        status: updateType === "text_end" ? "completed" : "running",
        detail,
        streamKey,
      }
    }

    if (updateType.startsWith("thinking_")) {
      return {
        title:
          updateType === "thinking_end" ? "Thinking completed" : "Thinking",
        status: updateType === "thinking_end" ? "completed" : "running",
      }
    }

    if (updateType.startsWith("toolcall_")) {
      const call =
        object(messageEvent?.toolCall) ?? object(messageEvent?.partialCall)
      const name = string(call?.name) ?? "tool"
      return {
        title: `Preparing ${name}`,
        status: updateType === "toolcall_end" ? "completed" : "running",
        detail:
          call?.arguments === undefined ? undefined : truncate(call.arguments),
      }
    }

    return { title: titleCase(updateType), status: "running" }
  }

  if (type === "tool_execution_start" || type === "tool_execution_update") {
    const name = string(event.toolName) ?? "tool"
    return {
      title:
        type === "tool_execution_start" ? `Using ${name}` : `${name} updated`,
      status: "running",
      detail:
        event.args === undefined
          ? event.partialResult === undefined
            ? undefined
            : truncate(event.partialResult)
          : truncate(event.args),
    }
  }

  if (type === "tool_execution_end") {
    const name = string(event.toolName) ?? "tool"
    const isError = event.isError === true
    return {
      title: `${name} ${isError ? "failed" : "finished"}`,
      status: isError ? "error" : "completed",
      detail: event.result === undefined ? undefined : truncate(event.result),
      meta:
        typeof event.durationMs === "number"
          ? `${event.durationMs} ms`
          : undefined,
    }
  }

  if (type === "turn_end")
    return { title: "Turn completed", status: "completed" }
  if (type === "agent_end") {
    return {
      title: event.willRetry === true ? "Agent will retry" : "Agent finished",
      status: event.willRetry === true ? "running" : "completed",
    }
  }

  if (type === "auto_retry_start") {
    const attempt = number(event.attempt)
    const maxAttempts = number(event.maxAttempts)
    return {
      title: "Retry started",
      status: "running",
      detail: string(event.error),
      meta:
        attempt === undefined
          ? undefined
          : `Attempt ${attempt}${maxAttempts ? ` of ${maxAttempts}` : ""}`,
    }
  }

  if (type === "auto_retry_end") {
    return {
      title: "Retry finished",
      status: event.success === false ? "error" : "completed",
    }
  }

  if (type === "agent_settled") {
    const aborted = event.aborted === true
    return {
      title: aborted ? "Agent stopped" : "Agent settled",
      status: aborted ? "error" : "completed",
    }
  }

  if (type === "process_exit") {
    const code = number(event.code)
    return {
      title: code === 0 ? "Process exited" : "Process failed",
      status: code === 0 ? "completed" : "error",
      meta: code === undefined ? undefined : `Exit code ${code}`,
    }
  }

  if (type === "stderr" || type === "process_error" || type === "unparsed") {
    return {
      title: "Run error",
      status: "error",
      detail: string(event.message) ?? truncate(event),
    }
  }

  return { title: titleCase(type), status: "completed" }
}

function appendDebugEvent(steps: UiStep[], event: PiEvent): UiStep[] {
  const type = string(event.type) ?? "event"
  return [
    ...steps,
    {
      id: `${type}-${steps.length}`,
      ...describeEvent(steps, event),
      event,
    },
  ]
}

function upsert(steps: UiStep[], step: UiStep) {
  const index = steps.findIndex((current) => current.id === step.id)
  if (index === -1) return [...steps, step]

  const next = [...steps]
  next[index] = { ...next[index], ...step }
  return next
}

function currentRunIndex(steps: UiStep[], startsNewRun: boolean) {
  const latestRun = steps.reduce(
    (latest, step) => Math.max(latest, step.runIndex ?? -1),
    -1,
  )
  return startsNewRun ? latestRun + 1 : Math.max(0, latestRun)
}

function reduceFriendlyEvent(steps: UiStep[], event: PiEvent): UiStep[] {
  const type = string(event.type) ?? "event"
  const runIndex = currentRunIndex(steps, type === "run_start")
  const id = (key: string) => `run-${runIndex}-${key}`
  const described = describeEvent(steps, event)
  const step = (key: string, changes: Partial<UiStep> = {}) =>
    upsert(steps, {
      id: id(key),
      ...described,
      ...changes,
      runIndex,
      event,
    })

  if (type === "run_start") {
    const agent = string(event.agent)
    return step("run", {
      title: agent ? `Running ${agent}` : "Run started",
    })
  }
  if (type === "session") return step("session")
  if (type === "agent_start") return step("agent", { title: "Agent working" })
  if (type === "agent_end" || type === "agent_settled") return step("agent")
  if (type === "turn_start") return step("turn", { title: "Processing prompt" })
  if (type === "turn_end") return step("turn")

  if (type === "message_start" || type === "message_end") {
    const message = object(event.message)
    const role = string(message?.role)
    if (role === "system") return steps
    if (role === "user") return step("prompt", { title: "Prompt" })
    if (role === "assistant")
      return step("response-0", { title: "Assistant response" })
  }

  if (type === "message_update") {
    const messageEvent = object(event.assistantMessageEvent)
    const updateType = string(messageEvent?.type) ?? "update"
    const contentIndex = number(messageEvent?.contentIndex) ?? 0
    if (updateType.startsWith("text_")) {
      return step(`response-${contentIndex}`, { title: "Assistant response" })
    }
    if (updateType.startsWith("thinking_")) {
      return step(`thinking-${contentIndex}`)
    }
    if (updateType.startsWith("toolcall_")) {
      const call =
        object(messageEvent?.toolCall) ?? object(messageEvent?.partialCall)
      return step(`tool-${string(call?.id) ?? contentIndex}`)
    }
  }

  if (type.startsWith("tool_execution_")) {
    return step(`tool-${string(event.toolCallId) ?? steps.length}`)
  }
  if (type.startsWith("auto_retry_")) return step("retry")
  if (type === "process_exit") {
    const code = number(event.code)
    return step("run", {
      title: code === 0 ? "Run completed" : "Run failed",
    })
  }

  return step(`${type}-${steps.length}`)
}

export function buildPiSteps(events: PiEvent[], debug: boolean) {
  return events.reduce(debug ? appendDebugEvent : reduceFriendlyEvent, [])
}
