#!/usr/bin/env bash
# branchless-court.sh — PostToolUse branchless (ANDON) court for edited .rs files.
# Skeleton: fmt-edited.sh:8-18 (stdin JSON -> tool_input.file_path -> .rs filter).
# Non-.rs or missing file: exit 0 without spawning the LSP server.
# Fail-open policy: any harness failure exits 0 with a loud [SKIP] line on
# stderr — this is a convenience hook, never a gate; it must not block the
# session (contrast: fail-closed gates like zero_string_gate.py, which exit
# nonzero on failure).
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
[ -f "$file" ] && [ -r "$file" ] || exit 0

BIN="${BRANCHLESS_LSP_BIN:-/Users/sac/lsp-max/target/release/branchless-lsp}"
if [ ! -x "$BIN" ]; then
  echo "[SKIP] branchless-court: no server binary at $BIN (rebuild: cd /Users/sac/lsp-max && cargo build --release -p branchless-lsp)" >&2
  exit 0
fi

# Staleness guard: the binary must carry the ANDON_REFUSED rule marker.
n=$(strings "$BIN" | grep -c ANDON_REFUSED) || n=0
if [ "$n" -lt 1 ]; then
  echo "[SKIP] branchless-court: stale server at $BIN (0 ANDON_REFUSED markers; rebuild: cd /Users/sac/lsp-max && cargo build --release -p branchless-lsp)" >&2
  exit 0
fi

# Debounce: skip if the file mtime is unchanged since the last court.
# Tier policy (lspu1-consume-plan ranked move #1): the server runs tier
# aarch64 always; riscv32imc only for files under crates/cgap-branchless/
# (the server keys the riscv tier on the path — no hook-side flag exists).
statedir=/tmp/branchless-court-state
mkdir -p "$statedir" 2>/dev/null
key=$(printf '%s' "$file" | shasum -a 256 | cut -d' ' -f1)
state="$statedir/$key"
mtime=$(stat -f %m "$file" 2>/dev/null)
if [ -f "$state" ] && [ "$(cat "$state" 2>/dev/null)" = "$mtime" ]; then
  exit 0
fi
printf '%s' "$mtime" > "$state" 2>/dev/null

# Drive the server via the lsp-max python client (diagnostics_for_files:
# one session, drains the did_open tier-1 publish so the did_save tier-2
# verdict is observed — see lane-g05 §2 quirk + WG2 drain).
out=$(python3 - "$file" "$BIN" <<'PYEOF' 2>&1
import sys
sys.path.insert(0, "/Users/sac/lsp-max/python")
from lsp_client.branchless_client import diagnostics_for_files
path, server_bin = sys.argv[1], sys.argv[2]
res = diagnostics_for_files([path], server_bin=server_bin, timeout=60.0)
for d in res.get(path, []):
    msg = str(d.get("message", ""))
    if "ANDON_REFUSED" in msg:
        line = d.get("range", {}).get("start", {}).get("line", "?")
        print("ANDON_REFUSED line %s: %s" % (line, msg))
        sys.exit(2)
sys.exit(0)
PYEOF
) ; rc=$?
if [ "$rc" -eq 2 ]; then
  printf '%s\n' "$out" >&2
  exit 2
fi
if [ "$rc" -ne 0 ]; then
  echo "[SKIP] branchless-court: harness failure (rc=$rc): $out" >&2
  exit 0
fi
exit 0
