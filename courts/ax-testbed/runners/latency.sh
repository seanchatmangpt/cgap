#!/usr/bin/env bash
# LANE AX-5 — containerized latency court runner (linux re-measure of the
# IPC-3 darwin latency court, lane-ipc-03-latency-court.md).
#
# Execution mode (AX-0 gate W4: cluster NO-GO, LOCAL PATH GO): container
# exec via podman if present, else docker (podman absent on this host,
# witnessed `podman --version` exit 127, lane-ax-00-gate.md W2).
#
# Phases:
#   A. build libcgap_ffi cdylib for linux INSIDE the container
#   B. regenerate A04 fixtures inside the container, byte-compare vs darwin
#   C. run the interop-chicago courts + batch stats (no-p99 law)
#   D. anti-vacuity flip witness: magic-flip MUST return typed INVALID_MAGIC
#
# Exit codes: 0 pass | 1 court fail | 2 subject missing | 3 loud skip
# (dylib/fixture not built — names the rebuild command, never a vacuous pass).
set -u

REPO="${CGAP_REPO:-/Users/sac/cgap}"
WORK="${CGAP_AX5_WORK:-/tmp/ax5-work}"
COURTS="$REPO/courts/interop-chicago"

log() { printf '[AX5] %s\n' "$*"; }

# --- runtime selection (podman per dispatch; docker per AX-0 W2) ----------
RT=""
if command -v podman >/dev/null 2>&1; then
  RT=podman
elif command -v docker >/dev/null 2>&1; then
  RT=docker
  docker info >/dev/null 2>&1 || { printf '[AX5] [SKIP] docker daemon not up\n'; exit 3; }
else
  printf '[AX5] [SKIP] no podman and no docker — container arms LOUD-BLOCKED\n'
  exit 3
fi
log "runtime=$RT (podman absent -> docker per AX-0 LOCAL path)"

rm -rf "$WORK"; mkdir -p "$WORK/target" "$WORK/fixtures" "$WORK/src"
cp -R "$COURTS" "$WORK/src/interop"

# --- Phase A: linux cdylib build inside the container ---------------------
log "phase A: cargo build --release -p cgap-ffi (linux, in-container)"
"$RT" run --rm \
  -v "$REPO":/repo:ro \
  -v "$WORK":/work \
  -e CARGO_TARGET_DIR=/work/target \
  rust:1-bookworm \
  bash -c 'cd /repo && cargo build --release --locked -p cgap-ffi' || {
  printf '[AX5] [SKIP] linux dylib build failed — rebuild with: %s\n' \
    "$RT run --rm -v $REPO:/repo:ro -v $WORK:/work rust:1-bookworm bash -c 'cd /repo && cargo build --release --locked -p cgap-ffi'"
  exit 3
}
[ -f "$WORK/target/release/libcgap_ffi.so" ] || {
  printf '[AX5] [SKIP] libcgap_ffi.so MISSING in-image — rebuild: %s\n' \
    "$RT run --rm -v $REPO:/repo:ro -v $WORK:/work rust:1-bookworm bash -c 'cd /repo && cargo build --release --locked -p cgap-ffi'"
  exit 3
}

# --- Phase B: fixture regeneration in-container + byte-identical replay ---
FXGEN=/tmp/ipc3-fxgen
[ -d "$FXGEN/src" ] || {
  printf '[AX5] [SKIP] %s absent — fixture generator is a /tmp lease; regenerate per lane-ipc-03 W1\n' "$FXGEN"
  exit 3
}
log "phase B: regenerate A04 fixtures in-container"
# fxgen path-deps into the repo; copy it into the rw work root and rewrite
# the absolute darwin paths to the /repo mount before building in-container.
cp -R "$FXGEN" "$WORK/fxgen"
find "$WORK/fxgen" -name Cargo.toml -exec sed -i '' \
  -e "s|/Users/sac/cgap/crates|/repo/crates|g" {} + 2>/dev/null || true
"$RT" run --rm \
  -v "$REPO":/repo:ro \
  -v "$WORK":/work \
  -e CARGO_TARGET_DIR=/work/fxgen-target \
  rust:1-bookworm \
  bash -c 'cd /work/fxgen && cargo run --release -- /work/fixtures' || {
  printf '[AX5] [SKIP] fixture generation failed in-container\n'
  exit 3
}
for f in frame_24.bin batch_1365.bin; do
  if [ -f "/tmp/ipc3-fixtures/$f" ]; then
    if cmp -s "/tmp/ipc3-fixtures/$f" "$WORK/fixtures/$f"; then
      log "replay witness: $f byte-identical darwin<->linux"
    else
      log "REPLAY DIVERGENCE: $f differs darwin vs linux build"
      exit 1
    fi
  else
    log "[SKIP] darwin fixture /tmp/ipc3-fixtures/$f absent — no replay witness this run"
  fi
done

# --- Phase C: courts + latency measurement in golang container ------------
log "phase C: go courts + benchmarks (CGAP_FIXTURE_DIR=/work/fixtures)"
# CGO_LDFLAGS_ALLOW not needed: the linux rpath (-L/work/target/release) is
# baked into the binding's #cgo linux directive (AX-5 adaptation).
"$RT" run --rm \
  -v "$WORK":/work \
  -e CGAP_FIXTURE_DIR=/work/fixtures \
  golang:1.23-bookworm \
  bash -c 'cd /work/src/interop && go vet ./... && go test -v -run "TestAdmitValidFrame|TestMutationGateBreach|TestTimerResolutionWitness|TestEmptyCgoCallFloor|TestBatchStats" . && go test -run XXX -bench "BenchmarkAdmitSingle|BenchmarkAdmitBatch1365" -benchtime 200x -count 5 .' || {
  printf '[AX5] court run FAILED (exit $?)\n'
  exit 1
}

# --- Phase D: explicit flip witness (anti-vacuity, must PASS) -------------
log "phase D: flip witness (TestMutationGateBreach must PASS)"
"$RT" run --rm \
  -v "$WORK":/work \
  -e CGAP_FIXTURE_DIR=/work/fixtures \
  golang:1.23-bookworm \
  bash -c 'cd /work/src/interop && go test -v -run TestMutationGateBreach -count=1 .' || {
  printf '[AX5] VACUOUS: flip witness did not pass — court FAIL\n'
  exit 1
}
log "DONE: all container arms PASS (exit 0)"
