import { UnityCatalogNav } from "@/app/(app)/unitycatalog/unitycatalog-nav"

export default function UnityCatalogLayout({
  children,
}: LayoutProps<"/unitycatalog">) {
  return (
    <main className="flex h-dvh flex-col">
      <UnityCatalogNav />
      {children}
    </main>
  )
}
