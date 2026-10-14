import { listComputes } from "@/app/(app)/pipelines/actions"
import { ComputeView } from "@/app/(app)/pipelines/computes/compute-view"

export const dynamic = "force-dynamic"

export default async function ComputePage() {
  try {
    const { items } = await listComputes()
    return <ComputeView computes={items} />
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error"
    return <ComputeView computes={[]} error={message} />
  }
}
