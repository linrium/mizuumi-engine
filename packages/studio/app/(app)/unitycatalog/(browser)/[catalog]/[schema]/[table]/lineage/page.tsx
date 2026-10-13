import { LineageGraph } from "@/app/(app)/unitycatalog/(browser)/[catalog]/[schema]/[table]/lineage/lineage-graph"
import { getTableLineage } from "@/lib/dagster"

export default async function TableLineagePage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string; table: string }>
}) {
  const { catalog, schema, table } = await params
  const response = await getTableLineage([catalog, schema, table])
  const result = response.data.assetNodeOrError

  if (result.__typename !== "AssetNode") {
    return <pre>{JSON.stringify(response, null, 2)}</pre>
  }

  return <LineageGraph current={result} />
}
