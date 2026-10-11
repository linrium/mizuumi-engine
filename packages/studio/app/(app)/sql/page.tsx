import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"
import { SqlEditor } from "@/app/(app)/sql/sql-editor"

export default async function SqlPage() {
  const session = await auth.api.getSession({ headers: await headers() })

  if (!session) {
    redirect("/login")
  }

  return <SqlEditor />
}
