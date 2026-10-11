"use client"

import { IconRefresh } from "@tabler/icons-react"
import { cn } from "cn"
import { useCallback, useEffect, useMemo, useState, useTransition } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { authClient } from "@/lib/auth-client"
import {
  getUnityCatalogInfo,
  getUnityCatalogPermissions,
  type PrivilegeAssignment,
  type SecurableType,
  updateUnityCatalogPermissions,
} from "../actions"
import type { Catalog, UnityCatalogData } from "../types"

const PRIVILEGES_BY_TYPE: Record<SecurableType, string[]> = {
  catalog: [
    "USE CATALOG",
    "CREATE SCHEMA",
    "USE SCHEMA",
    "CREATE TABLE",
    "SELECT",
    "MODIFY",
  ],
  schema: ["USE SCHEMA", "CREATE TABLE", "SELECT", "MODIFY"],
  table: ["SELECT", "MODIFY"],
}

export function PermissionsView() {
  const [catalogs, setCatalogs] = useState<Catalog[]>([])
  const [securableType, setSecurableType] = useState<SecurableType>("catalog")
  const [fullName, setFullName] = useState<string>("")
  const [principal, setPrincipal] = useState<string>("")
  const [checked, setChecked] = useState<Record<string, boolean>>({})
  const [assignments, setAssignments] = useState<PrivilegeAssignment[]>([])
  const [message, setMessage] = useState<
    { kind: "error"; text: string } | { kind: "success"; text: string } | null
  >(null)
  const [isPending, startTransition] = useTransition()

  useEffect(() => {
    startTransition(async () => {
      const next = await getUnityCatalogInfo()
      if (next.needsReconnect) {
        await authClient.signIn.social({
          provider: "keycloak",
          callbackURL: "/unitycatalog/permissions",
        })
        return
      }
      if (next.error) return
      setCatalogs(
        (next.data as UnityCatalogData | undefined)?.catalogs ?? [],
      )
    })
  }, [])

  const availableTargets = useMemo(() => {
    const items: Array<{ value: string; label: string }> = []
    for (const c of catalogs) {
      if (securableType === "catalog") {
        items.push({ value: c.name, label: c.name })
      } else if (securableType === "schema") {
        for (const s of c.schemas ?? []) {
          const full = s.full_name ?? `${c.name}.${s.name}`
          items.push({ value: full, label: full })
        }
      } else {
        for (const s of c.schemas ?? []) {
          for (const t of s.tables ?? []) {
            const full = `${c.name}.${s.name}.${t.name}`
            items.push({ value: full, label: full })
          }
        }
      }
    }
    return items
  }, [catalogs, securableType])

  const privileges = PRIVILEGES_BY_TYPE[securableType]

  const togglePrivilege = (p: string) =>
    setChecked((prev) => ({ ...prev, [p]: !prev[p] }))

  const selectedPrivileges = privileges.filter((p) => checked[p])

  const loadAssignments = useCallback(() => {
    if (!fullName) return
    startTransition(async () => {
      const res = await getUnityCatalogPermissions(securableType, fullName)
      if (res.needsReconnect) {
        await authClient.signIn.social({
          provider: "keycloak",
          callbackURL: "/unitycatalog/permissions",
        })
        return
      }
      if (res.error) {
        setAssignments([])
        setMessage({ kind: "error", text: res.error })
        return
      }
      setAssignments(res.data?.privilege_assignments ?? [])
      setMessage(null)
    })
  }, [fullName, securableType])

  const submit = (action: "grant" | "revoke") => {
    if (!fullName || !principal || selectedPrivileges.length === 0) {
      setMessage({
        kind: "error",
        text: "Select a securable, principal, and at least one privilege.",
      })
      return
    }
    startTransition(async () => {
      const res = await updateUnityCatalogPermissions({
        securableType,
        fullName,
        principal,
        add: action === "grant" ? selectedPrivileges : [],
        remove: action === "revoke" ? selectedPrivileges : [],
      })
      if (res.needsReconnect) {
        await authClient.signIn.social({
          provider: "keycloak",
          callbackURL: "/unitycatalog/permissions",
        })
        return
      }
      if (res.error) {
        setMessage({ kind: "error", text: res.error })
        return
      }
      setAssignments(res.data?.privilege_assignments ?? [])
      setMessage({
        kind: "success",
        text: `${action === "grant" ? "Granted" : "Revoked"} ${selectedPrivileges.join(", ")} ${action === "grant" ? "to" : "from"} ${principal}.`,
      })
      setChecked({})
    })
  }

  return (
    <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 lg:grid-cols-[1fr_1fr]">
      <Card className="min-h-0 overflow-hidden p-0">
        <CardHeader className="border-b py-3">
          <CardTitle className="text-sm">Grant permissions</CardTitle>
          <CardDescription>
            Grant or revoke Unity Catalog privileges on a securable for a
            principal.
          </CardDescription>
        </CardHeader>
        <ScrollArea className="h-full">
          <CardContent className="space-y-4 p-6">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <label className="text-xs font-medium" htmlFor="securable-type">
                  Securable type
                </label>
                <Select
                  value={securableType}
                  onValueChange={(v) => {
                    if (!v) return
                    setSecurableType(v as SecurableType)
                    setFullName("")
                    setChecked({})
                  }}
                >
                  <SelectTrigger className="h-9 w-full" id="securable-type">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem value="catalog">Catalog</SelectItem>
                      <SelectItem value="schema">Schema</SelectItem>
                      <SelectItem value="table">Table</SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <label className="text-xs font-medium" htmlFor="full-name">
                  Securable
                </label>
                {availableTargets.length > 0 ? (
                  <Select
                    value={fullName}
                    onValueChange={(v) => v && setFullName(v)}
                  >
                    <SelectTrigger className="h-9 w-full" id="full-name">
                      <SelectValue placeholder="Select…" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectGroup>
                        {availableTargets.map((o) => (
                          <SelectItem key={o.value} value={o.value}>
                            {o.label}
                          </SelectItem>
                        ))}
                      </SelectGroup>
                    </SelectContent>
                  </Select>
                ) : (
                  <Input
                    id="full-name"
                    placeholder={
                      securableType === "catalog"
                        ? "catalog_name"
                        : securableType === "schema"
                          ? "catalog.schema"
                          : "catalog.schema.table"
                    }
                    value={fullName}
                    onChange={(e) => setFullName(e.target.value)}
                  />
                )}
              </div>
            </div>

            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="principal">
                Principal (user, service principal, or group)
              </label>
              <Input
                id="principal"
                placeholder="user@example.com"
                value={principal}
                onChange={(e) => setPrincipal(e.target.value)}
              />
            </div>

            <div className="space-y-1.5">
              <div className="text-xs font-medium">Privileges</div>
              <div className="grid grid-cols-2 gap-1.5">
                {privileges.map((p) => (
                  <label
                    key={p}
                    className="flex items-center gap-2 rounded border px-2 py-1.5 text-xs hover:bg-muted"
                  >
                    <input
                      type="checkbox"
                      className="size-3.5"
                      checked={!!checked[p]}
                      onChange={() => togglePrivilege(p)}
                    />
                    <span className="font-mono">{p}</span>
                  </label>
                ))}
              </div>
            </div>

            <div className="flex flex-wrap gap-2">
              <Button
                size="sm"
                disabled={isPending}
                onClick={() => submit("grant")}
              >
                Grant
              </Button>
              <Button
                size="sm"
                variant="outline"
                disabled={isPending}
                onClick={() => submit("revoke")}
              >
                Revoke
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={isPending || !fullName}
                onClick={loadAssignments}
              >
                <IconRefresh className={cn(isPending && "animate-spin")} />
                Load current
              </Button>
            </div>

            {message ? (
              <div
                className={cn(
                  "rounded border px-3 py-2 text-xs",
                  message.kind === "error"
                    ? "border-destructive/40 text-destructive"
                    : "border-emerald-500/40 text-emerald-700 dark:text-emerald-400",
                )}
              >
                {message.text}
              </div>
            ) : null}
          </CardContent>
        </ScrollArea>
      </Card>

      <Card className="min-h-0 overflow-hidden p-0">
        <CardHeader className="border-b py-3">
          <CardTitle className="text-sm">Current assignments</CardTitle>
          <CardDescription>
            {fullName
              ? `${securableType}: ${fullName}`
              : "No securable selected"}
          </CardDescription>
        </CardHeader>
        <ScrollArea className="h-full">
          <div className="p-6">
            {assignments.length === 0 ? (
              <div className="py-6 text-center text-xs text-muted-foreground">
                No assignments loaded. Pick a securable and click "Load
                current".
              </div>
            ) : (
              <ul className="space-y-2">
                {assignments.map((a) => (
                  <li
                    key={a.principal}
                    className="rounded-md ring-1 ring-foreground/10 p-3"
                  >
                    <div className="font-mono text-xs">{a.principal}</div>
                    <div className="mt-1.5 flex flex-wrap gap-1">
                      {a.privileges.map((p) => (
                        <Badge key={p} variant="secondary">
                          {p}
                        </Badge>
                      ))}
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </ScrollArea>
      </Card>
    </div>
  )
}
