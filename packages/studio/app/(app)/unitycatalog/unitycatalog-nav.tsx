"use client"

import { cn } from "cn"
import Link from "next/link"
import { usePathname } from "next/navigation"

const tabs = [{ href: "/unitycatalog", label: "Browser" }] as const

export function UnityCatalogNav() {
  const pathname = usePathname()
  return (
    <div className="flex gap-1 border-b">
      {tabs.map((t) => {
        const active =
          t.href === "/unitycatalog"
            ? true
            : pathname === t.href || pathname.startsWith(`${t.href}/`)
        return (
          <Link
            key={t.href}
            href={t.href}
            className={cn(
              "border-b-2 px-3 py-1.5 text-xs font-medium transition-colors",
              active
                ? "border-foreground"
                : "border-transparent text-muted-foreground hover:text-foreground",
            )}
          >
            {t.label}
          </Link>
        )
      })}
    </div>
  )
}
