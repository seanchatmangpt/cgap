#!/bin/sh
# courts/ax-testbed/docker/build-cgap-ffi.sh — lane AX-3 (owner: this lane).
#
# Kills the dxd-03 dual-/tmp-target finding: the image owns ONE canonical
# dylib path — /opt/cgap/lib/libcgap_ffi.so — plus the frozen ABI header at
# /opt/cgap/include/cgap.h. No consumer-side /tmp target scalping.
#
# GOOS honesty: container = LINUX -> libcgap_ffi.so (ELF). IPC-0 session
# courts measured darwin/arm64 (.dylib); linux/arm64 re-measurement is this
# lane's FINDING, not a port. cgo LDFLAGS switch by GOOS (identical -L/-l on
# both arms; only the file name differs by platform convention):
#   #cgo darwin LDFLAGS: -L/opt/cgap/lib -lcgap_ffi  (libcgap_ffi.dylib)
#   #cgo linux  LDFLAGS: -L/opt/cgap/lib -lcgap_ffi  (libcgap_ffi.so)
#
# DRAFT-first law: podman absent on this host is fine — docker 28.0.4 with
# running daemon is the lawful container path (AX-0 gate W4: GO via docker).
# If neither runtime is present, fall back to LOCAL cargo build and mark
# courts DRAFT/LOCAL — never a silent fake container pass.
set -eu

REPO="${CGAP_REPO:-/Users/sac/cgap}"
STAGE="$REPO/courts/ax-testbed/docker/cgap-ffi.stage"
IMG=cgap-ffi:ax3
IMG_MUT=cgap-ffi:ax3-antivacuity
SANDBOX=$(mktemp -d /tmp/ax3-sandbox.XXXXXX)
CTX=/tmp/ax3-ctx
LEASE=/tmp/ax3-target
MODE=""; RT=""; PASS=0; FAIL=0; FLIP=0

# IPC-0 ground-truth oracle (docs/jira/v26.11.2/lanes/lane-ipc-00-ground-truth.md W3):
EXPECT="cgap_admit_frame
cgap_eval_bn3
cgap_commit_parquet
cgap_verify_p28
cgap_i32_batch
cgap_schema_addr
cgap_array_addr
cgap_batch_rows
cgap_version
cgap_digest_of
cgap_p28_invariant_check"

