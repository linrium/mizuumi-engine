"use client"

import { IconClipboardCheck, IconTemplate } from "@tabler/icons-react"
import Link from "next/link"
import { usePathname } from "next/navigation"
import { cn } from "@/lib/utils"

const items = [
  {
    href: "/review/requests",
    label: "Request queue",
    icon: IconClipboardCheck,
  },
  {
    href: "/review/policy-templates",
    label: "Policy templates",
    icon: IconTemplate,
  },
]

export function ReviewNav() {
  const pathname = usePathname()

  return (
    <nav className="flex gap-1" aria-label="Review navigation">
      {items.map((item) => {
        const Icon = item.icon
        const active =
          pathname === item.href || pathname.startsWith(`${item.href}/`)
        return (
          <Link
            key={item.href}
            href={item.href}
            className={cn(
              "flex items-center gap-2 rounded-md px-3 py-2 text-xs font-medium transition-colors",
              active
                ? "bg-primary text-primary-foreground"
                : "text-muted-foreground hover:bg-muted hover:text-foreground",
            )}
          >
            <Icon className="size-4" />
            {item.label}
          </Link>
        )
      })}
    </nav>
  )
}
