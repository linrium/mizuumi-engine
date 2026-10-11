"use client"

import { IconRefresh } from "@tabler/icons-react"
import { cn } from "cn"
import { useCallback, useEffect, useMemo, useState, useTransition } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import { authClient } from "@/lib/auth-client"
import {
  getUnityCatalogAllAssignments,
  revokeAllPrivileges,
  type SecurableAssignments,
} from "../actions"

export function AssignmentsView() {
  const [assignments, setAssignments] = useState<SecurableAssignments[]>([])
  const [error, setError] = useState<string | null>(null)
  const [loaded, setLoaded] = useState(false)
  const [filter, setFilter] = useState("")
  const [isPending, startTransition] = useTransition()
  const [deletingKey, setDeletingKey] = useState<string | null>(null)

  const load = useCallback(() => {
    startTransition(async () => {
      const res = await getUnityCatalogAllAssignments()
      if (res.needsReconnect) {
        await authClient.signIn.social({
          provider: "keycloak",
          callbackURL: "/unitycatalog/assignments",
        })
        return
      }
      if (res.error) {
        setError(res.error)
        setAssignments([])
        setLoaded(true)
        return
      }
      setError(null)
      setAssignments(res.data?.assignments ?? [])
      setLoaded(true)
    })
  }, [])

  useEffect(() => {
    load()
  }, [load])

  const removeAssignment = useCallback(
    (securableType: string, fullName: string, principal: string) => {
      const group = assignments.find(
        (a) => a.securable_type === securableType && a.full_name === fullName,
      )
      const pa = group?.privilege_assignments.find(
        (p) => p.principal === principal,
      )
      if (!pa || pa.privileges.length === 0) return
      if (
        !window.confirm(
          `Revoke ${pa.privileges.join(", ")} from ${principal} on ${securableType} ${fullName}?`,
        )
      )
        return
      const key = `${securableType}:${fullName}:${principal}`
      setDeletingKey(key)
      startTransition(async () => {
        const res = await revokeAllPrivileges({
          securableType,
          fullName,
          principal,
          privileges: pa.privileges,
        })
        setDeletingKey(null)
        if (res.needsReconnect) {
          await authClient.signIn.social({
            provider: "keycloak",
            callbackURL: "/unitycatalog/assignments",
          })
          return
        }
        if (res.error) {
          setError(res.error)
          return
        }
        setError(null)
        setAssignments((prev) =>
          prev
            .map((group) => {
              if (
                group.securable_type !== securableType ||
                group.full_name !== fullName
              )
                return group
              return {
                ...group,
                privilege_assignments: group.privilege_assignments.filter(
                  (p) => p.principal !== principal,
                ),
              }
            })
            .filter((group) => group.privilege_assignments.length > 0),
        )
      })
    },
    [assignments],
  )

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase()
    if (!q) return assignments
    return assignments.filter((a) => {
      if (
        a.full_name.toLowerCase().includes(q) ||
        a.securable_type.toLowerCase().includes(q)
      )
        return true
      return a.privilege_assignments.some(
        (pa) =>
          pa.principal.toLowerCase().includes(q) ||
          pa.privileges.some((p) => p.toLowerCase().includes(q)),
      )
    })
  }, [assignments, filter])

  const totalPrincipals = assignments.reduce(
    (n, a) => n + a.privilege_assignments.length,
    0,
  )

  return (
    <Card className="min-h-0 flex-1 overflow-hidden p-0">
      <CardHeader className="flex flex-row items-start justify-between gap-3 border-b py-3">
        <div>
          <CardTitle className="text-sm">All assignments</CardTitle>
          <CardDescription>
            {assignments.length} securable
            {assignments.length === 1 ? "" : "s"} · {totalPrincipals} principal
            {totalPrincipals === 1 ? "" : "s"}
          </CardDescription>
        </div>
        <div className="flex items-center gap-2">
          <Input
            placeholder="Filter by name, principal, or privilege"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="h-8 w-72"
          />
          <Button
            size="sm"
            variant="outline"
            disabled={isPending}
            onClick={load}
          >
            <IconRefresh className={cn(isPending && "animate-spin")} />
            {isPending ? "Loading" : "Refresh"}
          </Button>
        </div>
      </CardHeader>
      <ScrollArea className="h-full">
        <div className="p-6">
          {error ? (
            <div className="rounded border border-destructive/40 px-3 py-2 text-xs text-destructive">
              {error}
            </div>
          ) : !loaded && isPending ? (
            <div className="py-10 text-center text-xs text-muted-foreground">
              Loading assignments…
            </div>
          ) : filtered.length === 0 ? (
            <div className="py-10 text-center text-xs text-muted-foreground">
              {assignments.length === 0
                ? "No permissions assigned."
                : "No assignments match the filter."}
            </div>
          ) : (
            <div className="overflow-hidden rounded-md ring-1 ring-foreground/10">
              <table className="w-full text-xs">
                <thead className="bg-muted/50 text-muted-foreground">
                  <tr>
                    <th className="px-3 py-2 text-left font-medium">Type</th>
                    <th className="px-3 py-2 text-left font-medium">
                      Securable
                    </th>
                    <th className="px-3 py-2 text-left font-medium">
                      Principal
                    </th>
                    <th className="px-3 py-2 text-left font-medium">
                      Privileges
                    </th>
                    <th className="px-3 py-2 text-right font-medium">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.flatMap((group) =>
                    group.privilege_assignments.map((pa, i) => (
                      <tr
                        key={`${group.securable_type}:${group.full_name}:${pa.principal}`}
                        className={cn("border-t", i % 2 === 1 && "bg-muted/20")}
                      >
                        <td className="px-3 py-1.5">
                          <Badge variant="outline">
                            {group.securable_type}
                          </Badge>
                        </td>
                        <td className="px-3 py-1.5 font-mono break-all">
                          {group.full_name}
                        </td>
                        <td className="px-3 py-1.5 font-mono break-all">
                          {pa.principal}
                        </td>
                        <td className="px-3 py-1.5">
                          <div className="flex flex-wrap gap-1">
                            {pa.privileges.map((p) => (
                              <Badge key={p} variant="secondary">
                                {p}
                              </Badge>
                            ))}
                          </div>
                        </td>
                        <td className="px-3 py-1.5 text-right">
                          <Button
                            size="sm"
                            variant="ghost"
                            disabled={
                              deletingKey ===
                              `${group.securable_type}:${group.full_name}:${pa.principal}`
                            }
                            onClick={() =>
                              removeAssignment(
                                group.securable_type,
                                group.full_name,
                                pa.principal,
                              )
                            }
                          >
                            {deletingKey ===
                            `${group.securable_type}:${group.full_name}:${pa.principal}`
                              ? "Deleting…"
                              : "Delete"}
                          </Button>
                        </td>
                      </tr>
                    )),
                  )}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </ScrollArea>
    </Card>
  )
}
