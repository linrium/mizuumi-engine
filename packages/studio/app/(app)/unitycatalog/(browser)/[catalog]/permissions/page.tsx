"use client"

import { use } from "react"
import { PermissionsView } from "@/app/(app)/unitycatalog/(browser)/permissions-view"

export default function CatalogPermissionsPage({
  params,
}: {
  params: Promise<{ catalog: string }>
}) {
  const { catalog } = use(params)
  const callbackURL = `/unitycatalog/${encodeURIComponent(catalog)}/permissions`
  return (
    <PermissionsView
      securableType="catalog"
      fullName={catalog}
      callbackURL={callbackURL}
      tableVariant="columns"
    />
  )
}
