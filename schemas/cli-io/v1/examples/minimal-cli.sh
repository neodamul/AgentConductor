#!/bin/sh
# The smallest conforming CLI: reads acio.input/1 on stdin, writes acio.event/1 lines on stdout.
# `--acio-describe` prints the capability document. Diagnostics go to stderr only.
now() { date -u +%Y-%m-%dT%H:%M:%SZ; }
if [ "$1" = "--acio-describe" ]; then
  printf '%s\n' '{"schema":"acio.describe/1","name":"minimal-cli","version":"1.0.0","acio":{"versions":[1]},"capabilities":{"read_only":true,"streaming":false},"invocation":{"input":"stdin"}}'
  exit 0
fi
input=$(cat)
case "$input" in *'"schema":"acio.input/1"'*) ;; *) echo "minimal-cli: not an acio.input/1 document" >&2; exit 2;; esac
sid="s-$$"
printf '{"schema":"acio.event/1","seq":1,"ts":"%s","type":"session","session_id":"%s","data":{"session_id":"%s"}}\n' "$(now)" "$sid" "$sid"
printf '{"schema":"acio.event/1","seq":2,"ts":"%s","type":"message","session_id":"%s","data":{"role":"assistant","text":"hello","final":true}}\n' "$(now)" "$sid"
printf '{"schema":"acio.event/1","seq":3,"ts":"%s","type":"result","session_id":"%s","data":{"status":"success","summary":"hello"}}\n' "$(now)" "$sid"
