import { headers } from "next/headers"
import { auth } from "@/lib/auth"
import {
  assertManagedAgentSandbox,
  assertSandboxName,
  deleteSandbox,
  errorMessage,
} from "@/lib/openshell"

export const runtime = "nodejs"

export async function DELETE(
  _request: Request,
  context: RouteContext<"/api/agents/[name]">,
) {
  const session = await auth.api.getSession({ headers: await headers() })
  if (!session) {
    return Response.json({ error: "Unauthorized" }, { status: 401 })
  }

  try {
    const { name } = await context.params
    assertSandboxName(name)
    await assertManagedAgentSandbox(name)
    await deleteSandbox(name)
    return new Response(null, { status: 204 })
  } catch (error) {
    const message = errorMessage(error)
    return Response.json(
      { error: message },
      {
        status: message.startsWith("Agent name")
          ? 400
          : message.includes("not managed by Studio")
            ? 404
            : 502,
      },
    )
  }
}
