import { Skeleton } from "@/components/ui/skeleton"

export default function RequestQueueLoading() {
  return (
    <div className="space-y-4">
      <div className="grid gap-3 sm:grid-cols-3">
        {["requests", "teams", "oldest"].map((item) => (
          <Skeleton key={item} className="h-24 rounded-lg" />
        ))}
      </div>
      <Skeleton className="h-80 rounded-lg" />
    </div>
  )
}
