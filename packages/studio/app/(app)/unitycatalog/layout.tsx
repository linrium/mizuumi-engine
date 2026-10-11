import { UnityCatalogNav } from "./unitycatalog-nav"

export default function UnityCatalogLayout({
  children,
}: LayoutProps<"/unitycatalog">) {
  return (
    <main className="flex h-dvh flex-col gap-4 p-6">
      <header>
        <h1 className="text-lg font-semibold">Unity Catalog</h1>
        <p className="text-xs text-muted-foreground">
          Browse catalogs, schemas, and tables.
        </p>
      </header>
      <UnityCatalogNav />
      {children}
    </main>
  )
}
