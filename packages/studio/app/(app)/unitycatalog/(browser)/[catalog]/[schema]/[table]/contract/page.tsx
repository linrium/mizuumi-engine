import { getDataContract } from "./actions"
import { DataContractView } from "./view"

export default async function DataContractPage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string; table: string }>
}) {
  const target = await params
  const result = await getDataContract(target)

  return (
    <DataContractView
      target={target}
      initialContract={result.ok ? result.data : null}
      initialError={result.ok ? undefined : result.error}
    />
  )
}
