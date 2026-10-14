import { headers } from "next/headers"
import Link from "next/link"
import {
  IconBuildingBank,
  IconDatabase,
  IconMessageCircle,
  IconPlugConnected,
  IconTerminal2,
} from "@tabler/icons-react"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { auth } from "@/lib/auth"
import { redirect } from "next/navigation"

const workspaces = [
  {
    href: "/chat",
    label: "Pi Agents",
    description: "Converse with the Pi agent fleet and inspect reasoning steps.",
    icon: IconMessageCircle,
  },
  {
    href: "/mcp",
    label: "MCP Chat",
    description: "Explore tools exposed over the Model Context Protocol.",
    icon: IconPlugConnected,
  },
  {
    href: "/sql",
    label: "DuckDB SQL",
    description: "Run SQL against embedded DuckDB and shared datasets.",
    icon: IconTerminal2,
  },
  {
    href: "/unitycatalog",
    label: "Unity Catalog",
    description: "Browse catalogs, schemas, and tables managed by Unity.",
    icon: IconDatabase,
  },
  {
    href: "/banking",
    label: "Banking",
    description: "Operate the Temenos-backed banking simulator.",
    icon: IconBuildingBank,
  },
]

export default async function Home() {
  const session = await auth.api.getSession({
    headers: await headers(),
  })

  return (
    <main className="mx-auto flex w-full max-w-6xl flex-col gap-8 px-6 py-10">
      <section className="flex flex-col gap-2">
        <Badge variant="secondary" className="w-fit">
          Welcome
        </Badge>
        <h1 className="text-2xl font-semibold tracking-tight">
          Hi {session?.user.name ?? "there"}
        </h1>
        <p className="text-sm text-muted-foreground">
          Jump into a workspace to start building on Mizuumi.
        </p>
      </section>

      <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {workspaces.map((workspace) => {
          const Icon = workspace.icon
          return (
            <Link
              key={workspace.href}
              href={workspace.href}
              className="group focus-visible:outline-none"
            >
              <Card className="h-full transition-colors group-hover:border-primary/50 group-focus-visible:border-primary">
                <CardHeader>
                  <div className="flex size-10 items-center justify-center rounded-md bg-muted text-foreground">
                    <Icon className="size-5" />
                  </div>
                  <CardTitle className="mt-3">{workspace.label}</CardTitle>
                  <CardDescription>{workspace.description}</CardDescription>
                </CardHeader>
              </Card>
            </Link>
          )
        })}
      </section>
    </main>
  )
}
