#!/usr/bin/env bash
# alloc.sh — Lane AX-4 container runner for the IPC-2 alloc-purity
# courts (linux/arm64). Ports the darwin AllocsPerRun courts
# (courts/interop-chicago/ffi/alloc_court_test.go) into a linux/arm64
# container against the dylib served at /opt/cgap/lib.
#
# Honesty clause: cgo transition tax on linux/arm64 is a NEW measurement;
# darwin numbers (lane-ipc-03-latency-court.md) do NOT carry over.
#
# Execution mode: LOCAL via container engine (docker|podman, ENGINE env).
# Cluster apply stays DRAFT-UNRUN (AX-0 gate W4: cluster NO-GO).
# Image: AX-1 image if present, else pinned stock fallback
# (rust:1-bookworm + golang:1.23-bookworm) — fallback use is disclosed
# in the lane receipt.
set -euo pipefail

ROOT=/Users/sac/cgap
FFI_DIR=$ROOT/courts/interop-chicago/ffi
SHARED=/tmp/ax4-shared          # lane build-root lease (deleted at end)
ENGINE=${ENGINE:-docker}
AX_IMAGE=${AX_IMAGE:-cgap-ax:latest}
COURTS="^(TestSteadyStateAllocPurity|TestVersionAllocPurity|TestAntiVacuityFlip|TestAdmitPositiveControl|TestEvalPositiveControl|TestBLAKE3OfficialEmptyVector)$"

log() { printf '[ax4] %s\n' "$*"; }
die() { printf '[ax4] FAIL: %s\n' "$*" >&2; exit 1; }

command -v "$ENGINE" >/dev/null 2>&1 || die "engine '$ENGINE' absent (set ENGINE=docker|podman)"

# ---- 1. Prefer the AX-1 image when present (podman law per dispatch) ----
if "$ENGINE" image inspect "$AX_IMAGE" >/dev/null 2>&1; then
  log "AX-1 image $AX_IMAGE present; single-container mode"
  mkdir -p "$SHARED"
  "$ENGINE" run --rm --platform linux/arm64 \
    -v "$ROOT:/work:ro" -v "$SHARED:/ax-shared" \
    -e COURTS="$COURTS" \
    "$AX_IMAGE" bash -c '
      set -euo pipefail
      cd /work
      cargo build --release -p cgap-ffi
      mkdir -p /opt/cgap/lib /opt/cgap/include
      cp /tmp/target/release/libcgap_ffi.so /opt/cgap/lib/ 2>/dev/null || \
        find /tmp -name "libcgap_ffi.so" -exec cp {} /opt/cgap/lib/ \;
      cp /work/include/*.h /opt/cgap/include/
      mkdir -p /go-cache /tmp/go-build
      cd /work/courts/interop-chicago/ffi
      GOCACHE=/go-cache CGO_ENABLED=1 go test -v -count=1 -run "$COURTS" .'
  rc=$?
else
  log "AX-1 image absent — pinned stock fallback (rust:1-bookworm + golang:1.23-bookworm), DISCLOSED"
  mkdir -p "$SHARED"
  # Arm 1: build the linux/arm64 dylib in the rust container; publish to
  # the shared lease volume that arm 2 mounts at /opt/cgap/lib.
  "$ENGINE" run --rm --platform linux/arm64 \
    -v "$ROOT:/work:ro" -v "$SHARED:/ax-shared" \
    rust:1-bookworm bash -c '
      set -euo pipefail
      cd /work
      CARGO_TARGET_DIR=/tmp/ax4-target cargo build --release -p cgap-ffi
      mkdir -p /ax-shared/lib /ax-shared/include
      cp /tmp/ax4-target/release/libcgap_ffi.so /ax-shared/lib/
      cp /work/include/*.h /ax-shared/include/'
  [ $? -eq 0 ] || die "dylib build arm failed"
  # Arm 2: run the courts against /opt/cgap/lib inside the golang
  # container. Same test files as darwin (migrate, not re-implement).
  "$ENGINE" run --rm --platform linux/arm64 \
    -v "$SHARED:/opt/cgap:ro" \
    -v "$ROOT:/Users/sac/cgap:ro" \
    -v "$ROOT/courts/interop-chicago:/work:ro" \
    -e COURTS="$COURTS" \
    golang:1.23-bookworm bash -c '
      set -euo pipefail
      cd /work/ffi
      GOCACHE=/tmp/go-cache CGO_ENABLED=1 go test -v -count=1 -run "$COURTS" .'
  rc=$?
fi

log "container court run exit: $rc"
# Lease cleanup (same-checkout fanout cleanup law).
python3 -c "import shutil; shutil.rmtree('/tmp/ax4-shared', True)"
exit $rc