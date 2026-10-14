import { IconArrowLeft } from "@tabler/icons-react"
import Link from "next/link"
import { getCompute, getComputeMetrics } from "@/app/(app)/pipelines/actions"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"

export const dynamic = "force-dynamic"

export default async function ComputeDetailPage({
  params,
}: PageProps<"/compute/[compute_id]">) {
  const { compute_id: computeId } = await params

  try {
    const [compute, metrics] = await Promise.all([
      getCompute(computeId),
      getComputeMetrics(computeId),
    ])

    return (
      <main className="flex w-full flex-col gap-5">
        <header className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h1 className="font-mono text-lg font-semibold">{compute.name}</h1>
            <p className="mt-1 text-xs text-muted-foreground">
              {compute.namespace} · {compute.id}
            </p>
          </div>
          <Badge variant={compute.ready ? "default" : "secondary"}>
            {compute.ready ? "Ready" : compute.phase}
          </Badge>
        </header>

        <section className="flex flex-col gap-2">
          <h2 className="text-sm font-semibold">Metrics</h2>
          <pre className="max-h-[calc(100vh-13rem)] overflow-auto rounded-md border bg-muted/40 p-4 font-mono text-xs leading-relaxed">
            {JSON.stringify(metrics, null, 2)}
          </pre>
        </section>
      </main>
    )
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error"
    return (
      <main className="flex w-full flex-col gap-4 px-6 py-6">
        <Button
          nativeButton={false}
          className="w-fit"
          variant="ghost"
          size="sm"
          render={<Link href="/pipelines/compute" />}
        >
          <IconArrowLeft />
          Compute
        </Button>
        <div className="rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-sm text-destructive">
          {message}
        </div>
      </main>
    )
  }
}
