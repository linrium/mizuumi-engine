import {
  listMyAccessRequests,
  listRequestTeams,
} from "@/app/(app)/review/actions"
import { RequestsView } from "@/app/(app)/unitycatalog/(browser)/requests-view"

export default async function TableRequestsPage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string; table: string }>
}) {
  const { catalog, schema, table } = await params
  const fullName = `${catalog}.${schema}.${table}`
  const [teamsResult, requestsResult] = await Promise.all([
    listRequestTeams(),
    listMyAccessRequests("table", fullName),
  ])

  return (
    <RequestsView
      securableType="table"
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
