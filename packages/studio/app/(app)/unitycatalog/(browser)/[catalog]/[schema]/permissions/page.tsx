"use client"

import { use } from "react"
import { PermissionsView } from "@/app/(app)/unitycatalog/(browser)/permissions-view"

export default function SchemaPermissionsPage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string }>
}) {
  const { catalog, schema } = use(params)
  const fullName = `${catalog}.${schema}`
  const callbackURL = `/unitycatalog/${encodeURIComponent(catalog)}/${encodeURIComponent(schema)}/permissions`
  return (
    <PermissionsView
      securableType="schema"
      fullName={fullName}
      callbackURL={callbackURL}
    />
  )
}
