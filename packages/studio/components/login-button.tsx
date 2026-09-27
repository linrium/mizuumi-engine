"use client"

import { Button } from "@/components/ui/button"
import { authClient } from "@/lib/auth-client"

export function LoginButton() {
  async function signIn() {
    await authClient.signIn.social({
      provider: "keycloak",
      callbackURL: "/",
    })
  }

  return (
    <Button type="button" onClick={signIn}>
      Sign in with Keycloak
    </Button>
  )
}
