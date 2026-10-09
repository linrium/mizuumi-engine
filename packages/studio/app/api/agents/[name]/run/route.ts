import { headers } from "next/headers"
import { auth } from "@/lib/auth"
import {
  assertManagedAgentSandbox,
  assertSandboxName,
  errorMessage,
  execSandboxStream,
} from "@/lib/openshell"
import {
  isPiModel,
  isPiThinkingLevel,
  type PiModel,
  type PiThinkingLevel,
  supportsPiThinkingLevel,
} from "@/lib/pi-models"

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
  let model: PiModel
  let thinkingLevel: PiThinkingLevel

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
    const requestedModel =
      typeof body === "object" && body !== null && "model" in body
        ? body.model
        : undefined
    if (!isPiModel(requestedModel)) {
      throw new Error("Select a supported GPT model.")
    }
    model = requestedModel

    const requestedThinkingLevel =
      typeof body === "object" && body !== null && "thinkingLevel" in body
        ? body.thinkingLevel
        : undefined
    if (!isPiThinkingLevel(requestedThinkingLevel)) {
      throw new Error("Select a supported thinking level.")
    }
    thinkingLevel = requestedThinkingLevel
    if (!supportsPiThinkingLevel(model, thinkingLevel)) {
      throw new Error(
        `${model} does not support the ${thinkingLevel} thinking level.`,
      )
    }
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

  const abortController = new AbortController()
  const execution = execSandboxStream(
    name,
    [
      "pi-custom-openai",
      "--model",
      `custom-openai/${model}`,
      "--thinking",
      thinkingLevel,
      "--mode",
      "json",
      ...(newSession ? [] : ["--continue"]),
      "--",
      prompt,
    ],
    {
      workdir: "/workspace",
      timeoutSecs: 120,
      signal: abortController.signal,
    },
  )

  const encoder = new TextEncoder()
  let streamClosed = false
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      let stderr = ""
      const stderrDecoder = new TextDecoder()

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

      const enqueueStderr = (chunk: Uint8Array, stream = true) => {
        stderr += stderrDecoder.decode(chunk, { stream })
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
      }

      const finish = (code: number) => {
        enqueueStderr(new Uint8Array(), false)
        if (stderr) {
          enqueue(
            encoder.encode(
              `${JSON.stringify({ type: "stderr", message: stderr })}\n`,
            ),
          )
        }
        enqueue(
          encoder.encode(`${JSON.stringify({ type: "process_exit", code })}\n`),
        )
        close()
      }

      void (async () => {
        try {
          for await (const event of execution) {
            if ("type" in event) {
              finish(event.exitCode)
              return
            }

            if (event.stream === "stdout") enqueue(event.data)
            else enqueueStderr(event.data)
          }
        } catch (error) {
          if (!streamClosed) {
            enqueue(
              encoder.encode(
                `${JSON.stringify({ type: "process_error", message: errorMessage(error) })}\n`,
              ),
            )
            close()
          }
        }
      })()

      request.signal.addEventListener("abort", () => abortController.abort(), {
        once: true,
      })
    },
    cancel() {
      streamClosed = true
      abortController.abort()
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
