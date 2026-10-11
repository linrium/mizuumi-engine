"use client"

import { IconAlertCircle } from "@tabler/icons-react"
import { useCallback, useEffect, useState, useTransition } from "react"
import { ScrollArea } from "@/components/ui/scroll-area"
import { authClient } from "@/lib/auth-client"
import { getUnityCatalogInfo } from "@/app/(app)/unitycatalog/actions"
import type { Catalog, UnityCatalogData } from "@/app/(app)/unitycatalog/types"
import { BrowserContext } from "@/app/(app)/unitycatalog/(browser)/browser-context"
import { CatalogTree } from "@/app/(app)/unitycatalog/(browser)/catalog-tree"

export default function BrowserLayout({ children }: { children: React.ReactNode }) {
  const [catalogs, setCatalogs] = useState<Catalog[]>([])
  const [error, setError] = useState<string | null>(null)
  const [isPending, startTransition] = useTransition()

  const load = useCallback(() => {
    startTransition(async () => {
      const next = await getUnityCatalogInfo()
      if (next.needsReconnect) {
        await authClient.signIn.social({
          provider: "keycloak",
          callbackURL: "/unitycatalog",
        })
        return
      }
      if (next.error) {
        setError(next.error)
        setCatalogs([])
        return
      }
      setError(null)
      setCatalogs(
        (next.data as UnityCatalogData | undefined)?.catalogs ?? [],
      )
    })
  }, [])

  useEffect(() => {
    load()
  }, [load])

  return (
    <BrowserContext.Provider
      value={{ catalogs, isPending, error, refresh: load }}
    >
      {error ? (
        <div className="mx-4 flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive">
          <IconAlertCircle className="mt-0.5 size-4 shrink-0" />
          <div className="min-w-0">
            <div className="font-medium">Unable to load Unity Catalog</div>
            <div className="break-all opacity-80">{error}</div>
          </div>
        </div>
      ) : null}

      <div className="grid min-h-0 flex-1 grid-cols-[280px_1fr]">
        <div className="min-h-0 border-r">
          <CatalogTree />
        </div>
        <ScrollArea className="h-full min-h-0">
          <div className="p-5">{children}</div>
        </ScrollArea>
      </div>
    </BrowserContext.Provider>
  )
}
