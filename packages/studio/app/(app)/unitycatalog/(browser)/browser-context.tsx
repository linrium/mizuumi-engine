"use client"

import { createContext, useContext } from "react"
import type { Catalog } from "../types"

type BrowserContextValue = {
  catalogs: Catalog[]
  isPending: boolean
  error: string | null
  refresh: () => void
}

export const BrowserContext = createContext<BrowserContextValue | null>(null)

export function useBrowser() {
  const ctx = useContext(BrowserContext)
  if (!ctx) throw new Error("useBrowser must be used inside BrowserContext")
  return ctx
}
