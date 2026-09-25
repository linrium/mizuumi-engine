#!/usr/bin/env bash
set -euo pipefail

hostnames=(auth.mizuumi.test vault.mizuumi.test storage.mizuumi.test api.storage.mizuumi.test)
hostnames_joined="${hostnames[*]}"
address=127.0.0.1
hosts_file=/etc/hosts

configured=1
for hostname in "${hostnames[@]}"; do
  if ! awk -v address="$address" -v hostname="$hostname" '
    $1 == address { for (i = 2; i <= NF; i++) if ($i == hostname) found = 1 }
    END { exit !found }
  ' "$hosts_file"; then
    configured=0
    break
  fi
done
if (( configured )); then
  echo "$hostnames_joined already resolve to $address in $hosts_file."
  exit 0
fi

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT
awk -v hostnames="$hostnames_joined" '
  BEGIN {
    count = split(hostnames, names, " ")
    for (i = 1; i <= count; i++) managed[names[i]] = 1
  }
  {
    output = $1
    for (i = 2; i <= NF; i++) if (!managed[$i]) output = output " " $i
    if (NF == 0 || output != $1) print output
  }
' "$hosts_file" > "$tmp_file"
printf '%s\t%s\t# mizuumi local gateway\n' "$address" "$hostnames_joined" >> "$tmp_file"

if (( EUID == 0 )); then
  cp "$hosts_file" "$hosts_file.mizuumi.bak"
  cp "$tmp_file" "$hosts_file"
else
  echo "Updating $hosts_file requires administrator access."
  sudo cp "$hosts_file" "$hosts_file.mizuumi.bak"
  sudo cp "$tmp_file" "$hosts_file"
fi

if command -v dscacheutil >/dev/null 2>&1; then
  dscacheutil -flushcache
fi
echo "$hostnames_joined now resolve to $address on this workstation."
