#!/usr/bin/env bash
# fmt-edited.sh — PostToolUse formatter for edited .rs files.
# stdin: JSON with tool_input.file_path. If *.rs, run rustfmt on it.
# Formatting hook: exit 0 always — format, never block.
set -u
input=$(cat)
[ -z "$input" ] && exit 0
file=$(printf '%s' "$input" | python3 -c 'import json,sys
try:
    d=json.load(sys.stdin)
    print(d.get("tool_input",{}).get("file_path",""))
except Exception:
    print("")' 2>/dev/null)
[ -z "$file" ] && exit 0
case "$file" in
  *.rs) ;;
  *) exit 0 ;;
esac
[ -f "$file" ] || exit 0
rustfmt --edition 2021 "$file" >/dev/null 2>&1
exit 0
