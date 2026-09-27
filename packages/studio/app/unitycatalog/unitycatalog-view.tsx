"use client"

import { useCallback, useEffect, useState, useTransition } from "react"
import { getUnityCatalogInfo } from "./actions"

type UnityCatalogResult = Awaited<ReturnType<typeof getUnityCatalogInfo>>

export function UnityCatalogView() {
  const [result, setResult] = useState<UnityCatalogResult | null>(null)
  const [isPending, startTransition] = useTransition()

  const load = useCallback(() => {
    startTransition(async () => {
      setResult(await getUnityCatalogInfo())
    })
  }, [])

  useEffect(() => {
    load()
  }, [load])

  return (
    <main className="space-y-4 p-6">
      <h1 className="text-lg font-semibold">Unity Catalog</h1>
      <button
        className="rounded border px-3 py-1 text-sm"
        disabled={isPending}
        onClick={load}
        type="button"
      >
        {isPending ? "Loading…" : "Refresh"}
      </button>
      <pre className="overflow-auto rounded border p-4 text-xs">
        {JSON.stringify(
          result ?? { status: "Loading Unity Catalog metadata…" },
          null,
          2,
        )}
      </pre>
    </main>
  )
}
