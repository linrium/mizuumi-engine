"use client"

import { use } from "react"
import { PermissionsView } from "@/app/(app)/unitycatalog/(browser)/permissions-view"

export default function TablePermissionsPage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string; table: string }>
}) {
  const { catalog, schema, table } = use(params)
  const fullName = `${catalog}.${schema}.${table}`
  const callbackURL = `/unitycatalog/${encodeURIComponent(catalog)}/${encodeURIComponent(schema)}/${encodeURIComponent(table)}/permissions`
  return (
    <PermissionsView
      securableType="table"
      fullName={fullName}
      callbackURL={callbackURL}
    />
  )
}
