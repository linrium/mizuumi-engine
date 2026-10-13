import {
  listMyAccessRequests,
  listRequestTeams,
} from "@/app/(app)/review/actions"
import { RequestsView } from "@/app/(app)/unitycatalog/(browser)/requests-view"
import { getDataContract, getDataQualityRuns } from "../contract/actions"

export default async function TableRequestsPage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string; table: string }>
}) {
  const { catalog, schema, table } = await params
  const fullName = `${catalog}.${schema}.${table}`
  const [teamsResult, requestsResult, contractResult, qualityResult] =
    await Promise.all([
      listRequestTeams(),
      listMyAccessRequests("table", fullName),
      getDataContract({ catalog, schema, table }),
      getDataQualityRuns({ catalog, schema, table }),
    ])

  return (
    <RequestsView
      securableType="table"
      fullName={fullName}
      teams={teamsResult.ok ? teamsResult.data : []}
      initialRequests={requestsResult.ok ? requestsResult.data : []}
      dataContract={contractResult.ok ? contractResult.data : null}
      latestQualityRun={qualityResult.ok ? qualityResult.data[0] : undefined}
      initialError={
        !teamsResult.ok
          ? teamsResult.error
          : !requestsResult.ok
            ? requestsResult.error
            : !contractResult.ok
              ? contractResult.error
              : contractResult.data && !qualityResult.ok
                ? qualityResult.error
                : undefined
      }
    />
  )
}
