"use client"

import { IconAlertCircle, IconRefresh } from "@tabler/icons-react"
import { cn } from "cn"
import { useCallback, useEffect, useState, useTransition } from "react"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"
import { authClient } from "@/lib/auth-client"
import { getUnityCatalogInfo } from "../actions"
import type { Catalog, UnityCatalogData } from "../types"
import { BrowserContext } from "./browser-context"
import { CatalogTree } from "./catalog-tree"

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
      <div className="flex justify-end">
        <Button size="sm" variant="outline" disabled={isPending} onClick={load}>
          <IconRefresh className={cn(isPending && "animate-spin")} />
          {isPending ? "Loading" : "Refresh"}
        </Button>
      </div>

      {error ? (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <IconAlertCircle className="size-4" />
              Unable to load Unity Catalog
            </CardTitle>
            <CardDescription className="break-all">{error}</CardDescription>
          </CardHeader>
        </Card>
      ) : null}

      <div className="grid min-h-0 flex-1 grid-cols-[320px_1fr] gap-4">
        <CatalogTree />
        <Card className="min-h-0 overflow-hidden p-0">
          <ScrollArea className="h-full">
            <div className="p-6">{children}</div>
          </ScrollArea>
        </Card>
      </div>
    </BrowserContext.Provider>
  )
}
