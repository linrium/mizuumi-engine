import { getTableLineage } from "@/lib/dagster"

export default async function TableLineagePage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string; table: string }>
}) {
  const { catalog, schema, table } = await params
  const response = await getTableLineage([catalog, schema, table])

  return <pre>{JSON.stringify(response, null, 2)}</pre>
}
