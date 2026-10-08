#!/bin/sh
set -eu

goal="${1:-inspect the sandbox runtime}"
workspace="${OPENSHELL_WORKSPACE:-/sandbox}"
result="$workspace/minimal-agent-result.txt"

{
  printf 'Minimal OpenShell agent\n'
  printf 'goal: %s\n' "$goal"
  printf 'user: %s\n' "$(id -u):$(id -g)"
  printf 'kernel: %s\n' "$(uname -srm)"
  printf 'workspace_writable: '
  if test -w "$workspace"; then
    printf 'yes\n'
  else
    printf 'no\n'
  fi
  printf 'status: completed\n'
} | tee "$result"
