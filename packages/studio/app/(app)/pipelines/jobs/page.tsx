import { listPipelines } from "@/app/(app)/pipelines/actions"
import { PipelinesView } from "@/app/(app)/pipelines/pipelines-view"

export const dynamic = "force-dynamic"

export default async function PipelinesJobsPage() {
  try {
    const { items } = await listPipelines()
    return <PipelinesView initialPipelines={items} />
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error"
    return <PipelinesView initialPipelines={[]} initialError={message} />
  }
}
