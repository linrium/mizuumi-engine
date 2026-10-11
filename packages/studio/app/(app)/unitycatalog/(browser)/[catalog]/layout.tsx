"use client"

import { IconDatabase } from "@tabler/icons-react"
import { usePathname } from "next/navigation"
import { use } from "react"
import { useBrowser } from "@/app/(app)/unitycatalog/(browser)/browser-context"
import { HeaderBlock, NotFound } from "@/app/(app)/unitycatalog/(browser)/detail"
import { CatalogNav } from "@/app/(app)/unitycatalog/(browser)/[catalog]/catalog-nav"

export default function CatalogLayout({
  params,
  children,
}: {
  params: Promise<{ catalog: string }>
  children: React.ReactNode
}) {
  const { catalog: catalogParam } = use(params)
  const { catalogs, isPending } = useBrowser()
  const catalog = catalogs.find((c) => c.name === catalogParam)
  const pathname = usePathname()
  const base = `/unitycatalog/${encodeURIComponent(catalogParam)}`
  const isCatalogLevel =
    pathname === base || pathname === `${base}/permissions`

  if (!catalog) {
    return (
      <NotFound
        message={
          isPending ? "Loading…" : `Catalog "${catalogParam}" not found.`
        }
      />
    )
  }

  if (!isCatalogLevel) {
    return <>{children}</>
  }

  return (
    <div className="space-y-3">
      <HeaderBlock
        icon={<IconDatabase className="size-4" />}
        kind="Catalog"
        title={catalog.name}
        subtitle={catalog.comment ?? undefined}
      />
      <CatalogNav catalog={catalog.name} />
      {children}
    </div>
  )
}
