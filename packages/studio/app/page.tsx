"use client"

import { Button } from "@/components/ui/button"
import { authClient } from "@/lib/auth-client"

export default function Home() {
  const { data: session, isPending } = authClient.useSession()

  async function signIn() {
    await authClient.signIn.social({
      provider: "keycloak",
      callbackURL: "/",
    })
  }

  async function signOut() {
    await authClient.signOut({ callbackURL: "/" })
  }

  if (isPending) {
    return (
      <main className="flex min-h-screen items-center justify-center">
        <p>Loading...</p>
      </main>
    )
  }

  if (session) {
    return (
      <main className="flex min-h-screen items-center justify-center">
        <section className="space-y-4 rounded-lg border p-6">
          <h1 className="text-lg font-semibold">Signed in</h1>
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
            <dt className="text-muted-foreground">Name</dt>
            <dd>{session.user.name}</dd>
            <dt className="text-muted-foreground">Email</dt>
            <dd>{session.user.email}</dd>
            <dt className="text-muted-foreground">User ID</dt>
            <dd className="font-mono">{session.user.id}</dd>
          </dl>
          <Button type="button" variant="outline" onClick={signOut}>
            Sign out
          </Button>
        </section>
      </main>
    )
  }

  return (
    <main className="flex min-h-screen items-center justify-center">
      <Button type="button" onClick={signIn}>
        Sign in with Keycloak
      </Button>
    </main>
  )
}
