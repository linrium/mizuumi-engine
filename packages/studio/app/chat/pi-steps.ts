export type StepStatus = "running" | "completed" | "error"

export type UiStep = {
  id: string
  title: string
  status: StepStatus
  detail?: string
  meta?: string
}

type PiEvent = Record<string, unknown>

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
      text = JSON.stringify(value, null, 2)
    } catch {
      text = String(value)
    }
  }

  return text.length > length ? `${text.slice(0, length)}…` : text
}

function upsert(steps: UiStep[], step: UiStep) {
  const index = steps.findIndex((current) => current.id === step.id)
  if (index === -1) return [...steps, step]

  const next = [...steps]
  next[index] = { ...next[index], ...step }
  return next
}

function update(
  steps: UiStep[],
  id: string,
  changes: Partial<Omit<UiStep, "id">>,
) {
  const current = steps.find((step) => step.id === id)
  if (!current) return steps
  return upsert(steps, { ...current, ...changes })
}

function completeLifecycle(steps: UiStep[], aborted: boolean) {
  const status = aborted ? "error" : "completed"
  let next = update(steps, "turn", { status })
  next = update(next, "agent", {
    status,
    title: aborted ? "Agent stopped" : "Agent finished",
  })
  return update(next, "run", {
    status,
    title: aborted ? "Run stopped" : "Run completed",
  })
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

function finishAssistantMessage(
  steps: UiStep[],
  message: Record<string, unknown>,
) {
  const stopReason = string(message.stopReason)
  const status =
    stopReason === "error" || stopReason === "aborted" ? "error" : "completed"
  const meta = assistantMeta(message) || undefined
  let next = steps

  if (typeof message.content === "string") {
    return upsert(next, {
      id: "response-0",
      title: "Assistant response",
      status,
      detail: message.content,
      meta,
    })
  }

  if (Array.isArray(message.content)) {
    message.content.forEach((item, index) => {
      const content = object(item)
      if (content?.type !== "text") return
      next = upsert(next, {
        id: `response-${index}`,
        title: "Assistant response",
        status,
        detail: string(content.text),
        meta,
      })
    })
  }

  return update(next, "response-0", { status, meta })
}

function reduceMessageUpdate(steps: UiStep[], event: PiEvent) {
  const messageEvent = object(event.assistantMessageEvent)
  const eventType = string(messageEvent?.type)
  const contentIndex = number(messageEvent?.contentIndex) ?? 0

  if (eventType === "text_start") {
    return upsert(steps, {
      id: `response-${contentIndex}`,
      title: "Assistant response",
      status: "running",
    })
  }

  if (eventType === "text_delta") {
    const id = `response-${contentIndex}`
    const current = steps.find((step) => step.id === id)
    return upsert(steps, {
      id,
      title: "Assistant response",
      status: "running",
      detail: `${current?.detail ?? ""}${string(messageEvent?.delta) ?? ""}`,
    })
  }

  if (eventType === "text_end") {
    return upsert(steps, {
      id: `response-${contentIndex}`,
      title: "Assistant response",
      status: "completed",
      detail: string(messageEvent?.content),
    })
  }

  if (eventType?.startsWith("thinking_")) {
    return upsert(steps, {
      id: `thinking-${contentIndex}`,
      title: "Thinking",
      status: eventType === "thinking_end" ? "completed" : "running",
    })
  }

  if (eventType?.startsWith("toolcall_")) {
    const call =
      object(messageEvent?.toolCall) ?? object(messageEvent?.partialCall)
    const id = string(call?.id) ?? `pending-${contentIndex}`
    const name = string(call?.name) ?? "tool"
    return upsert(steps, {
      id: `tool-${id}`,
      title: `Preparing ${name}`,
      status: eventType === "toolcall_end" ? "completed" : "running",
      detail:
        call?.arguments === undefined ? undefined : truncate(call.arguments),
    })
  }

  return steps
}

export function reducePiEvent(steps: UiStep[], event: PiEvent): UiStep[] {
  const type = string(event.type)

  if (type === "run_start") {
    const agent = string(event.agent)
    return upsert(steps, {
      id: "run",
      title: agent ? `Running ${agent}` : "Run started",
      status: "running",
    })
  }

  if (type === "session") {
    return upsert(steps, {
      id: "session",
      title: "Session started",
      status: "completed",
      detail: string(event.cwd),
      meta: string(event.id),
    })
  }

  if (type === "agent_start") {
    return upsert(steps, {
      id: "agent",
      title: "Agent working",
      status: "running",
    })
  }

  if (type === "turn_start") {
    return upsert(steps, {
      id: "turn",
      title: "Processing prompt",
      status: "running",
    })
  }

  if (type === "message_start" || type === "message_end") {
    const message = object(event.message)
    const role = string(message?.role)
    if (role === "system") return steps

    if (role === "user") {
      return upsert(steps, {
        id: "prompt",
        title: "Prompt",
        status: type === "message_end" ? "completed" : "running",
        detail: messageText(message),
      })
    }

    if (role === "assistant" && message) {
      if (type === "message_end") return finishAssistantMessage(steps, message)
      return steps
    }
  }

  if (type === "message_update") return reduceMessageUpdate(steps, event)

  if (type === "tool_execution_start") {
    const id = string(event.toolCallId) ?? String(steps.length)
    const name = string(event.toolName) ?? "tool"
    return upsert(steps, {
      id: `tool-${id}`,
      title: `Using ${name}`,
      status: "running",
      detail: event.args === undefined ? undefined : truncate(event.args),
    })
  }

  if (type === "tool_execution_end") {
    const id = string(event.toolCallId) ?? String(steps.length)
    const name = string(event.toolName) ?? "tool"
    const isError = event.isError === true
    return upsert(steps, {
      id: `tool-${id}`,
      title: `${name} ${isError ? "failed" : "finished"}`,
      status: isError ? "error" : "completed",
      detail: event.result === undefined ? undefined : truncate(event.result),
      meta:
        typeof event.durationMs === "number"
          ? `${event.durationMs} ms`
          : undefined,
    })
  }

  if (type === "turn_end") {
    return update(steps, "turn", {
      title: "Turn completed",
      status: "completed",
    })
  }

  if (type === "agent_end") {
    return update(steps, "agent", {
      title: event.willRetry === true ? "Agent retrying" : "Agent finished",
      status: event.willRetry === true ? "running" : "completed",
    })
  }

  if (type === "auto_retry_start") {
    const attempt = number(event.attempt)
    const maxAttempts = number(event.maxAttempts)
    return upsert(steps, {
      id: "retry",
      title: "Retrying",
      status: "running",
      detail: string(event.error),
      meta:
        attempt === undefined
          ? undefined
          : `Attempt ${attempt}${maxAttempts ? ` of ${maxAttempts}` : ""}`,
    })
  }

  if (type === "auto_retry_end") {
    return update(steps, "retry", {
      status: event.success === false ? "error" : "completed",
    })
  }

  if (type === "agent_settled") {
    return completeLifecycle(steps, event.aborted === true)
  }

  if (type === "process_exit") {
    const code = number(event.code)
    return update(steps, "run", {
      title: code === 0 ? "Run completed" : "Run failed",
      status: code === 0 ? "completed" : "error",
      meta: code === undefined ? undefined : `Exit code ${code}`,
    })
  }

  if (type === "stderr" || type === "process_error" || type === "unparsed") {
    return [
      ...steps,
      {
        id: `error-${steps.length}`,
        title: "Run error",
        status: "error",
        detail: string(event.message) ?? truncate(event),
      },
    ]
  }

  return steps
}
