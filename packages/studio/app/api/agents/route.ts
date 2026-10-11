import { headers } from "next/headers"
import { auth } from "@/lib/auth"
import {
  assertSandboxName,
  createSandbox,
  errorMessage,
  listAgentSandboxes,
} from "@/lib/openshell"
import { getPiConfig } from "@/lib/pi-config"
import { DEFAULT_PI_MODEL, PI_MODELS_JSON } from "@/lib/pi-models"

export const runtime = "nodejs"

async function isSignedIn() {
  return Boolean(await auth.api.getSession({ headers: await headers() }))
}

export async function GET() {
  if (!(await isSignedIn())) {
    return Response.json({ error: "Unauthorized" }, { status: 401 })
  }

  try {
    return Response.json({ agents: await listAgentSandboxes() })
  } catch (error) {
    return Response.json({ error: errorMessage(error) }, { status: 502 })
  }
}

export async function POST(request: Request) {
  if (!(await isSignedIn())) {
    return Response.json({ error: "Unauthorized" }, { status: 401 })
  }

  try {
    const body: unknown = await request.json()
    const name =
      typeof body === "object" && body !== null && "name" in body
        ? body.name
        : undefined
    assertSandboxName(name)

    const image = process.env.OPENSHELL_PI_IMAGE ?? "pi-custom-openai:local-1"
    const provider = process.env.OPENSHELL_PI_PROVIDER ?? "pi-custom-openai"
    const mcpProvider =
      process.env.OPENSHELL_MCP_PROVIDER ?? "mizuumi-mcp"
    const { baseUrl } = await getPiConfig()

    const mcpUrl =
      process.env.MIZUUMI_MCP_URL ??
      "http://mcp.mcp.svc.cluster.local:8080/mcp"
    const mcpServersJson = JSON.stringify({
      mizuumi: {
        url: mcpUrl,
        timeout: 60,
        enabled: true,
        exposure: "direct",
        description:
          "Mizuumi DuckDB Quack tools: run_sql and get_demo_orders.",
      },
    })

    const agent = await createSandbox({
      name,
      image,
      providers: [provider, mcpProvider],
      environment: {
        CUSTOM_OPENAI_BASE_URL: baseUrl,
        CUSTOM_OPENAI_MODEL: DEFAULT_PI_MODEL,
        CUSTOM_OPENAI_MODELS_JSON: PI_MODELS_JSON,
        PI_CODING_AGENT_DIR: "/tmp/pi-agent-runtime",
        PI_MCP_SERVERS_JSON: mcpServersJson,
      },
      labels: { "mizuumi.studio.agent": "true" },
      policy: {
        version: 1,
        filesystem: {
          includeWorkdir: true,
          readWrite: ["/workspace"],
        },
      },
      command: ["sleep", "infinity"],
      tty: false,
    })

    return Response.json({ agent }, { status: 201 })
  } catch (error) {
    const message = errorMessage(error)
    return Response.json(
      { error: message },
      { status: message.startsWith("Agent name") ? 400 : 502 },
    )
  }
}
