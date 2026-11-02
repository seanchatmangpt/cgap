#!/usr/bin/env bash
# guard-secrets.sh — PreToolUse guard for secret surfaces.
# stdin: JSON with tool_input.file_path. Exit 2 (block) if the path matches
# .env*, secrets/**, or *credentials*; else exit 0.
set -u
input=$(cat)
[ -z "$input" ] && exit 0
file=$(printf '%s' "$input" | python3 -c 'import json,sys
try:
    d=json.load(sys.stdin)
    print(d.get("tool_input",{}).get("file_path",""))
except Exception:
    print("__MALFORMED__")' 2>/dev/null)
[ "$file" = "__MALFORMED__" ] && exit 2
[ -z "$file" ] && exit 0
base=$(basename "$file")
case "$base" in
  .env|.env.*) exit 2 ;;
  *.env|*.env.*|env.backup|env.backup.*) exit 2 ;;
  *credentials*|.git-credentials|.netrc) exit 2 ;;
  *.pem|*.key|*.p12|*.pfx) exit 2 ;;
  id_rsa|id_rsa.*|id_ed25519|id_ed25519.*|id_ecdsa|id_ecdsa.*) exit 2 ;;
esac
case "$file" in
  */.env|*/.env.*) exit 2 ;;
  */secrets/*|secrets/*) exit 2 ;;
  */credentials*|*credentials*) exit 2 ;;
  */.netrc|.netrc) exit 2 ;;
esac
exit 0
