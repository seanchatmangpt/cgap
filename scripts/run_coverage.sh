#!/usr/bin/env bash
# LANE C01 — llvm-cov coverage baseline runner (fail-closed).
# Snapshot only: sweeper lanes concurrently edit cgap-branchless/csct/bench sources.
set -euo pipefail

REPO=/Users/sac/cgap
TARGET=/tmp/c01-cov
OUT="$REPO/target/coverage"
NIGHTBIN="$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin"

for tool in llvm-profdata llvm-cov; do
  [ -x "$NIGHTBIN/$tool" ] || { echo "BLOCKED: $tool missing in $NIGHTBIN" >&2; exit 2; }
done

mkdir -p "$TARGET" "$OUT"
rm -f "$TARGET"/*.profraw "$TARGET"/coverage.profdata

export LLVM_PROFILE_FILE="$TARGET/cov-%p-%m.profraw"
export CARGO_TARGET_DIR="$TARGET"
export RUSTFLAGS="-C instrument-coverage"

# test phase (exclusions may be appended here on in-flight-lane failures)
EXCLUDE_ARGS=(--exclude bench)
if [ -n "${COV_EXCLUDE:-}" ]; then
  for c in ${COV_EXCLUDE//,/ }; do EXCLUDE_ARGS+=(--exclude "$c"); done
fi

cargo test --workspace --all-targets "${EXCLUDE_ARGS[@]}" "$@"

# merge phase
shopt -s nullglob
profraws=("$TARGET"/cov-*.profraw)
[ ${#profraws[@]} -gt 0 ] || { echo "FAIL: no profraw files emitted" >&2; exit 3; }
"$NIGHTBIN/llvm-profdata" merge -sparse "${profraws[@]}" -o "$TARGET/coverage.profdata"

# locate test binaries from cargo metadata
mapfile -t bins < <(cargo test --workspace --all-targets --no-run --message-format=json 2>/dev/null \
  | python3 -c "
import json,sys
for line in sys.stdin:
    try: m=json.loads(line)
    except Exception: continue
    if m.get('reason')=='compiler-artifact' and m.get('executable') and m.get('profile',{}).get('test'):
        print(m['executable'])
")

# export phase: explicit -object flags (bare positionals are source filters)
OBJ_ARGS=()
for b in "${bins[@]}"; do OBJ_ARGS+=(-object "$b"); done
"$NIGHTBIN/llvm-cov" export \
  --instr-profile="$TARGET/coverage.profdata" \
  --ignore-filename-regex='(/rustc/|/\.cargo/registry|/\.cargo/git|/library/|/opt/homebrew|/usr/lib)' \
  "${OBJ_ARGS[@]}" > "$OUT/coverage_export.json"

echo "COV-OK: $(wc -c < "$OUT/coverage_export.json") bytes export; ${#bins[@]} test binaries"
