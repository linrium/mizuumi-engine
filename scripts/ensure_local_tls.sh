#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 3 || $# -gt 5 ]]; then
  echo "Usage: $0 <tls-directory> <server-config> <ca-config> [cert-name] [key-name]" >&2
  exit 2
fi

tls_dir="$1"
server_config="$2"
ca_config="$3"
server_cert="$tls_dir/${4:-tls.crt}"
server_key="$tls_dir/${5:-tls.key}"
umask 077
mkdir -p "$tls_dir"

if [[ ! -f "$tls_dir/ca.crt" || ! -f "$tls_dir/ca.key" ]]; then
  if [[ -f "$tls_dir/ca.crt" && -f "$server_cert" && -f "$server_key" ]] &&
     cmp -s "$tls_dir/ca.crt" "$server_cert" &&
     openssl x509 -in "$server_cert" -noout -text | grep -q 'CA:TRUE'; then
    cp "$server_key" "$tls_dir/ca.key"
    echo "Preserved the existing local CA in $tls_dir."
  elif [[ ! -e "$tls_dir/ca.crt" && ! -e "$tls_dir/ca.key" && ! -e "$server_cert" && ! -e "$server_key" ]]; then
    openssl req -x509 -newkey rsa:3072 -sha256 -nodes -days 3650 \
      -config "$ca_config" -keyout "$tls_dir/ca.key" -out "$tls_dir/ca.crt"
    echo "Generated a local CA in $tls_dir."
  else
    echo "Incomplete or unrecognized TLS CA material in $tls_dir; refusing to replace it." >&2
    exit 1
  fi
fi

if [[ -e "$server_cert" || -e "$server_key" ]]; then
  if [[ ! -f "$server_cert" || ! -f "$server_key" ]]; then
    echo "Incomplete TLS server keypair in $tls_dir; refusing to replace it." >&2
    exit 1
  fi
  if ! openssl x509 -in "$server_cert" -noout -text | grep -q 'CA:TRUE'; then
    openssl verify -CAfile "$tls_dir/ca.crt" "$server_cert"
    san_line="$(openssl x509 -in "$server_cert" -noout -text | awk '/Subject Alternative Name/ { getline; print; exit }')"
    renew=0
    while IFS= read -r host; do
      if [[ ",${san_line// /}," != *",DNS:$host,"* ]]; then
        renew=1
        break
      fi
    done < <(awk -F= '/^[[:space:]]*DNS\.[0-9]+[[:space:]]*=/ { gsub(/[[:space:]]/, "", $2); print $2 }' "$server_config")
    if (( renew == 0 )); then
      exit 0
    fi
    echo "Renewing $server_cert to include the configured DNS names."
  fi
fi

openssl req -new -newkey rsa:3072 -sha256 -nodes -config "$server_config" \
  -keyout "$server_key.new" -out "$tls_dir/tls.csr"
openssl x509 -req -in "$tls_dir/tls.csr" -CA "$tls_dir/ca.crt" -CAkey "$tls_dir/ca.key" \
  -CAserial "$tls_dir/ca.srl" -CAcreateserial \
  -days 365 -sha256 -extfile "$server_config" -extensions ext \
  -out "$server_cert.new"
openssl verify -CAfile "$tls_dir/ca.crt" "$server_cert.new"
mv "$server_key.new" "$server_key"
mv "$server_cert.new" "$server_cert"
rm -f "$tls_dir/tls.csr"
echo "Installed a CA-signed server certificate in $tls_dir."
