import {
  listMyAccessRequests,
  listRequestTeams,
} from "@/app/(app)/review/actions"
import { RequestsView } from "@/app/(app)/unitycatalog/(browser)/requests-view"

export default async function SchemaRequestsPage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string }>
}) {
  const { catalog, schema } = await params
  const fullName = `${catalog}.${schema}`
  const [teamsResult, requestsResult] = await Promise.all([
    listRequestTeams(),
    listMyAccessRequests("schema", fullName),
  ])

  return (
    <RequestsView
      securableType="schema"
      fullName={fullName}
      teams={teamsResult.ok ? teamsResult.data : []}
      initialRequests={requestsResult.ok ? requestsResult.data : []}
      initialError={
        !teamsResult.ok
          ? teamsResult.error
          : !requestsResult.ok
            ? requestsResult.error
            : undefined
      }
    />
  )
}
