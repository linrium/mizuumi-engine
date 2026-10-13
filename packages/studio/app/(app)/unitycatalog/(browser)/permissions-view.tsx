"use client"

import { IconPlus } from "@tabler/icons-react"
import { cn } from "cn"
import { useCallback, useEffect, useMemo, useState, useTransition } from "react"
import {
  getUnityCatalogPermissions,
  type PrivilegeAssignment,
  type SecurableType,
  updateUnityCatalogPermissions,
} from "@/app/(app)/unitycatalog/actions"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { authClient } from "@/lib/auth-client"

const PRIVILEGES_BY_TYPE: Record<SecurableType, string[]> = {
  catalog: [
    "BROWSE",
    "USE CATALOG",
    "CREATE SCHEMA",
    "USE SCHEMA",
    "CREATE TABLE",
    "SELECT",
    "MODIFY",
  ],
  schema: ["BROWSE", "USE SCHEMA", "CREATE TABLE", "SELECT", "MODIFY"],
  table: ["SELECT", "MODIFY"],
}

export function PermissionsView({
  securableType,
  fullName,
  callbackURL,
  tableVariant = "default",
}: {
  securableType: SecurableType
  fullName: string
  callbackURL: string
  tableVariant?: "default" | "columns"
}) {
  const privileges = PRIVILEGES_BY_TYPE[securableType]
  const useColumnsTableStyle = tableVariant === "columns"

  const [assignments, setAssignments] = useState<PrivilegeAssignment[]>([])
  const [message, setMessage] = useState<
    { kind: "error"; text: string } | { kind: "success"; text: string } | null
  >(null)
  const [grantOpen, setGrantOpen] = useState(false)
  const [principal, setPrincipal] = useState<string>("")
  const [checked, setChecked] = useState<Record<string, boolean>>({})
  const [isPending, startTransition] = useTransition()

  const permissionRows = useMemo(
    () =>
      assignments.flatMap((assignment) =>
        assignment.privileges.map((privilege) => ({
          principal: assignment.principal,
          privilege,
        })),
      ),
    [assignments],
  )

  const togglePrivilege = (p: string) =>
    setChecked((prev) => ({ ...prev, [p]: !prev[p] }))

  const selectedPrivileges = privileges.filter((p) => checked[p])

  const reconnect = useCallback(async () => {
    await authClient.signIn.social({
      provider: "keycloak",
      callbackURL,
    })
  }, [callbackURL])

  const loadAssignments = useCallback(() => {
    startTransition(async () => {
      const res = await getUnityCatalogPermissions(securableType, fullName)
      if (res.needsReconnect) {
        await reconnect()
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
  }, [securableType, fullName, reconnect])

  useEffect(() => {
    loadAssignments()
  }, [loadAssignments])

  const grant = () => {
    if (!principal || selectedPrivileges.length === 0) {
      setMessage({
        kind: "error",
        text: "Enter a principal and select at least one privilege.",
      })
      return
    }
    startTransition(async () => {
      const res = await updateUnityCatalogPermissions({
        securableType,
        fullName,
        principal,
        add: selectedPrivileges,
      })
      if (res.needsReconnect) {
        await reconnect()
        return
      }
      if (res.error) {
        setMessage({ kind: "error", text: res.error })
        return
      }
      setAssignments(res.data?.privilege_assignments ?? [])
      setMessage({
        kind: "success",
        text: `Granted ${selectedPrivileges.join(", ")} to ${principal}.`,
      })
      setChecked({})
      setPrincipal("")
      setGrantOpen(false)
    })
  }

  const revoke = (principal: string, privilege: string) => {
    startTransition(async () => {
      const res = await updateUnityCatalogPermissions({
        securableType,
        fullName,
        principal,
        remove: [privilege],
      })
      if (res.needsReconnect) {
        await reconnect()
        return
      }
      if (res.error) {
        setMessage({ kind: "error", text: res.error })
        return
      }
      setAssignments(res.data?.privilege_assignments ?? [])
      setMessage({
        kind: "success",
        text: `Revoked ${privilege} from ${principal}.`,
      })
    })
  }

  return (
    <div className="space-y-3">
      <div className="flex justify-end">
        <Button size="sm" onClick={() => setGrantOpen(true)}>
          <IconPlus />
          Grant
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

      <div
        className={cn(
          useColumnsTableStyle &&
            "overflow-hidden rounded-md ring-1 ring-foreground/10",
        )}
      >
        <Table>
          <TableHeader
            className={cn(
              useColumnsTableStyle && "bg-muted/50 text-muted-foreground",
            )}
          >
            <TableRow>
              <TableHead
                className={cn(
                  useColumnsTableStyle &&
                    "h-auto px-3 py-2 text-muted-foreground",
                )}
              >
                Securable
              </TableHead>
              <TableHead
                className={cn(
                  useColumnsTableStyle &&
                    "h-auto px-3 py-2 text-muted-foreground",
                )}
              >
                Principal
              </TableHead>
              <TableHead
                className={cn(
                  useColumnsTableStyle &&
                    "h-auto px-3 py-2 text-muted-foreground",
                )}
              >
                Privilege
              </TableHead>
              <TableHead
                className={cn(
                  "w-0",
                  useColumnsTableStyle && "h-auto px-3 py-2",
                )}
              />
            </TableRow>
          </TableHeader>
          <TableBody>
            {permissionRows.length === 0 ? (
              <TableRow className={cn(useColumnsTableStyle && "border-b-0")}>
                <TableCell
                  colSpan={4}
                  className={cn(
                    "py-6 text-center text-muted-foreground",
                    useColumnsTableStyle && "px-3",
                  )}
                >
                  {isPending ? "Loading…" : "No assignments."}
                </TableCell>
              </TableRow>
            ) : (
              permissionRows.map((permission, index) => (
                <TableRow
                  key={`${permission.principal}-${permission.privilege}`}
                  className={cn(
                    useColumnsTableStyle &&
                      "border-t border-b-0 hover:bg-muted/30",
                    useColumnsTableStyle && index % 2 === 1 && "bg-muted/20",
                  )}
                >
                  <TableCell
                    className={cn(
                      "font-mono",
                      useColumnsTableStyle && "px-3 py-1.5",
                    )}
                  >
                    {fullName}
                  </TableCell>
                  <TableCell
                    className={cn(
                      "font-mono",
                      useColumnsTableStyle && "px-3 py-1.5",
                    )}
                  >
                    {permission.principal}
                  </TableCell>
                  <TableCell
                    className={cn(useColumnsTableStyle && "px-3 py-1.5")}
                  >
                    <span className="font-mono">{permission.privilege}</span>
                  </TableCell>
                  <TableCell
                    className={cn(
                      "text-right",
                      useColumnsTableStyle && "px-3 py-1.5",
                    )}
                  >
                    <Button
                      size={useColumnsTableStyle ? "xs" : "sm"}
                      variant="outline"
                      disabled={isPending}
                      onClick={() =>
                        revoke(permission.principal, permission.privilege)
                      }
                    >
                      Revoke
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <Dialog
        open={grantOpen}
        onOpenChange={(open) => {
          setGrantOpen(open)
          if (!open) {
            setPrincipal("")
            setChecked({})
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Grant permissions</DialogTitle>
            <DialogDescription>
              Grant privileges on <span className="font-mono">{fullName}</span>{" "}
              to a principal.
            </DialogDescription>
          </DialogHeader>

          <div className="mt-4 space-y-4">
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
          </div>

          <DialogFooter>
            <Button
              size="sm"
              variant="outline"
              disabled={isPending}
              onClick={() => setGrantOpen(false)}
            >
              Cancel
            </Button>
            <Button size="sm" disabled={isPending} onClick={grant}>
              Grant
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
