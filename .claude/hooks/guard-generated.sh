#!/usr/bin/env bash
# guard-generated.sh — PreToolUse guard for generated surfaces (C5 upgrade).
# stdin: JSON with tool_input.file_path. Exit 2 (block) if the path targets
# a generated artifact per CLAUDE.md's generated-artifacts table; else 0.
# Exit 3 = loud skip (no/malformed-but-empty input shape handled below).
# C5 findings fixed:
#   1. absolute-path bypass (tools pass absolute paths per CLAUDE.md law)
#   2. ggen.lock was exit 0 despite being in the table -> now exit 2
#   3. root generated/ dir (ggen projections) was unprotected -> now blocked
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
[ -z "$file" ] && exit 3
# Normalize absolute paths under the repo to repo-relative so absolute-path
# invocations cannot bypass the case table (CLAUDE.md: absolute paths only).
repo=${CLAUDE_PROJECT_DIR:-/Users/sac/cgap}
case "$file" in
  "$repo") file="." ;;
  "$repo"/*) file=${file#"$repo"/} ;;
esac
case "$file" in
  crates/cli/src/generated_*.rs) exit 2 ;;
  crates/cli/Cargo.toml) exit 2 ;;
  crates/cli/docs/command-reference.md) exit 2 ;;
  crates/cli/ggen.lock) exit 2 ;;
  crates/cli/tests/zero_code_cli.rs) exit 2 ;;
  ontology/cgap.ttl) exit 2 ;;
  ontology/cgap.ttl/*) exit 2 ;;
  generated) exit 2 ;;
  generated/*) exit 2 ;;
  *) exit 0 ;;
esac
