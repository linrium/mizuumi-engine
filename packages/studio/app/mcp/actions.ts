"use server"

import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"

export type McpTable = {
  columns: string[]
  columnTypes?: string[]
  rows: unknown[][]
  rowCount: number
}

export type McpChatResult = {
  tool: string
  summary: string
  table?: McpTable
  raw?: string
  error?: string
}

type JsonRpc = {
  jsonrpc: "2.0"
  id?: number | string
  method?: string
  params?: Record<string, unknown>
  result?: Record<string, unknown>
  error?: { code: number; message: string; data?: unknown }
}

const MCP_URL = process.env.MCP_URL ?? "https://mcp.mizuumi.test/mcp"
let nextRpcId = 1

async function rpc(
  body: JsonRpc,
  sessionId?: string,
): Promise<{ response: JsonRpc | null; sessionId?: string }> {
  const headers: Record<string, string> = {
    "content-type": "application/json",
    accept: "application/json, text/event-stream",
  }
  if (sessionId) headers["mcp-session-id"] = sessionId

  const response = await fetch(MCP_URL, {
    method: "POST",
    headers,
    body: JSON.stringify(body),
  })
  if (!response.ok) {
    throw new Error(
      `MCP ${body.method ?? "request"} failed: ${response.status} ${await response.text()}`,
    )
  }
  const nextSessionId = response.headers.get("mcp-session-id") ?? sessionId
  const text = await response.text()
  if (!text) return { response: null, sessionId: nextSessionId ?? undefined }

  const contentType = response.headers.get("content-type") ?? ""
  const payload = contentType.includes("text/event-stream")
    ? parseSse(text)
    : (JSON.parse(text) as JsonRpc)
  return { response: payload, sessionId: nextSessionId ?? undefined }
}

function parseSse(text: string): JsonRpc | null {
  for (const chunk of text.split(/\r?\n\r?\n/)) {
    for (const line of chunk.split(/\r?\n/)) {
      if (line.startsWith("data:")) {
        try {
          return JSON.parse(line.slice(5).trim()) as JsonRpc
        } catch {
          continue
        }
      }
    }
  }
  return null
}

async function callMcpTool(
  tool: string,
  args: Record<string, unknown>,
): Promise<unknown> {
  const init = await rpc({
    jsonrpc: "2.0",
    id: nextRpcId++,
    method: "initialize",
    params: {
      protocolVersion: "2025-06-18",
      capabilities: {},
      clientInfo: { name: "mizuumi-studio", version: "0.1.0" },
    },
  })
  const sessionId = init.sessionId
  if (!sessionId) {
    throw new Error("MCP did not return a session id.")
  }

  await rpc(
    {
      jsonrpc: "2.0",
      method: "notifications/initialized",
      params: {},
    },
    sessionId,
  )

  const call = await rpc(
    {
      jsonrpc: "2.0",
      id: nextRpcId++,
      method: "tools/call",
      params: { name: tool, arguments: args },
    },
    sessionId,
  )
  if (call.response?.error) {
    throw new Error(call.response.error.message)
  }
  const result = call.response?.result
  const content = (result?.content as Array<{ type: string; text?: string }>) ?? []
  const first = content.find((item) => item.type === "text")
  return first?.text ? JSON.parse(first.text) : null
}

function pickTool(message: string): { tool: string; args: Record<string, unknown> } {
  const trimmed = message.trim()
  const normalized = trimmed.toLowerCase()
  const looksLikeSql = /^(select|with|show|describe|from)\b/.test(normalized)
  if (looksLikeSql) {
    return { tool: "run_sql", args: { sql: trimmed } }
  }
  if (normalized.includes("order")) {
    return { tool: "get_demo_orders", args: {} }
  }
  // Default demo behavior: show the orders table.
  return { tool: "get_demo_orders", args: {} }
}

export async function askMcp(message: string): Promise<McpChatResult> {
  const session = await auth.api.getSession({ headers: await headers() })
  if (!session) redirect("/login")

  const trimmed = message.trim()
  if (!trimmed) {
    return { tool: "", summary: "", error: "Type a message first." }
  }

  const { tool, args } = pickTool(trimmed)
  try {
    const parsed = await callMcpTool(tool, args)
    if (parsed && typeof parsed === "object" && "columns" in parsed && "rows" in parsed) {
      const table = parsed as McpTable
      return {
        tool,
        summary:
          tool === "get_demo_orders"
            ? `Fetched ${table.rowCount} row${table.rowCount === 1 ? "" : "s"} from unity.demo.orders.`
            : `Query returned ${table.rowCount} row${table.rowCount === 1 ? "" : "s"}.`,
        table,
      }
    }
    return {
      tool,
      summary: `Tool ${tool} completed.`,
      raw: JSON.stringify(parsed, null, 2),
    }
  } catch (error) {
    return {
      tool,
      summary: "",
      error: error instanceof Error ? error.message : "MCP call failed.",
    }
  }
}
