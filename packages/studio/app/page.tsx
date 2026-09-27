import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { SignOutButton } from "@/components/sign-out-button"
import { auth } from "@/lib/auth"
import Link from "next/link"

export default async function Home() {
  const session = await auth.api.getSession({
    headers: await headers(),
  })

  if (!session) {
    redirect("/login")
  }

  return (
    <main className="">
      <div>
        <Link href="/lineages">Lineages</Link>
        <Link href="/unitycatalog">Unity Catalog</Link>
      </div>
      <div className="flex min-h-screen items-center justify-center">
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
          <SignOutButton />
        </section>
      </div>
    </main>
  )
}
