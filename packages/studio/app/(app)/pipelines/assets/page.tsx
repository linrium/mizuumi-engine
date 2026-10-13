import { AssetsView } from "@/app/(app)/pipelines/assets/assets-view"
import { getAssets } from "@/lib/dagster"

export const dynamic = "force-dynamic"

export default async function PipelinesAssetsPage() {
  try {
    return <AssetsView initialAssets={await getAssets()} />
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error"
    return <AssetsView initialAssets={[]} initialError={message} />
  }
}
