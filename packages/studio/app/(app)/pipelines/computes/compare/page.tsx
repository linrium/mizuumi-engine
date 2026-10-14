import { IconArrowLeft } from "@tabler/icons-react"
import Link from "next/link"
import { getComputeMetrics, listComputes } from "@/app/(app)/pipelines/actions"
import { ComputeComparison } from "@/app/(app)/pipelines/computes/compare/compute-comparison"
import { Button } from "@/components/ui/button"

export const dynamic = "force-dynamic"

function parameter(value: string | string[] | undefined) {
  return Array.isArray(value) ? value[0] : value
}

export default async function ComputeComparisonPage({
  searchParams,
}: {
  searchParams: Promise<Record<string, string | string[] | undefined>>
}) {
  try {
    const { items } = await listComputes()
    const requested = await searchParams
    const leftId =
      parameter(requested.left) ??
      items.find((compute) => compute.kind === "duck_server")?.id ??
      items[0]?.id
    const rightId =
      parameter(requested.right) ??
      items.find((compute) => compute.kind === "spark_application")?.id ??
      items.find((compute) => compute.id !== leftId)?.id

    const left = items.find((compute) => compute.id === leftId)
    const right = items.find((compute) => compute.id === rightId)
    const end = Date.now()
    const range = { start: end - 60 * 60 * 1_000, end, step: 60 }
    const [leftMetrics, rightMetrics] = await Promise.all([
      left ? getComputeMetrics(left.id, range) : Promise.resolve(null),
      right ? getComputeMetrics(right.id, range) : Promise.resolve(null),
    ])

    return (
      <main className="flex w-full flex-col gap-5">
        <header className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h1 className="text-lg font-semibold">Compute comparison</h1>
            <p className="mt-1 text-xs text-muted-foreground">
              Latest per-operation maxima over the same one-hour window
            </p>
          </div>
          <Button
            nativeButton={false}
            size="sm"
            variant="ghost"
            render={<Link href="/pipelines/computes" />}
          >
            <IconArrowLeft />
            Computes
          </Button>
        </header>
        <ComputeComparison
          computes={items}
          left={left ?? null}
          right={right ?? null}
          leftMetrics={leftMetrics}
          rightMetrics={rightMetrics}
        />
      </main>
    )
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error"
    return (
      <main className="flex w-full flex-col gap-4">
        <Button
          nativeButton={false}
          className="w-fit"
          variant="ghost"
          size="sm"
          render={<Link href="/pipelines/computes" />}
        >
          <IconArrowLeft />
          Computes
        </Button>
        <div className="rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-sm text-destructive">
          {message}
        </div>
      </main>
    )
  }
}
