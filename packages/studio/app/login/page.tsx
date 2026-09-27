import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { LoginButton } from "@/components/login-button"
import { auth } from "@/lib/auth"

export default async function LoginPage() {
  const session = await auth.api.getSession({
    headers: await headers(),
  })

  if (session) {
    redirect("/")
  }

  return (
    <main className="flex min-h-screen items-center justify-center">
      <LoginButton />
    </main>
  )
}
