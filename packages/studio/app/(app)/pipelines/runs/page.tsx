import { RunsView } from "@/app/(app)/pipelines/runs/runs-view"
import { getRuns } from "@/lib/dagster"

export const dynamic = "force-dynamic"

export default async function PipelinesRunsPage() {
  try {
    return <RunsView initialRuns={await getRuns()} />
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error"
    return <RunsView initialRuns={[]} initialError={message} />
  }
}
