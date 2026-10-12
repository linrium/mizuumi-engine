import {
  listMyAccessRequests,
  listRequestTeams,
} from "@/app/(app)/review/actions"
import { RequestsView } from "@/app/(app)/unitycatalog/(browser)/requests-view"

export default async function CatalogRequestsPage({
  params,
}: {
  params: Promise<{ catalog: string }>
}) {
  const { catalog } = await params
  const [teamsResult, requestsResult] = await Promise.all([
    listRequestTeams(),
    listMyAccessRequests("catalog", catalog),
  ])

  return (
    <RequestsView
      securableType="catalog"
      fullName={catalog}
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