pass() { printf 'PASS  %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf 'FAIL  %s\n' "$1"; FAIL=$((FAIL+1)); }
end()
{
  printf '\nSUMMARY %s: pass=%d fail=%d flip=%s\n' "$MODE" "$PASS" "$FAIL" "$FLIP"
  [ "$FAIL" -eq 0 ] || exit 1
}
cleanup()
{
  python3 -c "import shutil; shutil.rmtree('$SANDBOX', True); shutil.rmtree('/tmp/ax3-ctx', True)"
  "$RT" rmi -f "$IMG_MUT" >/dev/null 2>&1 || true
  [ -d "$LEASE" ] && [ "$MODE" = LOCAL-FALLBACK ] && python3 -c "import shutil; shutil.rmtree('$LEASE', True)"
  return 0
}

detect_runtime() {
  if command -v podman >/dev/null 2>&1; then RT=podman; MODE=CONTAINER
  elif command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    RT=docker; MODE=CONTAINER
  else
    MODE=LOCAL-FALLBACK
  fi
}

prepare_ctx() {  # $1 = source root (Cargo.toml/crates/include), $2 = ctx dir
  rm -rf "$2"; mkdir -p "$2"
  ( cd "$1" && tar cf - --exclude '*/target' --exclude '*/target/*' --exclude '*/target-a09' --exclude '*/target-a09/*' \
      Cargo.toml Cargo.lock crates include ontologies ) \
    | ( cd "$2" && tar xf - )
  cp "$REPO/courts/ax-testbed/docker/cgap-ffi.stage" "$2/cgap-ffi.stage"
}
build_context_tar() {  # $1 = ctx dir (contains cgap-ffi.stage at root)
  tar -C "$1" -cf - .
}

court_container() {
  prepare_ctx "$REPO" "$CTX"
  if ! build_context_tar "$CTX" | "$RT" build -f cgap-ffi.stage -t "$IMG" - >/dev/null; then
    fail "C1 in-image release build FAILED"
    cleanup; end
  fi
  pass "C1 in-image release build (cargo build --release -p cgap-ffi) exit 0"
}
court_container_local_fallback() {
  CARGO_TARGET_DIR="$LEASE" cargo build --release -p cgap-ffi --manifest-path "$REPO/Cargo.toml" >/dev/null
  pass "C1-LOCAL release build (honest fallback: no container runtime)"
}

court_artifact() {  # $1 = image (or empty in LOCAL mode)
  if [ "$MODE" = CONTAINER ]; then
    "$RT" run --rm "$1" test -f /opt/cgap/lib/libcgap_ffi.so
    "$RT" run --rm "$1" test -f /opt/cgap/include/cgap.h
    "$RT" run --rm "$1" sh -c 'test -s /opt/cgap/lib/libcgap_ffi.so'
    pass "C2 canonical artifact path /opt/cgap/lib/libcgap_ffi.so (+header), non-empty"
  else
    test -s "$LEASE/release/libcgap_ffi.dylib"
    pass "C2-LOCAL dylib artifact at fixed lease path (darwin fallback)"
  fi
}

symbols_of() {  # $1 = image; prints sorted cgap_* dynamic exports
  "$RT" run --rm "$1" sh -c \
    'nm -D --defined-only /opt/cgap/lib/libcgap_ffi.so | grep -oE "cgap_[a-z0-9_]+" | sort -u'
}

court_symbols() {  # $1 = image; asserts exact IPC-0 11-symbol set
  SYMS=$(symbols_of "$1")
  printf '%s\n' "$SYMS" > /tmp/ax3-syms.$$
  N=$(grep -c . /tmp/ax3-syms.$$)
  printf '%s\n' "$EXPECT" | sort > /tmp/ax3-exp.$$
  if [ "$N" -ne 11 ]; then
    fail "C3 symbol count $N != 11 (IPC-0 oracle)"
  elif diff /tmp/ax3-exp.$$ /tmp/ax3-syms.$$ >/dev/null; then
    pass "C3 symbol count == 11 and exact IPC-0 set match"
  else
    fail "C3 symbol set differs from IPC-0 oracle (count=$N)"
    diff /tmp/ax3-exp.$$ /tmp/ax3-syms.$$ || true
  fi
  rm -f /tmp/ax3-syms.$$ /tmp/ax3-exp.$$
}

decls_of() {  # $1 = image; prints cgap_ function names DECLARED in the header
  "$RT" run --rm "$1" sh -c \
    'grep -oE "cgap_[a-z0-9_]+\(" /opt/cgap/include/cgap.h | tr -d "( " | sort -u'
}

court_header() {  # $1 = image. Declared surface must EQUAL the frozen 5-name set.
  DECLS=$(decls_of "$1")
  SYMS=$(symbols_of "$1")
  printf '%s\n' "$DECLS" > /tmp/ax3-decls.$$
  printf '%s\n' "$SYMS" > /tmp/ax3-syms.$$
  NDECL=$(grep -c . /tmp/ax3-decls.$$)
  # Frozen header surface: 4 sovereign entrypoints + cgap_version.
  EXPECT_DECLS="cgap_admit_frame
cgap_eval_bn3
cgap_commit_parquet
cgap_verify_p28
cgap_version"
  printf '%s\n' "$EXPECT_DECLS" | sort > /tmp/ax3-expdecls.$$
  if [ "$NDECL" -ne 5 ]; then
    fail "C4 header court: declared-surface count $NDECL != 5 (header mutation or drift)"
    RC=1
  elif ! diff /tmp/ax3-expdecls.$$ /tmp/ax3-decls.$$ >/dev/null; then
    fail "C4 header court: declared-surface set drift"
    diff /tmp/ax3-expdecls.$$ /tmp/ax3-decls.$$ || true
    RC=1
  elif comm -23 /tmp/ax3-decls.$$ /tmp/ax3-syms.$$ | grep -q .; then
    fail "C4 header court: declared-not-exported"
    RC=1
  else
    pass "C4 header court: declared surface == frozen 5-name set, all exported"
    RC=0
  fi
  rm -f /tmp/ax3-decls.$$ /tmp/ax3-syms.$$ /tmp/ax3-expdecls.$$
  return $RC
}

court_antivacuity() {
  # Mutate a /tmp SANDBOX copy of the header (never the shared tree): drop the
  # cgap_verify_p28 declaration. The header court MUST flip to FAIL; a green
  # run here is a VACUOUS court.
  ( cd "$REPO" && tar cf - --exclude '*/target' --exclude '*/target/*' --exclude '*/target-a09' --exclude '*/target-a09/*' \
      Cargo.toml Cargo.lock crates include ontologies ) \
    | ( cd "$SANDBOX" && tar xf - )
  python3 - "$SANDBOX/include/cgap.h" <<'PY'
import re, sys
p = sys.argv[1]
s = open(p).read()
s2 = re.sub(r'CgapStatus cgap_verify_p28\([^;]*\);', '', s, count=1)
assert s2 != s, "mutation did not apply"
assert not re.search(r'CgapStatus\s+cgap_verify_p28\(', s2), "declaration not removed"
open(p,'w').write(s2)
PY
  if grep -qE 'CgapStatus[[:space:]]+cgap_verify_p28\(' "$SANDBOX/include/cgap.h"; then
    echo "[SKIP] anti-vacuity mutation did not apply (declaration still present)"
    return 3
  fi
  cp "$REPO/courts/ax-testbed/docker/cgap-ffi.stage" "$SANDBOX/cgap-ffi.stage"
  if ! ( build_context_tar "$SANDBOX" | "$RT" build -q -f cgap-ffi.stage -t "$IMG_MUT" - >/dev/null ); then
    echo "[SKIP] mutated-image build failed (context/build error)"
    return 3
  fi
  if court_header "$IMG_MUT"; then
    fail "ANTI-VACUITY court did NOT flip on mutated header (VACUOUS)"
    FLIP=still-green
  else
    FAIL=$((FAIL-1))  # witnessed flip is not a lane failure; recorded via FLIP=1
    pass "ANTI-VACUITY: mutated header -> C4 header court flips to FAIL"
    FLIP=1
  fi
}

main() {
  detect_runtime
  HSUM=$(shasum -a 256 "$REPO/include/cgap.h" | cut -d" " -f1)
  printf 'AX-3 build-cgap-ffi  mode=%s runtime=%s\n' "$MODE" "${RT:-none}"
  if [ "$MODE" = CONTAINER ]; then
    court_container
    court_artifact "$IMG"
    court_symbols "$IMG"
    court_header "$IMG" || true
    court_antivacuity
  else
    printf 'DRAFT/LOCAL fallback: no container runtime — honest degraded mode\n'
    CARGO_TARGET_DIR="$LEASE" cargo build --release -p cgap-ffi \
      --manifest-path "$REPO/Cargo.toml" >/dev/null
    pass "C1-LOCAL release build exit 0"
    if test -s "$LEASE/release/libcgap_ffi.dylib"; then
      pass "C2-LOCAL dylib artifact at fixed lease path"
    else
      fail "C2-LOCAL dylib artifact missing"
    fi
    SYMS=$(nm -gU "$LEASE/release/libcgap_ffi.dylib" | grep -oE "cgap_[a-z0-9_]+" | sort -u)
    printf '%s\n' "$SYMS" > /tmp/ax3-syms.$$
    N=$(grep -c . /tmp/ax3-syms.$$)
    printf '%s\n' "$EXPECT" > /tmp/ax3-exp.$$
    if [ "$N" -eq 11 ] && diff /tmp/ax3-exp.$$ /tmp/ax3-syms.$$ >/dev/null; then
      pass "C3-LOCAL 11-symbol IPC-0 oracle match"
    else
      fail "C3-LOCAL symbol court (count=$N)"
    fi
    nm -gU "$LEASE/release/libcgap_ffi.dylib" | grep -oE "cgap_[a-z0-9_]+" | sort -u > /tmp/ax3-syms.$$
    grep -oE "cgap_[a-z0-9_]+\(" "$REPO/include/cgap.h" | tr -d "( " | sort -u > /tmp/ax3-decls.$$
    if comm -23 /tmp/ax3-decls.$$ /tmp/ax3-syms.$$ | grep -q .; then
      fail "C4-LOCAL header court: declared-not-exported"
    else
      pass "C4-LOCAL header court: every cgap.h declaration exported"
    fi
    rm -f /tmp/ax3-decls.$$ /tmp/ax3-syms.$$
  fi
  HSUM2=$(shasum -a 256 "$REPO/include/cgap.h" | cut -d" " -f1)
  if [ "$HSUM" = "$HSUM2" ]; then
    pass "sandbox law: shared include/cgap.h sha256 unchanged ($HSUM)"
  else
    fail "shared include/cgap.h MUTATED (sha256 drift) — restore violation"
  fi
  cleanup
  end
}
main "$@"
