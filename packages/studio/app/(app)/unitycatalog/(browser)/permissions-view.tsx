"use client"

import { IconPlus } from "@tabler/icons-react"
import { cn } from "cn"
import { useCallback, useEffect, useMemo, useState, useTransition } from "react"
import { useForm } from "react-hook-form"
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

type GrantPermissionsForm = {
  principal: string
  privileges: string[]
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
  const [isPending, startTransition] = useTransition()
  const {
    register,
    handleSubmit,
    reset,
    setError,
    formState: { errors },
  } = useForm<GrantPermissionsForm>({
    defaultValues: {
      principal: "",
      privileges: [],
    },
  })

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

  const grant = (values: GrantPermissionsForm) => {
    const principal = values.principal.trim()

    startTransition(async () => {
      const res = await updateUnityCatalogPermissions({
        securableType,
        fullName,
        principal,
        add: values.privileges,
      })
      if (res.needsReconnect) {
        await reconnect()
        return
      }
      if (res.error) {
        setError("root.server", { message: res.error })
        return
      }
      setAssignments(res.data?.privilege_assignments ?? [])
      setMessage({
        kind: "success",
        text: `Granted ${values.privileges.join(", ")} to ${principal}.`,
      })
      reset()
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
            reset()
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

          <form onSubmit={handleSubmit(grant)}>
            <div className="mt-4 space-y-4">
              <div className="space-y-1">
                <label className="text-xs font-medium" htmlFor="principal">
                  Principal (user, service principal, or group)
                </label>
                <Input
                  id="principal"
                  placeholder="user@example.com"
                  aria-invalid={!!errors.principal}
                  aria-describedby={
                    errors.principal ? "principal-error" : undefined
                  }
                  {...register("principal", {
                    validate: (value) =>
                      value.trim().length > 0 || "Principal is required.",
                  })}
                />
                {errors.principal ? (
                  <p
                    id="principal-error"
                    className="text-xs text-destructive"
                    role="alert"
                  >
                    {errors.principal.message}
                  </p>
                ) : null}
              </div>

              <fieldset className="space-y-1.5">
                <legend className="text-xs font-medium">Privileges</legend>
                <div className="grid grid-cols-2 gap-1.5">
                  {privileges.map((p) => (
                    <label
                      key={p}
                      className="flex items-center gap-2 rounded border px-2 py-1.5 text-xs hover:bg-muted"
                    >
                      <input
                        type="checkbox"
                        value={p}
                        className="size-3.5"
                        aria-invalid={!!errors.privileges}
                        aria-describedby={
                          errors.privileges ? "privileges-error" : undefined
                        }
                        {...register("privileges", {
                          validate: (value) =>
                            value.length > 0 ||
                            "Select at least one privilege.",
                        })}
                      />
                      <span className="font-mono">{p}</span>
                    </label>
                  ))}
                </div>
                {errors.privileges ? (
                  <p
                    id="privileges-error"
                    className="text-xs text-destructive"
                    role="alert"
                  >
                    {errors.privileges.message}
                  </p>
                ) : null}
              </fieldset>

              {errors.root?.server ? (
                <div
                  className="rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive"
                  role="alert"
                >
                  {errors.root.server.message}
                </div>
              ) : null}
            </div>

            <DialogFooter>
              <Button
                type="button"
                size="sm"
                variant="outline"
                disabled={isPending}
                onClick={() => setGrantOpen(false)}
              >
                Cancel
              </Button>
              <Button type="submit" size="sm" disabled={isPending}>
                Grant
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}
