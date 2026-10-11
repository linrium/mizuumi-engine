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

const SYSTEM_PROMPT = `You are the Mizuumi data assistant. You answer by calling MCP tools against a DuckDB-backed Unity Catalog.

All tool parameters are required. For filters that take catalog/schema, pass "" to use
the server default (catalog "unity", schema "demo") or "*" to disable the filter. For
preview_table, pass limit=0 to use the default (500 rows).

Discovery tools — use these to find data you don't already know about:
- "list_catalogs" — every attached catalog (database).
- "list_schemas"  — schemas, filtered by catalog. Pass catalog="*" to see all.
- "list_tables"   — tables and views. Pass catalog/schema="*" to broaden.
- "describe_table" — columns, types, nullability for one table.

Reading tools:
- "preview_table" — first N rows of a table (default 500, max 5000).
- "run_sql"       — arbitrary DuckDB SQL. Use for aggregations, joins, filters.

How to work:
1. If the user names a table/catalog/schema explicitly, go straight to preview_table or run_sql.
2. If they describe data loosely ("show me orders", "what customers do we have"), start with
   list_tables (default catalog="unity", schema="demo"), broaden with "*" if nothing matches,
   then describe_table before writing SQL.
3. Prefer fully qualified names (catalog.schema.table) in run_sql.
4. After a tool returns, briefly state what you fetched. Do NOT re-paste tables — the UI renders
   them from the tool result. If a tool errors, explain it in plain language and try a recovery
   (e.g. broaden the schema filter, describe_table, inspect information_schema).`

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
      maxSteps: 10,
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
