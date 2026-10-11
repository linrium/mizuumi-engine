import { Skeleton } from "@/components/ui/skeleton"

export default function PolicyTemplatesLoading() {
  return (
    <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_22rem]">
      <div className="space-y-3">
        {["first", "second", "third"].map((item) => (
          <Skeleton key={item} className="h-44 rounded-lg" />
        ))}
      </div>
      <Skeleton className="h-[36rem] rounded-lg" />
    </div>
  )
}
