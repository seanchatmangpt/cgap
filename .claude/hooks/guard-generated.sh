#!/usr/bin/env bash
# guard-generated.sh — PreToolUse guard for generated surfaces.
# stdin: JSON with tool_input.file_path. Exit 2 (block) if the path targets
# a generated artifact per CLAUDE.md's generated-artifacts table; else 0.
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
case "$file" in
  crates/cli/src/generated_*.rs) exit 2 ;;
  crates/cli/Cargo.toml) exit 2 ;;
  crates/cli/docs/command-reference.md) exit 2 ;;
  crates/cli/ggen.lock) exit 0 ;; # not in table; regenerate flow covers it
  ontology/cgap.ttl) exit 2 ;;
  crates/cli/tests/zero_code_cli.rs) exit 2 ;;
  *) exit 0 ;;
esac
