import { IconBuilding, IconFolder, IconId, IconUser } from "@tabler/icons-react"
import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { auth } from "@/lib/auth"

type OrganizationMembership = {
  alias: string
  id?: string
  groups: string[]
}

type UserInfo = {
  organization?: unknown
}

function parseOrganizationMemberships(
  claim: unknown,
): OrganizationMembership[] {
  if (!claim || typeof claim !== "object" || Array.isArray(claim)) return []

  return Object.entries(claim)
    .map(([alias, value]) => {
      if (!value || typeof value !== "object" || Array.isArray(value)) {
        return { alias, groups: [] }
      }

      const membership = value as Record<string, unknown>
      return {
        alias,
        id: typeof membership.id === "string" ? membership.id : undefined,
        groups: Array.isArray(membership.groups)
          ? membership.groups.filter(
              (group): group is string => typeof group === "string",
            )
          : [],
      }
    })
    .sort((left, right) => left.alias.localeCompare(right.alias))
}

async function loadOrganizationMemberships(): Promise<{
  organizations: OrganizationMembership[]
  error?: string
}> {
  const requestHeaders = await headers()
  const session = await auth.api.getSession({ headers: requestHeaders })
  if (!session) redirect("/login")

  let accessToken: string
  try {
    ;({ accessToken } = await auth.api.getAccessToken({
      headers: requestHeaders,
      body: { useAccountCookie: true, userId: session.user.id },
    }))
  } catch {
    return {
      organizations: [],
      error: "Reconnect your Keycloak account to refresh organization access.",
    }
  }

  const issuer = (
    process.env.KEYCLOAK_ISSUER ?? "https://auth.mizuumi.test/realms/sovico"
  ).replace(/\/$/, "")
  const response = await fetch(`${issuer}/protocol/openid-connect/userinfo`, {
    headers: { Authorization: `Bearer ${accessToken}` },
    cache: "no-store",
  }).catch(() => null)

  if (!response) {
    return {
      organizations: [],
      error: "Keycloak is unavailable. Check the issuer and TLS configuration.",
    }
  }
  if (!response.ok) {
    return {
      organizations: [],
      error: `Keycloak userinfo returned HTTP ${response.status}.`,
    }
  }

  const userInfo = (await response.json()) as UserInfo
  return {
    organizations: parseOrganizationMemberships(userInfo.organization),
  }
}

export default async function GroupsPage() {
  const requestHeaders = await headers()
  const session = await auth.api.getSession({ headers: requestHeaders })
  if (!session) redirect("/login")

  const { organizations, error } = await loadOrganizationMemberships()
  const groupCount = organizations.reduce(
    (count, organization) => count + organization.groups.length,
    0,
  )

  return (
    <main className="mx-auto flex w-full max-w-6xl flex-col gap-6 px-6 py-10">
      <section className="flex flex-col gap-2">
        <Badge variant="secondary" className="w-fit">
          Keycloak
        </Badge>
        <h1 className="text-2xl font-semibold tracking-tight">
          Your organizations and groups
        </h1>
        <p className="max-w-2xl text-sm text-muted-foreground">
          Memberships are read from your signed-in Keycloak account. Only your
          own organization context is visible here.
        </p>
      </section>

      <section className="grid gap-4 sm:grid-cols-3">
        <Card size="sm">
          <CardHeader>
            <CardDescription>Organizations</CardDescription>
            <CardTitle className="flex items-center gap-2 text-xl">
              <IconBuilding className="size-4 text-muted-foreground" />
              {organizations.length}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card size="sm">
          <CardHeader>
            <CardDescription>Groups</CardDescription>
            <CardTitle className="flex items-center gap-2 text-xl">
              <IconFolder className="size-4 text-muted-foreground" />
              {groupCount}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card size="sm">
          <CardHeader>
            <CardDescription>Member</CardDescription>
            <CardTitle className="flex items-center gap-2 truncate text-sm">
              <IconUser className="size-4 shrink-0 text-muted-foreground" />
              <span className="truncate">
                {session.user.name ?? session.user.email ?? "Signed in"}
              </span>
            </CardTitle>
          </CardHeader>
        </Card>
      </section>

      {error ? (
        <Card>
          <CardHeader>
            <Badge variant="destructive" className="w-fit">
              Connection error
            </Badge>
            <CardTitle>Could not load memberships</CardTitle>
            <CardDescription>{error}</CardDescription>
          </CardHeader>
        </Card>
      ) : organizations.length === 0 ? (
        <Card>
          <CardHeader>
            <CardTitle>No memberships found</CardTitle>
            <CardDescription>
              This account does not have an organization membership in the
              current Keycloak token. If membership was just configured, sign
              out and sign in again to request a fresh organization scope.
            </CardDescription>
          </CardHeader>
        </Card>
      ) : (
        <section className="grid gap-4">
          {organizations.map((organization) => (
            <Card key={organization.alias}>
              <CardHeader className="border-b">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <CardTitle className="flex items-center gap-2">
                      <IconBuilding className="size-4" />
                      {organization.alias}
                    </CardTitle>
                    <CardDescription>
                      {organization.groups.length} group
                      {organization.groups.length === 1 ? "" : "s"} assigned
                    </CardDescription>
                  </div>
                  <Badge variant="outline">Member</Badge>
                </div>
                {organization.id ? (
                  <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                    <IconId className="size-3.5" />
                    <code>{organization.id}</code>
                  </div>
                ) : null}
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Group</TableHead>
                      <TableHead>Member</TableHead>
                      <TableHead className="text-right">Status</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {organization.groups.length > 0 ? (
                      organization.groups.map((group) => (
                        <TableRow key={group}>
                          <TableCell className="font-medium">{group}</TableCell>
                          <TableCell>
                            <div className="flex flex-col">
                              <span>
                                {session.user.name ?? "Signed in user"}
                              </span>
                              {session.user.email ? (
                                <span className="text-muted-foreground">
                                  {session.user.email}
                                </span>
                              ) : null}
                            </div>
                          </TableCell>
                          <TableCell className="text-right">
                            <Badge variant="secondary">Active</Badge>
                          </TableCell>
                        </TableRow>
                      ))
                    ) : (
                      <TableRow>
                        <TableCell
                          colSpan={3}
                          className="py-8 text-center text-muted-foreground"
                        >
                          You belong to this organization but have no group
                          assignment.
                        </TableCell>
                      </TableRow>
                    )}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          ))}
        </section>
      )}
    </main>
  )
}
