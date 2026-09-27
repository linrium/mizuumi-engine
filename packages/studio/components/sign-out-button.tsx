"use client"

import { Button } from "@/components/ui/button"
import { authClient } from "@/lib/auth-client"

export function SignOutButton() {
  async function signOut() {
    await authClient.signOut({ callbackURL: "/" })
  }

  return (
    <Button type="button" variant="outline" onClick={signOut}>
      Sign out
    </Button>
  )
}
