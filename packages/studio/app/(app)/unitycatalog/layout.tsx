export default function UnityCatalogLayout({
  children,
}: LayoutProps<"/unitycatalog">) {
  return (
    <main className="flex h-dvh flex-col">
      {children}
    </main>
  )
}
