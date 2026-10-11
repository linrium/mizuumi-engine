"use client"

import { useChat } from "@ai-sdk/react"
import {
  IconAlertCircle,
  IconRobot,
  IconSend,
  IconTool,
  IconUser,
} from "@tabler/icons-react"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Textarea } from "@/components/ui/textarea"

type ToolInvocation =
  | {
      state: "partial-call" | "call"
      toolCallId: string
      toolName: string
      args?: Record<string, unknown>
    }
  | {
      state: "result"
      toolCallId: string
      toolName: string
      args?: Record<string, unknown>
      result: unknown
    }

type Table = {
  columns: string[]
  columnTypes?: string[]
  rows: unknown[][]
  rowCount?: number
}

const SUGGESTIONS = [
  "Get the demo orders",
  "Show every row from unity.demo.orders",
  "How many orders do we have?",
]

export function McpChat() {
  const { messages, input, handleInputChange, handleSubmit, status, error } =
    useChat({ api: "/api/mcp/chat" })
  const busy = status === "submitted" || status === "streaming"

  function ask(message: string) {
    const synthetic = {
      preventDefault() {},
    } as unknown as React.FormEvent<HTMLFormElement>
    handleInputChange({
      target: { value: message },
    } as unknown as React.ChangeEvent<HTMLTextAreaElement>)
    queueMicrotask(() => handleSubmit(synthetic))
  }

  return (
    <main className="mx-auto flex min-h-dvh w-full max-w-3xl flex-col gap-4 p-6">
      <header>
        <h1 className="flex items-center gap-2 text-lg font-semibold">
          <IconRobot className="size-5" />
          MCP chat
        </h1>
        <p className="text-xs text-muted-foreground">
          OpenAI reasons over the Mizuumi MCP tools (<code>get_demo_orders</code>
          , <code>run_sql</code>) and renders table results inline.
        </p>
      </header>

      <Card className="min-h-0 flex-1">
        <CardHeader>
          <CardTitle>Conversation</CardTitle>
          <CardDescription>
            Try one of:
            <span className="ml-2 inline-flex flex-wrap gap-2">
              {SUGGESTIONS.map((suggestion) => (
                <button
                  className="rounded-md border px-2 py-0.5 text-xs hover:bg-muted disabled:opacity-50"
                  disabled={busy}
                  key={suggestion}
                  type="button"
                  onClick={() => ask(suggestion)}
                >
                  {suggestion}
                </button>
              ))}
            </span>
          </CardDescription>
        </CardHeader>
        <CardContent className="min-h-0">
          <ScrollArea className="h-[28rem] rounded-md border">
            {messages.length === 0 ? (
              <p className="p-4 text-sm text-muted-foreground">
                Ask about Mizuumi data. The assistant will call the MCP tool and
                render the result as a table.
              </p>
            ) : (
              <ol className="space-y-3 p-3" aria-live="polite">
                {messages.map((message) => (
                  <li key={message.id}>
                    {message.role === "user" ? (
                      <UserBubble content={textOf(message.parts)} />
                    ) : (
                      <AssistantBubble parts={message.parts ?? []} />
                    )}
                  </li>
                ))}
                {busy ? (
                  <li className="text-sm text-muted-foreground">Thinking…</li>
                ) : null}
              </ol>
            )}
          </ScrollArea>

          <form className="mt-4 space-y-2" onSubmit={handleSubmit}>
            <Textarea
              aria-label="Message"
              className="min-h-20 resize-y"
              disabled={busy}
              placeholder="Ask about demo orders or hand over SQL…"
              value={input}
              onChange={handleInputChange}
              onKeyDown={(event) => {
                if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
                  event.currentTarget.form?.requestSubmit()
                }
              }}
            />
            <div className="flex items-center justify-between gap-2">
              <span className="text-xs text-muted-foreground">
                Command/Ctrl + Enter to send.
              </span>
              <Button type="submit" disabled={busy || !input.trim()}>
                <IconSend />
                Send
              </Button>
            </div>
          </form>

          {error ? (
            <div className="mt-3 flex gap-2 rounded-md border border-destructive/40 bg-destructive/5 p-3 text-sm">
              <IconAlertCircle className="mt-0.5 size-4 text-destructive" />
              <div>
                <div className="font-medium text-destructive">Chat error</div>
                <div className="whitespace-pre-wrap font-mono text-xs">
                  {error.message}
                </div>
              </div>
            </div>
          ) : null}
        </CardContent>
      </Card>
    </main>
  )
}

function textOf(parts: unknown): string {
  if (!Array.isArray(parts)) return ""
  return parts
    .filter(
      (part): part is { type: "text"; text: string } =>
        typeof part === "object" &&
        part !== null &&
        (part as { type?: string }).type === "text",
    )
    .map((part) => part.text)
    .join("")
}

function UserBubble({ content }: { content: string }) {
  return (
    <div className="flex gap-2">
      <IconUser className="mt-1 size-4 text-muted-foreground" />
      <div className="whitespace-pre-wrap rounded-md border bg-muted px-3 py-2 text-sm">
        {content}
      </div>
    </div>
  )
}

