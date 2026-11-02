#!/usr/bin/env bash
# build_image.sh — lane TC-14. Assembles + builds the COMPOSED
# cgap-ax-testbed image: AX-1's Dockerfile stages (digest-pinned bases,
# pinned ax commit b82f57e, nightly+riscv32imc toolchains) MERGED with
# AX-3's cgap-ffi stage fragment (libcgap_ffi.so at the canonical
# /opt/cgap/lib path + frozen ABI header at /opt/cgap/include).
#
# Merge method: programmatic — AX-1's courts/ax-testbed/Dockerfile is the
# source of truth for stages 1-2; the AX-3 fragment is spliced in
# (builder stage before the final FROM, artifact COPYs appended to the
# final stage). AX-1's file is never edited. Context is staged in /tmp
# (repo tree stays untouched; leases deleted at exit per cleanup law).
#
# TC-14 enabling additions (disclosed in the lane receipt): ENV
# CARGO_TARGET_DIR=/tmp/target and the seeded /tmp/ipc2-target/release
# dylib copy — alloc.sh's in-image arm cargo-builds into a writable
# target dir and its cgo LDFLAGS pin -L/tmp/ipc2-target/release
# (courts/interop-chicago/ffi/ffi.go:18); without these the in-image arm
# fails on the read-only /work mount / missing link path.
#
# Gate exit codes (CLAUDE.md law): 0 pass / 1 fail / 2 subject missing /
# 3 loud skip. No stock-image fallback: a build failure exits 2 LOUD.
set -euo pipefail

REPO=/Users/sac/cgap
AX_DIR=$REPO/courts/ax-testbed
CTX=/tmp/tc14-ctx
IMG=cgap-ax-testbed:latest
ENGINE=${ENGINE:-docker}

log() { printf '[tc14] %s\n' "$*"; }

command -v "$ENGINE" >/dev/null 2>&1 || { log "FAIL: engine '$ENGINE' absent"; exit 2; }
"$ENGINE" info >/dev/null 2>&1 || { log "FAIL: $ENGINE daemon not running"; exit 2; }
[ -f "$AX_DIR/Dockerfile" ] || { log "FAIL: AX-1 Dockerfile missing"; exit 2; }
[ -f "$AX_DIR/docker/cgap-ffi.stage" ] || { log "FAIL: AX-3 stage fragment missing"; exit 2; }

# ---- 1. Stage the build context in /tmp (lease) ----
rm -rf "$CTX"; mkdir -p "$CTX"
( cd "$REPO" && tar cf - --exclude '*/target' --exclude '*/target/*' \
    --exclude '*/target-a09' --exclude '*/target-a09/*' \
    Cargo.toml Cargo.lock crates include ontologies ) \
  | ( cd "$CTX" && tar xf - )
cp "$AX_DIR/entrypoint.sh" "$CTX/entrypoint.sh"
log "context staged at $CTX ($(du -sh "$CTX" | cut -f1))"

# ---- 2. Assemble the merged Dockerfile (AX-1 verbatim + AX-3 splice) ----
python3 - "$AX_DIR/Dockerfile" "$AX_DIR/docker/cgap-ffi.stage" "$CTX/Dockerfile" <<'PY'
import sys
ax1_path, frag_path, out_path = sys.argv[1:4]
ax1 = open(ax1_path).read().splitlines()
frag = open(frag_path).read().splitlines()
# Final stage starts at the debian:bookworm-slim FROM (AX-1 line 21).
fi = next(i for i, l in enumerate(ax1) if l.startswith("FROM debian:bookworm-slim"))
head, tail = ax1[:fi], ax1[fi:]
# DOCUMENTED DIVERGENCE (TC-14, spliced copy only — AX-1's file untouched):
# AX-1 hardcodes go1.23.4.linux-AMD64.tar.gz; this host is darwin/arm64 and
# docker pulls arm64 debian bases, so amd64 go emits `-m64` into the
# aarch64 gcc and EVERY cgo build dies (witnessed exit 1, lane receipt W2).
# The composed image must be arm64-end-to-end for the in-image cgo courts.
tail = [l.replace("go1.23.4.linux-amd64.tar.gz", "go1.23.4.linux-arm64.tar.gz") for l in tail]
assert any("linux-arm64" in l for l in tail), "arm64 go substitution did not apply"
# AX-3 fragment: builder stage only (stop before the fragment's own final FROM).
fb = next(i for i, l in enumerate(frag) if l.startswith("FROM rust:1-slim AS cgap-ffi-builder"))
fe = next(i for i, l in enumerate(frag) if i > fb and l.startswith("FROM rust:1-slim"))
builder_stage = ["# --- TC-14 splice: AX-3 cgap-ffi builder stage (verbatim fragment lines) ---"] + frag[fb:fe]
merger = [
    "# --- TC-14 merge: AX-3 cgap-ffi artifacts into the AX-1 final stage ---",
    "COPY --from=cgap-ffi-builder /work/target/release/libcgap_ffi.so /opt/cgap/lib/libcgap_ffi.so",
    "COPY --from=cgap-ffi-builder /work/include/cgap.h /opt/cgap/include/cgap.h",
    "# TC-14 enabling additions (disclosed in lane receipt): writable cargo",
    "# target dir + seeded cgo-pinned link path for the alloc.sh in-image arm.",
    "ENV CARGO_TARGET_DIR=/tmp/target",
    "RUN mkdir -p /tmp/ipc2-target/release && cp /opt/cgap/lib/libcgap_ffi.so /tmp/ipc2-target/release/",
]
with open(out_path, "w") as f:
    for line in head + builder_stage + tail + merger:
        f.write(line + "\n")
print("[tc14] merged Dockerfile assembled: %d lines" % (len(head) + len(builder_stage) + len(tail) + len(merger)))
PY
grep -q "cgap-ffi-builder" "$CTX/Dockerfile" || { log "FAIL: splice produced no cgap-ffi stage"; exit 1; }
log "merged Dockerfile: $(grep -c '^FROM' "$CTX/Dockerfile") FROM stages"

# ---- 3. Build (compose never falls back to a stock image) ----
log "building $IMG"
"$ENGINE" build -f "$CTX/Dockerfile" -t "$IMG" "$CTX"

# ---- 4. Composition artifact courts (in-image, real filesystem) ----
"$ENGINE" run --rm --entrypoint /bin/sh "$IMG" -c \
  'set -eu
   test -s /opt/cgap/lib/libcgap_ffi.so
   test -f /opt/cgap/include/cgap.h
   test -s /tmp/ipc2-target/release/libcgap_ffi.so
   test -x /usr/local/bin/ax-task-runner
   grep -q "$(cat /usr/local/share/ax_commit.txt)" /usr/local/share/ax_commit.txt
   nm -D --defined-only /opt/cgap/lib/libcgap_ffi.so | grep -c cgap_ | grep -qv "^0$"'
log "PASS artifact courts (ffi.so + header + seeded link path + runner + pin)"

# ---- 5. Cleanup: context lease (image persists by design) ----
python3 -c "import shutil; shutil.rmtree('$CTX', True)"
log "DONE: $IMG built (composition of AX-1 stages + AX-3 cgap-ffi stage)"

