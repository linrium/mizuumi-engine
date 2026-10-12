"use client"

import { useCallback, useEffect, useMemo, useState } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Separator } from "@/components/ui/separator"
import { Textarea } from "@/components/ui/textarea"
import {
  DEFAULT_PI_MODEL,
  DEFAULT_PI_THINKING_LEVEL,
  getPiThinkingLevels,
  PI_MODELS,
  type PiModel,
  type PiThinkingLevel,
} from "@/lib/pi-models"
import { buildPiSteps, type PiEvent, type StepStatus } from "@/app/(app)/agents/pi-steps"

type Agent = { name: string; status: string }

const modelItems = PI_MODELS.map(({ id, label }) => ({ value: id, label }))

const statusVariant: Record<
  StepStatus,
  "default" | "secondary" | "destructive"
> = {
  running: "default",
  completed: "secondary",
  error: "destructive",
}

async function responseError(response: Response) {
  const body: unknown = await response.json().catch(() => null)
  if (
    typeof body === "object" &&
    body !== null &&
    "error" in body &&
    typeof body.error === "string"
  ) {
    return body.error
  }
  return `Request failed (${response.status}).`
}

export function Chat() {
  const [agents, setAgents] = useState<Agent[]>([])
  const [selected, setSelected] = useState<string>()
  const [name, setName] = useState("")
  const [prompt, setPrompt] = useState("")
  const [model, setModel] = useState<PiModel>(DEFAULT_PI_MODEL)
  const [thinkingLevel, setThinkingLevel] = useState<PiThinkingLevel>(
    DEFAULT_PI_THINKING_LEVEL,
  )
  const [events, setEvents] = useState<PiEvent[]>([])
  const [debug, setDebug] = useState(false)
  const [newSessionAgents, setNewSessionAgents] = useState<Set<string>>(
    () => new Set(),
  )
  const [error, setError] = useState<string>()
  const [busy, setBusy] = useState(false)
  const steps = useMemo(() => buildPiSteps(events, debug), [events, debug])

  const loadAgents = useCallback(async () => {
    const response = await fetch("/api/agents", { cache: "no-store" })
    if (!response.ok) throw new Error(await responseError(response))
    const body = (await response.json()) as { agents: Agent[] }
    setAgents(body.agents)
    setSelected((current) =>
      current && body.agents.some((agent) => agent.name === current)
        ? current
        : body.agents[0]?.name,
    )
  }, [])

  useEffect(() => {
    loadAgents().catch((reason: unknown) =>
      setError(
        reason instanceof Error ? reason.message : "Unable to load agents.",
      ),
    )
  }, [loadAgents])

  async function createAgent(event: React.FormEvent) {
    event.preventDefault()
    setBusy(true)
    setError(undefined)
    try {
      const response = await fetch("/api/agents", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      })
      if (!response.ok) throw new Error(await responseError(response))
      setName("")
      await loadAgents()
      setSelected(name)
    } catch (reason) {
      setError(
        reason instanceof Error ? reason.message : "Unable to create agent.",
      )
    } finally {
      setBusy(false)
    }
  }

  async function deleteAgent(agentName: string) {
    setBusy(true)
    setError(undefined)
    try {
      const response = await fetch(
        `/api/agents/${encodeURIComponent(agentName)}`,
        {
          method: "DELETE",
        },
      )
      if (!response.ok) throw new Error(await responseError(response))
      setAgents((current) =>
        current.filter((agent) => agent.name !== agentName),
      )
      setSelected((current) => (current === agentName ? undefined : current))
    } catch (reason) {
      setError(
        reason instanceof Error ? reason.message : "Unable to delete agent.",
      )
    } finally {
      setBusy(false)
    }
  }

  async function runAgent(event: React.FormEvent) {
    event.preventDefault()
    if (!selected) return

    setBusy(true)
    setError(undefined)
    const newSession = newSessionAgents.has(selected)
    try {
      const response = await fetch(
        `/api/agents/${encodeURIComponent(selected)}/run`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ prompt, newSession, model, thinkingLevel }),
        },
      )
      if (!response.ok) throw new Error(await responseError(response))
      if (!response.body) throw new Error("The server did not return a stream.")

      if (newSession) {
        setNewSessionAgents((current) => {
          const next = new Set(current)
          next.delete(selected)
          return next
        })
      }
      setPrompt("")
      const reader = response.body.getReader()
      const decoder = new TextDecoder()
      let buffer = ""

      while (true) {
        const { value, done } = await reader.read()
        buffer += decoder.decode(value, { stream: !done })
        const lines = buffer.split("\n")
        buffer = lines.pop() ?? ""

        for (const line of lines) {
          if (!line) continue
          try {
            const piEvent = JSON.parse(line) as Record<string, unknown>
            setEvents((current) => [...current, piEvent])
          } catch {
            setEvents((current) => [
              ...current,
              { type: "unparsed", message: line },
            ])
          }
        }

        if (done) break
      }

      if (buffer) {
        try {
          const piEvent = JSON.parse(buffer) as Record<string, unknown>
          setEvents((current) => [...current, piEvent])
        } catch {
          setEvents((current) => [
            ...current,
            { type: "unparsed", message: buffer },
          ])
        }
      }
    } catch (reason) {
      setError(
        reason instanceof Error ? reason.message : "Unable to run agent.",
      )
    } finally {
      setBusy(false)
    }
  }

  function startNewSession() {
    if (!selected) return
    setNewSessionAgents((current) => new Set(current).add(selected))
    setEvents([])
    setError(undefined)
  }

  const thinkingLevels = getPiThinkingLevels(model)
  const thinkingItems = thinkingLevels.map((level) => ({
    value: level,
    label: level === "off" ? "None" : level,
  }))

  function selectModel(nextModel: PiModel) {
    setModel(nextModel)
    const supportedLevels = getPiThinkingLevels(nextModel)
    if (!supportedLevels.some((level) => level === thinkingLevel)) {
      setThinkingLevel(DEFAULT_PI_THINKING_LEVEL)
    }
  }

  return (
    <main className="mx-auto w-full max-w-3xl space-y-4 p-4">
      <h1 className="text-xl font-semibold">Pi agents</h1>

      <Card>
        <CardHeader>
          <CardTitle>Create agent</CardTitle>
          <CardDescription>Start a new OpenShell sandbox.</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="flex gap-2" onSubmit={createAgent}>
            <label className="flex-1" htmlFor="agent-name">
              <span className="sr-only">New agent name</span>
              <Input
                id="agent-name"
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="my-agent"
                required
              />
            </label>
            <Button type="submit" disabled={busy}>
              Create
            </Button>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Running agents</CardTitle>
        </CardHeader>
        <CardContent>
          {agents.length === 0 ? <p>No agents.</p> : null}
          <ul className="space-y-2">
            {agents.map((agent, index) => (
              <li key={agent.name}>
                {index > 0 ? <Separator className="mb-2" /> : null}
                <div className="flex items-center gap-2">
                  <Button
                    type="button"
                    variant={selected === agent.name ? "secondary" : "ghost"}
                    onClick={() => setSelected(agent.name)}
                  >
                    {agent.name}
                  </Button>
                  <Badge variant="outline">{agent.status}</Badge>
                  <Button
                    className="ml-auto"
                    type="button"
                    variant="destructive"
                    disabled={busy}
                    onClick={() => deleteAgent(agent.name)}
                  >
                    Delete
                  </Button>
                </div>
              </li>
            ))}
          </ul>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between gap-2">
            <CardTitle>Run {selected ?? "an agent"}</CardTitle>
            <Button
              type="button"
              variant="outline"
              disabled={busy || !selected}
              onClick={startNewSession}
            >
              New session
            </Button>
          </div>
          <CardDescription>
            {selected && newSessionAgents.has(selected)
              ? "Your next message will start a new Pi session."
              : "Messages continue the current Pi session."}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <form className="space-y-2" onSubmit={runAgent}>
            <div className="grid gap-2 sm:grid-cols-2">
              <label className="space-y-1" htmlFor="agent-model">
                <span className="text-sm font-medium">Model</span>
                <Select
                  items={modelItems}
                  value={model}
                  disabled={busy}
                  onValueChange={(value) =>
                    value && selectModel(value as PiModel)
                  }
                >
                  <SelectTrigger className="h-9 w-full" id="agent-model">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {modelItems.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </label>
              <label className="space-y-1" htmlFor="thinking-level">
                <span className="text-sm font-medium">Thinking level</span>
                <Select
                  items={thinkingItems}
                  value={thinkingLevel}
                  disabled={busy}
                  onValueChange={(value) =>
                    value && setThinkingLevel(value as PiThinkingLevel)
                  }
                >
                  <SelectTrigger className="h-9 w-full" id="thinking-level">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {thinkingItems.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </label>
            </div>
            <label htmlFor="agent-prompt">
              <span className="sr-only">Prompt</span>
              <Textarea
                id="agent-prompt"
                value={prompt}
                onChange={(event) => setPrompt(event.target.value)}
                placeholder="Ask the agent to do something"
                required
              />
            </label>
            <div className="mt-3">
              <Button type="submit" disabled={busy || !selected}>
                Run
              </Button>
            </div>
          </form>

          {error ? (
            <p className="text-destructive" role="alert">
              {error}
            </p>
          ) : null}

          <Separator />

          <section aria-labelledby="steps-heading">
            <div className="mb-2 flex items-center justify-between gap-2">
              <h2 className="font-medium" id="steps-heading">
                Steps
              </h2>
              <Button
                type="button"
                size="sm"
                variant={debug ? "secondary" : "outline"}
                aria-pressed={debug}
                onClick={() => setDebug((current) => !current)}
              >
                Debug {debug ? "on" : "off"}
              </Button>
            </div>
            <ScrollArea className="h-96 rounded-md border">
              {steps.length === 0 ? (
                <p className="p-3 text-muted-foreground">No steps yet.</p>
              ) : (
                <ol className="space-y-2 p-3" aria-live="polite">
                  {steps.map((step) => (
                    <li
                      className="space-y-1 rounded-md border p-3"
                      key={step.id}
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="font-medium">{step.title}</span>
                        <Badge variant={statusVariant[step.status]}>
                          {step.status}
                        </Badge>
                      </div>
                      {step.detail ? (
                        <p className="whitespace-pre-wrap text-sm">
                          {step.detail}
                        </p>
                      ) : null}
                      {step.meta ? (
                        <p className="text-xs text-muted-foreground">
                          {step.meta}
                        </p>
                      ) : null}
                      {debug ? (
                        <details>
                          <summary className="cursor-pointer text-sm">
                            JSON
                          </summary>
                          <pre className="mt-2 overflow-x-auto whitespace-pre-wrap rounded-md border p-2 text-xs">
                            {JSON.stringify(step.event, null, 2)}
                          </pre>
                        </details>
                      ) : null}
                    </li>
                  ))}
                </ol>
              )}
            </ScrollArea>
          </section>
        </CardContent>
      </Card>
    </main>
  )
}
