"use server"

import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"
import { getTableLineage, type TableLineageNode } from "@/lib/dagster"

export type ExpandLineageResult = {
  node?: TableLineageNode
  error?: string
}

export async function expandLineageNode(
  assetKey: string[],
): Promise<ExpandLineageResult> {
  const session = await auth.api.getSession({ headers: await headers() })
  if (!session) redirect("/login")

  if (
    assetKey.length === 0 ||
    assetKey.length > 10 ||
    assetKey.some((part) => !part || part.length > 512)
  ) {
    return { error: "Invalid Dagster asset key." }
  }

  try {
    const response = await getTableLineage(assetKey)
    const result = response.data.assetNodeOrError

    if (result.__typename !== "AssetNode") {
      return { error: result.message }
    }

    return { node: result }
  } catch (error) {
    return {
      error:
        error instanceof Error
          ? error.message
          : "Unable to load neighboring assets.",
    }
  }
}
