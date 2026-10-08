import { headers } from "next/headers"
import { auth } from "@/lib/auth"
import {
  assertManagedAgentSandbox,
  assertSandboxName,
  errorMessage,
  spawnOpenShell,
} from "@/lib/openshell"

export const runtime = "nodejs"
export const dynamic = "force-dynamic"

export async function POST(
  request: Request,
  context: RouteContext<"/api/agents/[name]/run">,
) {
  const session = await auth.api.getSession({ headers: await headers() })
  if (!session) {
    return Response.json({ error: "Unauthorized" }, { status: 401 })
  }

  let name: string
  let prompt: string
  let newSession: boolean

  try {
    name = (await context.params).name
    assertSandboxName(name)
    const body: unknown = await request.json()
    newSession =
      typeof body === "object" &&
      body !== null &&
      "newSession" in body &&
      body.newSession === true
    prompt =
      typeof body === "object" &&
      body !== null &&
      "prompt" in body &&
      typeof body.prompt === "string"
        ? body.prompt.trim()
        : ""
    if (!prompt || prompt.length > 20_000) {
      throw new Error("Prompt must contain between 1 and 20,000 characters.")
    }
  } catch (error) {
    return Response.json({ error: errorMessage(error) }, { status: 400 })
  }

  try {
    await assertManagedAgentSandbox(name)
  } catch (error) {
    const message = errorMessage(error)
    return Response.json(
      { error: message },
      { status: message.includes("not managed by Studio") ? 404 : 502 },
    )
  }

  const child = spawnOpenShell([
    "sandbox",
    "exec",
    "--name",
    name,
    "--workdir",
    "/workspace",
    "--timeout",
    "120",
    "--no-tty",
    "--",
    "pi-custom-openai",
    "--mode",
    "json",
    ...(newSession ? [] : ["--continue"]),
    "--",
    prompt,
  ])
  child.stdin.end()

  const encoder = new TextEncoder()
  let streamClosed = false
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      let stderr = ""

      const enqueue = (chunk: Uint8Array) => {
        if (!streamClosed) controller.enqueue(chunk)
      }

      const close = () => {
        if (!streamClosed) {
          streamClosed = true
          controller.close()
        }
      }

      enqueue(
        encoder.encode(
          `${JSON.stringify({ type: "run_start", agent: name })}\n`,
        ),
      )

      child.stdout.on("data", (chunk: Buffer) => enqueue(chunk))
      child.stderr.on("data", (chunk: Buffer) => {
        stderr += chunk.toString("utf8")
        const lines = stderr.split("\n")
        stderr = lines.pop() ?? ""
        for (const line of lines) {
          if (line) {
            enqueue(
              encoder.encode(
                `${JSON.stringify({ type: "stderr", message: line })}\n`,
              ),
            )
          }
        }
      })
      child.on("error", (error) => {
        enqueue(
          encoder.encode(
            `${JSON.stringify({ type: "process_error", message: error.message })}\n`,
          ),
        )
        close()
      })
      child.on("close", (code) => {
        if (streamClosed) return
        if (stderr) {
          enqueue(
            encoder.encode(
              `${JSON.stringify({ type: "stderr", message: stderr })}\n`,
            ),
          )
        }
        enqueue(
          encoder.encode(
            `${JSON.stringify({ type: "process_exit", code: code ?? 1 })}\n`,
          ),
        )
        close()
      })

      request.signal.addEventListener("abort", () => child.kill("SIGTERM"), {
        once: true,
      })
    },
    cancel() {
      streamClosed = true
      child.kill("SIGTERM")
    },
  })

  return new Response(stream, {
    headers: {
      "Cache-Control": "no-cache, no-transform",
      "Content-Type": "application/x-ndjson; charset=utf-8",
      "X-Accel-Buffering": "no",
    },
  })
}
