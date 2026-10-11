"use client"

import { usePathname, useRouter } from "next/navigation"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"

const items = [
  { href: "/review/requests", label: "Requests" },
  { href: "/review/policy-templates", label: "Policy templates" },
]

export function ReviewNav() {
  const router = useRouter()
  const pathname = usePathname()
  const active =
    items.find(
      (item) => pathname === item.href || pathname.startsWith(`${item.href}/`),
    )?.href ?? items[0].href

  return (
    <Tabs value={active} onValueChange={(value) => router.push(value)}>
      <TabsList>
        {items.map((item) => (
          <TabsTrigger key={item.href} value={item.href}>
            {item.label}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  )
}
