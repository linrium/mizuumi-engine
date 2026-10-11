import { cn } from "cn"
import { Separator } from "@/components/ui/separator"

export function HeaderBlock({
  icon,
  kind,
  title,
  subtitle,
  badges,
}: {
  icon: React.ReactNode
  kind: string
  title: string
  subtitle?: string
  badges?: React.ReactNode
}) {
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 text-xs text-muted-foreground">
        {icon}
        <span>{kind}</span>
      </div>
      <h2 className="font-mono text-base font-semibold break-all">{title}</h2>
      {subtitle ? (
        <p className="text-xs text-muted-foreground">{subtitle}</p>
      ) : null}
      {badges ? <div className="flex flex-wrap gap-1.5">{badges}</div> : null}
      <Separator />
    </div>
  )
}

export function MetaGrid({ items }: { items: Array<[string, string]> }) {
  return (
    <dl className="grid grid-cols-1 gap-x-6 gap-y-2 text-xs sm:grid-cols-2">
      {items.map(([label, value]) => (
        <div key={label} className="min-w-0">
          <dt className="text-muted-foreground">{label}</dt>
          <dd className="truncate font-mono">{value}</dd>
        </div>
      ))}
    </dl>
  )
}

export function NotFound({ message }: { message: string }) {
  return (
    <div
      className={cn(
        "flex h-full min-h-[200px] items-center justify-center text-center text-xs text-muted-foreground",
      )}
    >
      {message}
    </div>
  )
}
