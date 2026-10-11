import { PipelinesNav } from "./pipelines-nav"

export default function PipelinesLayout({
  children,
}: LayoutProps<"/pipelines">) {
  return (
    <main className="flex w-full flex-col gap-4 px-6 py-6">
      <PipelinesNav />
      {children}
    </main>
  )
}
