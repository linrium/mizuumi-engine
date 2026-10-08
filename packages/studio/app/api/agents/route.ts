import { headers } from "next/headers"
import { auth } from "@/lib/auth"
import {
  assertSandboxName,
  errorMessage,
  listAgentSandboxes,
  runOpenShell,
} from "@/lib/openshell"
import { getPiConfig } from "@/lib/pi-config"

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

    const image = process.env.OPENSHELL_PI_IMAGE ?? "pi-custom-openai:local"
    const provider = process.env.OPENSHELL_PI_PROVIDER ?? "pi-custom-openai"
    const { baseUrl, model } = await getPiConfig()

    const output = await runOpenShell([
      "sandbox",
      "create",
      "--name",
      name,
      "--from",
      image,
      "--provider",
      provider,
      "--env",
      `CUSTOM_OPENAI_BASE_URL=${baseUrl}`,
      "--env",
      `CUSTOM_OPENAI_MODEL=${model}`,
      "--env",
      "PI_CODING_AGENT_DIR=/tmp/pi-agent-runtime",
      "--label",
      "mizuumi.studio.agent=true",
      "--detach",
      "--no-tty",
      "--output",
      "json",
      "--",
      "sleep",
      "infinity",
    ])

    return Response.json({ agent: JSON.parse(output) }, { status: 201 })
  } catch (error) {
    const message = errorMessage(error)
    return Response.json(
      { error: message },
      { status: message.startsWith("Agent name") ? 400 : 502 },
    )
  }
}