function AssistantBubble({ parts }: { parts: unknown[] }) {
  return (
    <div className="flex gap-2">
      <IconRobot className="mt-1 size-4 text-muted-foreground" />
      <div className="w-full space-y-2">
        {parts.map((part, index) => {
          if (!part || typeof part !== "object") return null
          const type = (part as { type?: string }).type

          if (type === "text") {
            const text = (part as { text?: unknown }).text
            if (typeof text !== "string" || !text.trim()) return null
            return (
              <div
                className="whitespace-pre-wrap rounded-md border px-3 py-2 text-sm"
                key={`text-${index}`}
              >
                {text}
              </div>
            )
          }

          if (type === "tool-invocation") {
            const invocation = (part as { toolInvocation?: ToolInvocation })
              .toolInvocation
            if (!invocation) return null
            return (
              <ToolBubble
                invocation={invocation}
                key={invocation.toolCallId ?? `tool-${index}`}
              />
            )
          }

          if (type === "reasoning") {
            const reasoning =
              (part as { reasoning?: unknown }).reasoning ??
              (part as { text?: unknown }).text
            if (typeof reasoning !== "string" || !reasoning.trim()) return null
            return (
              <details
                className="rounded-md border bg-muted/40 px-3 py-2 text-xs"
                key={`reasoning-${index}`}
              >
                <summary className="cursor-pointer text-muted-foreground">
                  Reasoning
                </summary>
                <p className="mt-2 whitespace-pre-wrap">{reasoning}</p>
              </details>
            )
          }

          return null
        })}
      </div>
    </div>
  )
}

function ToolBubble({ invocation }: { invocation: ToolInvocation }) {
  const argsPreview = invocation.args ? JSON.stringify(invocation.args) : ""
  return (
    <div className="space-y-2 rounded-md border bg-muted/40 px-3 py-2 text-sm">
      <div className="flex items-center gap-2">
        <IconTool className="size-4 text-muted-foreground" />
        <span className="font-medium">{invocation.toolName}</span>
        <span className="text-xs text-muted-foreground">
          {invocation.state === "result" ? "returned" : "calling…"}
        </span>
      </div>
      {argsPreview ? (
        <code className="block overflow-x-auto rounded border bg-background p-1.5 text-xs">
          {argsPreview}
        </code>
      ) : null}
      {invocation.state === "result" ? (
        <ToolResult result={invocation.result} />
      ) : null}
    </div>
  )
}

function ToolResult({ result }: { result: unknown }) {
  const parsed = parseResult(result)
  if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
    const maybeTable = parsed as Partial<Table> & { error?: unknown }
    if (
      Array.isArray(maybeTable.columns) &&
      Array.isArray(maybeTable.rows)
    ) {
      return <ResultTable table={maybeTable as Table} />
    }
  }
  if (typeof parsed === "string") {
    return (
      <pre className="overflow-x-auto rounded-md border bg-background p-2 text-xs">
        {parsed}
      </pre>
    )
  }
  return (
    <pre className="overflow-x-auto rounded-md border bg-background p-2 text-xs">
      {JSON.stringify(parsed, null, 2)}
    </pre>
  )
}

function parseResult(result: unknown): unknown {
  if (result && typeof result === "object" && "content" in result) {
    const content = (result as { content: unknown }).content
    if (Array.isArray(content)) {
      const text = content
        .filter(
          (part): part is { type: string; text: string } =>
            typeof part === "object" &&
            part !== null &&
            (part as { type?: string }).type === "text" &&
            typeof (part as { text?: unknown }).text === "string",
        )
        .map((part) => part.text)
        .join("\n")
      return parseMaybeJson(text)
    }
  }
  if (typeof result === "string") return parseMaybeJson(result)
  return result
}

function parseMaybeJson(text: string): unknown {
  const trimmed = text.trim()
  if (!trimmed) return trimmed
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    try {
      return JSON.parse(trimmed)
    } catch {
      return trimmed
    }
  }
  return trimmed
}

function ResultTable({ table }: { table: Table }) {
  if (table.rows.length === 0) {
    return <p className="text-xs text-muted-foreground">No rows.</p>
  }
  return (
    <div className="overflow-auto rounded-md border bg-background">
      <table className="w-full border-collapse text-left text-xs">
        <thead className="sticky top-0 bg-card">
          <tr>
            {table.columns.map((column, index) => (
              <th
                className="border-b px-3 py-2 font-medium whitespace-nowrap"
                key={`${column}:${index}`}
              >
                <span>{column}</span>
                {table.columnTypes?.[index] ? (
                  <span className="ml-2 font-normal text-muted-foreground">
                    {table.columnTypes[index]}
                  </span>
                ) : null}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {table.rows.map((row, rowIndex) => (
            <tr
              className="border-b last:border-0"
              key={`row-${rowIndex}-${JSON.stringify(row).slice(0, 32)}`}
            >
              {row.map((value, columnIndex) => (
                <td
                  className="max-w-96 px-3 py-2 align-top whitespace-pre-wrap"
                  key={`cell-${rowIndex}-${columnIndex}`}
                >
                  {formatValue(value)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function formatValue(value: unknown) {
  if (value === null) return "NULL"
  if (typeof value === "object") return JSON.stringify(value)
  return String(value)
}
