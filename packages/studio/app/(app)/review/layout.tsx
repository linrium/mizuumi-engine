import { IconGitPullRequest } from "@tabler/icons-react"
import { ReviewNav } from "./review-nav"

export default function ReviewLayout({ children }: LayoutProps<"/review">) {
  return (
    <main className="mx-auto flex w-full max-w-7xl flex-col gap-6 px-6 py-8">
      <header className="flex flex-col gap-4 border-b pb-5 sm:flex-row sm:items-end sm:justify-between">
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-xs font-medium text-primary">
            <IconGitPullRequest className="size-4" />
            Access governance
          </div>
          <div>
            <h1 className="text-2xl font-semibold tracking-tight">Review</h1>
            <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
              Route access through reusable policies and accountable approvals.
            </p>
          </div>
        </div>
        <ReviewNav />
      </header>
      {children}
    </main>
  )
}
