#!/usr/bin/env bash
set -euo pipefail

profile_name="${CUSTOM_OPENAI_PROFILE:-pi-custom-openai}"
provider_name="${CUSTOM_OPENAI_PROVIDER:-pi-custom-openai}"
sandbox_name="${CUSTOM_OPENAI_SANDBOX:-pi-custom-openai}"

command -v openshell >/dev/null || {
  echo "Missing required command: openshell" >&2
  exit 1
}

if ! openshell status >/dev/null; then
  cat >&2 <<'EOF'
Cannot reach the selected OpenShell gateway. Start or reconnect the gateway
(for the Kubernetes setup, keep its kubectl port-forward running), then retry.
EOF
  exit 1
fi

if openshell sandbox get "$sandbox_name" >/dev/null 2>&1; then
  openshell sandbox delete "$sandbox_name"

  sandbox_deleted=false
  for ((attempt = 0; attempt < 30; attempt += 1)); do
    if ! openshell sandbox get "$sandbox_name" >/dev/null 2>&1; then
      if openshell status >/dev/null 2>&1; then
        sandbox_deleted=true
        break
      fi
      echo "Lost connection to the OpenShell gateway while waiting for sandbox deletion." >&2
      exit 1
    fi
    sleep 1
  done

  if [[ "$sandbox_deleted" != true ]]; then
    echo "Timed out waiting for sandbox '$sandbox_name' to be deleted." >&2
    exit 1
  fi
else
  echo "Sandbox '$sandbox_name' is already absent."
fi

if openshell provider get "$provider_name" >/dev/null 2>&1; then
  openshell provider delete "$provider_name"
else
  echo "Provider '$provider_name' is already absent."
fi

if openshell profile describe "$profile_name" >/dev/null 2>&1; then
  openshell profile delete "$profile_name"
else
  echo "Profile '$profile_name' is already absent."
fi

echo "OpenShell cleanup complete."
