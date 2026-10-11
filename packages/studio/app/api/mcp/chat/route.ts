import { createOpenAI } from "@ai-sdk/openai"
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js"
import {
  convertToCoreMessages,
  experimental_createMCPClient as createMCPClient,
  streamText,
  type Message,
} from "ai"
import { headers } from "next/headers"
import { auth } from "@/lib/auth"

export const runtime = "nodejs"
export const maxDuration = 60

const MCP_URL = process.env.MCP_URL ?? "https://mcp.mizuumi.test/mcp"
const MODEL = process.env.OPENAI_MODEL ?? "gpt-4o-mini"

const SYSTEM_PROMPT = `You are the Mizuumi data assistant. You answer by calling the MCP tools:
- "get_demo_orders" fetches every row from unity.demo.orders.
- "run_sql" runs an arbitrary SQL query against the Mizuumi DuckDB Quack server
  (DuckDB SQL dialect). The default catalog is "unity" and the default schema
  is "demo", so prefer fully qualified names like unity.demo.orders.
Call a tool whenever the user asks for data. After a tool returns, briefly
describe what you fetched; do not re-paste the entire table — the UI renders
it. If a tool returns an error, explain it in plain language.`

export async function POST(request: Request) {
  const session = await auth.api.getSession({ headers: await headers() })
  if (!session) {
    return Response.json({ error: "Unauthorized" }, { status: 401 })
  }

  const apiKey = process.env.OPENAI_API_KEY
  if (!apiKey) {
    return Response.json(
      { error: "OPENAI_API_KEY is not configured on the server." },
      { status: 500 },
    )
  }

  const { messages } = (await request.json()) as { messages: Message[] }

  const openai = createOpenAI({
    apiKey,
    ...(process.env.OPENAI_BASE_URL
      ? { baseURL: process.env.OPENAI_BASE_URL }
      : {}),
  })

  const mcp = await createMCPClient({
    transport: new StreamableHTTPClientTransport(new URL(MCP_URL)),
  })

  try {
    const tools = await mcp.tools()
    const result = streamText({
      model: openai(MODEL),
      system: SYSTEM_PROMPT,
      messages: convertToCoreMessages(messages),
      tools,
      maxSteps: 5,
      onFinish: () => {
        void mcp.close()
      },
      onError({ error }) {
        console.error("[/api/mcp/chat] streamText error:", error)
      },
    })
    return result.toDataStreamResponse({
      getErrorMessage(error) {
        console.error("[/api/mcp/chat] stream error:", error)
        if (error == null) return "Unknown error"
        if (typeof error === "string") return error
        if (error instanceof Error) return error.message
        return JSON.stringify(error)
      },
    })
  } catch (error) {
    console.error("[/api/mcp/chat] setup error:", error)
    await mcp.close().catch(() => undefined)
    const message = error instanceof Error ? error.message : "MCP chat failed."
    return Response.json({ error: message }, { status: 502 })
  }
}
