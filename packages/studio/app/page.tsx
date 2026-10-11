import { headers } from "next/headers"
import Link from "next/link"
import { redirect } from "next/navigation"
import { SignOutButton } from "@/components/sign-out-button"
import { auth } from "@/lib/auth"

export default async function Home() {
  const session = await auth.api.getSession({
    headers: await headers(),
  })

  if (!session) {
    redirect("/login")
  }

  return (
    <main className="">
      <header className="sticky top-0 z-10 border-b bg-background/80 backdrop-blur">
        <nav className="mx-auto flex w-full max-w-5xl items-center justify-between gap-4 px-6 py-3">
          <Link
            href="/"
            className="text-sm font-semibold tracking-tight"
          >
            Mizuumi Studio
          </Link>
          <ul className="flex items-center gap-1 text-sm">
            {[
              { href: "/chat", label: "Pi Agents" },
              { href: "/mcp", label: "MCP Chat" },
              { href: "/unitycatalog", label: "Unity Catalog" },
              { href: "/banking", label: "Banking" },
              { href: "/sql", label: "DuckDB SQL" },
            ].map((item) => (
              <li key={item.href}>
                <Link
                  href={item.href}
                  className="rounded-md px-3 py-1.5 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                >
                  {item.label}
                </Link>
              </li>
            ))}
          </ul>
        </nav>
      </header>
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
