import { listRequestQueue } from "../actions"
import { RequestQueue } from "./request-queue"

export default async function RequestQueuePage() {
  const result = await listRequestQueue()

  return (
    <RequestQueue
      initialRequests={result.ok ? result.data.requests : []}
      approvalTeams={result.ok ? result.data.approvalTeams : []}
      initialError={result.ok ? undefined : result.error}
    />
  )
}
