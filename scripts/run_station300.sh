#!/usr/bin/env bash
# run_station300.sh -- TPS manual §3 battery as ONE fail-closed script.
# Every command runs via scripts/run_checked.py (tees output, propagates
# the command's OWN rc). Failing stage halts with the failing command
# named; full green emits typed STATION-300-PASS + sha256 over the
# concatenated transcript.
#
# Usage: run_station300.sh [--dry-run]
#   --dry-run injects a deliberately failing stage (sh -c 'exit 1') and
#   must halt non-green (anti-vacuity flip).
#
# Logs: /tmp/tps5-stage-N.log (kept as receipt evidence).
set -u
REPO=/Users/sac/cgap
LOGS=""
DIR="$REPO"   # cwd for the next stage; set before each run_stage call

run_stage() {
  # run_stage <n> <timeout> <label> <cmd...>   (uses $DIR as cwd)
  local n="$1"; shift
  local to="$1"; shift
  local label="$1"; shift
  echo "=== STAGE $n: $label (cwd=$DIR)"
  ( cd "$DIR" && python3 "$REPO/scripts/run_checked.py" \
      --out "/tmp/tps5-stage-$n.log" \
      --label "station300-stage-$n" \
      --timeout "$to" \
      -- "$@" )
  local rc=$?
  LOGS="$LOGS /tmp/tps5-stage-$n.log"
  if [ "$rc" -ne 0 ]; then
    echo "STATION-300-HALT stage=$n rc=$rc label=$label cmd=$*"
    echo "failing command: $*"
    return "$rc"
  fi
  echo "STAGE $n OK"
  return 0
}

DRY_RUN=0
[ "${1:-}" = "--dry-run" ] && DRY_RUN=1

if [ "$DRY_RUN" -eq 1 ]; then
  echo "STATION-300 DRY-RUN mode: injecting deliberately failing stage"
  run_stage 99 30 "dry-run injected failure" sh -c 'exit 1'
  rc=$?
  [ "$rc" -ne 0 ] && echo "STATION-300-DRYRUN-FLIP-CONFIRMED rc=$rc (anti-vacuity: battery halts on failure)"
  exit "$rc"
fi

# ---- Stage 1: interop-chicago go court --------------------------------
DIR="$REPO/courts/interop-chicago"
run_stage 1a 900 "go vet ./..." go vet ./...
[ $? -ne 0 ] && exit $?
run_stage 1b 900 "go test -count=1 ./..." go test -count=1 ./...
rc=$?
[ "$rc" -ne 0 ] && exit "$rc"

# ---- Stage 2: cargo test battery (CGO tests need the dylib) -----------
# cgap-ffi CGO tests link against the ffi dylib; build it first per the
# documented flow (crates/cgap-ffi cdylib target).
DIR="$REPO"
run_stage 2a 2400 "cargo build -p cgap-ffi --lib" \
  cargo build -p cgap-ffi --lib
rc=$?
[ "$rc" -ne 0 ] && exit "$rc"
run_stage 2b 3600 "cargo test -p cgap-core -p cgap-git -p cgap-ffi (workspace)" \
  cargo test -p cgap-core -p cgap-git -p cgap-ffi
rc=$?
[ "$rc" -ne 0 ] && exit "$rc"

# csct is a STANDALONE crate (own [workspace], like crates/cli): needs
# --manifest-path, not a -p selector in the root workspace.
run_stage 2c 1800 "cargo test -p csct (standalone via manifest-path)" \
  cargo test --manifest-path crates/csct/Cargo.toml
rc=$?
[ "$rc" -ne 0 ] && exit "$rc"

# ---- Stage 3/4: verification scripts ----------------------------------
run_stage 3 600 "verify_disassembly_invariants.py" \
  python3 scripts/verify_disassembly_invariants.py
rc=$?
[ "$rc" -ne 0 ] && exit "$rc"
run_stage 4 600 "verify_cscb_sram.py" \
  python3 scripts/verify_cscb_sram.py
rc=$?
[ "$rc" -ne 0 ] && exit "$rc"

# ---- Stage 5: docker linux/arm64 row (SKIP if daemon dead) ------------
if docker info >/dev/null 2>&1; then
  run_stage 5 2400 "docker linux/arm64 row" \
    docker run --rm --platform linux/arm64 \
    -v "$REPO":/work -w /work rust:1-bookworm \
    bash -c 'export PATH=/usr/local/cargo/bin:$PATH && cargo test -p cgap-ffi' || exit $?
else
  echo "[SKIP] stage 5: docker daemon not alive (docker info failed) -- typed skip, not a failure"
  : > /tmp/tps5-stage-5.log
  printf '[SKIP] docker daemon not alive\n' >> /tmp/tps5-stage-5.log
  LOGS="$LOGS /tmp/tps5-stage-5.log"
fi

# ---- Verdict -----------------------------------------------------------
TRANSCRIPT=/tmp/tps5-transcript.log
cat $LOGS > "$TRANSCRIPT" 2>/dev/null
DIGEST=$(shasum -a 256 "$TRANSCRIPT" | awk '{print $1}')
echo "STATION-300-PASS digest=$DIGEST transcript=$TRANSCRIPT logs:$LOGS"
exit 0
