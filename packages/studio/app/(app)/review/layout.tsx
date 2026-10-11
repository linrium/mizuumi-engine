import { ReviewNav } from "./review-nav"

export default function ReviewLayout({ children }: LayoutProps<"/review">) {
  return (
    <main className="flex w-full flex-col gap-4 px-6 py-6">
      <ReviewNav />
      {children}
    </main>
  )
}
