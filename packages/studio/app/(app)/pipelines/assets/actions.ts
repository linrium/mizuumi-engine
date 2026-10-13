"use server"

import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"
import {
  DagsterInvalidSubsetError,
  getAssets,
  launchAssetMaterialization,
} from "@/lib/dagster"

export async function materializeAsset(
  assetKey: string[],
  repositoryName: string,
  repositoryLocationName: string,
) {
  const session = await auth.api.getSession({ headers: await headers() })
  if (!session) redirect("/login")

  const assets = await getAssets()
  const asset = assets.find(
    (candidate) =>
      candidate.repository.name === repositoryName &&
      candidate.repository.location.name === repositoryLocationName &&
      candidate.assetKey.path.length === assetKey.length &&
      candidate.assetKey.path.every((part, index) => part === assetKey[index]),
  )

  if (!asset) throw new Error("Asset not found")
  if (!asset.isMaterializable) throw new Error("Asset is not materializable")

  const jobName = asset.jobNames[0]
  if (!jobName) throw new Error("Asset has no materialization job")

  const launch = (assetKeys: string[][]) =>
    launchAssetMaterialization({
      assetKeys,
      jobName,
      repositoryName: asset.repository.name,
      repositoryLocationName: asset.repository.location.name,
    })

  try {
    return await launch([asset.assetKey.path])
  } catch (error) {
    if (!(error instanceof DagsterInvalidSubsetError) || !asset.opName) {
      throw error
    }

    const requiredAssetKeys = assets
      .filter(
        (candidate) =>
          candidate.isMaterializable &&
          candidate.opName === asset.opName &&
          candidate.repository.name === asset.repository.name &&
          candidate.repository.location.name ===
            asset.repository.location.name &&
          candidate.jobNames.includes(jobName),
      )
      .map((candidate) => candidate.assetKey.path)

    if (requiredAssetKeys.length < 2) throw error
    return launch(requiredAssetKeys)
  }
}
