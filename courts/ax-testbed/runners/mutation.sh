#!/usr/bin/env bash
# LANE AX-7 — in-container mutation court runner (4 arms x BEFORE/AFTER).
#
# Usage:
#   ./mutation.sh              # container mode (podman preferred, docker
#                              # fallback) over the cgap-ax-testbed image
#   ./mutation.sh --host       # darwin/linux host mode: run the SAME Go
#                              # mutation court directly via go test
#                              # (the witnessed baseline when the image
#                              # is not built)
#
# Gate exit codes (CLAUDE.md law): 0 pass / 1 court fail / 2 subject
# missing / 3 loud skip (container or fixture absent). Skips are loud,
# never silent; a court that cannot flip is FAIL.
#
# The court under test: courts/interop-chicago/ffi/mutation_court_test.go
# (AX-7 port of POSH-5's refusal matrix arms 1/2/4; arm 3 is
# [SKIP-MISSING-EXPORT] cgap_ingress_bytes — IPC-0 W5).
set -u

REPO=/Users/sac/cgap
COURT_DIR="$REPO/courts/interop-chicago"
RUNNERS_DIR="$REPO/courts/ax-testbed/runners"
IMAGE=cgap-ax-testbed
DYLIB_DIR=/tmp/ipc2-target/release   # pinned by ffi.go cgo LDFLAGS
LEASE=/tmp/ax7-target                # AX-7 build-root lease

mkdir -p "$RUNNERS_DIR" 2>/dev/null || true

log() { printf '[AX-7] %s\n' "$*"; }

# ---- dylib: exact subject, rebuilt fresh into the pinned path ----
build_dylib() {
  log "building libcgap_ffi (exact subject) -> $DYLIB_DIR"
  CARGO_TARGET_DIR=/tmp/ipc2-target cargo build --release -p cgap-ffi \
    --manifest-path "$REPO/Cargo.toml" --quiet || return 1
  ls -l "$DYLIB_DIR"/libcgap_ffi.* 2>/dev/null || true
}

# ---- host mode: the same court, on the darwin/linux host ----
host_mode() {
  log "HOST mode: darwin-side witnessed baseline (container legs LOUD-BLOCKED elsewhere)"
  build_dylib || { log "BLOCKED: dylib build failed"; exit 2; }
  cd "$COURT_DIR" || exit 2
  CGO_ENABLED=1 go test -v ./ffi/ -run 'TestArm1|TestArm2|TestArm3|TestArm4' 2>&1
  rc=$?
  cd "$REPO"
  if [ "$rc" -eq 0 ]; then
    log "HOST court run PASS (exit 0) — arms 1/2/4 ALIVE on host; arm 3 SKIP-MISSING-EXPORT witnessed in-test"
  else
    log "HOST court run FAIL (exit $rc)"
  fi
  exit "$rc"
}

# ---- container mode ----
container_mode() {
  # pick runtime
  RT=""
  if command -v podman >/dev/null 2>&1; then RT=podman; fi
  if [ -z "$RT" ] && command -v docker >/dev/null 2>&1; then RT=docker; fi
  if [ -z "$RT" ]; then
    log "[LOUD-BLOCKED] no container runtime (podman/docker) on PATH — container arms not run"
    log "[LOUD-BLOCKED] darwin-side equivalents (mutation.sh --host) stand as witnessed baseline"
    exit 3
  fi
  if ! "$RT" image inspect "$IMAGE" >/dev/null 2>&1; then
    log "[LOUD-BLOCKED] image $IMAGE not built (courts/ax-testbed/Dockerfile, lane AX-1) — container arms not run"
    log "[LOUD-BLOCKED] build: $RT build -f $REPO/courts/ax-testbed/Dockerfile -t $IMAGE $REPO/courts/ax-testbed"
    log "[LOUD-BLOCKED] darwin-side equivalents (mutation.sh --host) stand as witnessed baseline"
    exit 3
  fi
  log "container runtime: $RT, image: $IMAGE"
  # Linux dylib must exist inside the image/mount; the court needs
  # libcgap_ffi.so — rebuild in-container is owned by the image build,
  # so require a linux dylib mounted at the pinned path.
  if [ ! -f "$DYLIB_DIR/libcgap_ffi.so" ]; then
    log "[LOUD-BLOCKED] $DYLIB_DIR/libcgap_ffi.so missing (linux build required for in-container link)"
    exit 3
  fi
  build_dylib || { log "BLOCKED: dylib build failed"; exit 2; }
  "$RT" run --rm \
    -v "$REPO/courts:/work/courts:ro" \
    -v "$DYLIB_DIR:/tmp/ipc2-target/release:ro" \
    -e CGO_ENABLED=1 \
    -w /work/courts/interop-chicago \
    "$IMAGE" \
    go test -v ./ffi/ -run 'TestArm1|TestArm2|TestArm3|TestArm4'
}

case "${1:-}" in
  --host) host_mode ;;
  "")     container_mode ;;
  *)      log "usage: mutation.sh [--host]"; exit 2 ;;
esac
