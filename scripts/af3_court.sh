#!/bin/bash
# AF-3 flip court (v26.11.3): proves the hash_wide_simd NEON-leaf fix is
# load-bearing. Mutation lives in a /tmp sandbox copy of the crate (never
# the shared tree); sha256-exact restore, fail-closed; mtime bump law.
# Exit 0 = AFTER (fix present) passes AND BEFORE (scalar-leaf mutation)
# flips to FAIL. A court that cannot flip is FAIL.
set -u
REPO=/Users/sac/cgap
CRATE=$REPO/crates/cgap-blake3
SANDBOX=/tmp/af3-court-sandbox.$$
DID_MUTATE=0
cleanup() {
  rm -rf "$SANDBOX"
  if [ "$DID_MUTATE" = "1" ]; then touch "$CRATE/src/lib.rs"; fi
}
trap cleanup EXIT
rm -rf "$SANDBOX"; mkdir -p "$SANDBOX"
cp -R "$CRATE/." "$SANDBOX/"
rm -rf "$SANDBOX/target"
python3 /tmp/af3_manifest_patch.py "$SANDBOX/Cargo.toml"
BASE_SHA=$(shasum -a 256 "$CRATE/src/lib.rs" | awk '{print $1}')
echo "[COURT] subject lib.rs sha256: $BASE_SHA"

# ---- probe file --------------------------------------------------------
cp /tmp/af3_probe.rs "$SANDBOX/tests/af3_court_probe.rs"

echo "[COURT] AFTER arm (fix present): expect PASS"
AFTER=PASS
OUT=$(cd "$SANDBOX" && cargo test --release --test af3_court_probe 2>&1)
RC=$?
echo "$OUT" | tail -3
if [ $RC -ne 0 ]; then AFTER=FAIL; fi
echo "[COURT] AFTER arm result: $AFTER"
if [ "$AFTER" != "PASS" ]; then
  echo "COURT FAIL: fix-present probe did not clear the bar"
  exit 1
fi

# ---- BEFORE arm: mutate sandbox, expect probe FAIL ---------------------
echo "[COURT] BEFORE arm (scalar-leaf SHIM-COURT mutation): expect FAIL"
python3 /tmp/af3_mutate.py "$SANDBOX/src/lib.rs"
if ! grep -q "SHIM-COURT" "$SANDBOX/src/lib.rs"; then
  echo "COURT FAIL: mutation marker missing"; exit 1
fi
BEFORE=FAIL
OUT=$(cd "$SANDBOX" && cargo test --release --test af3_court_probe 2>&1)
RC=$?
echo "$OUT" | tail -3
if [ $RC -eq 0 ]; then BEFORE=PASS; fi
echo "[COURT] BEFORE arm result: $BEFORE (must be FAIL to flip)"
if [ "$BEFORE" != "FAIL" ]; then
  echo "COURT FAIL: mutated probe did not flip (VACUOUS court)"; exit 1
fi

# ---- restore verification (shared tree untouched, fail-closed) --------
AFTER_SHA=$(shasum -a 256 "$CRATE/src/lib.rs" | awk '{print $1}')
if [ "$AFTER_SHA" != "$BASE_SHA" ]; then
  echo "COURT FAIL: shared-tree sha drift $BASE_SHA -> $AFTER_SHA"
  exit 1
fi
echo "[COURT] shared-tree sha unchanged: $AFTER_SHA"
echo "COURT PASS: AFTER=PASS BEFORE=FAIL (flip proven)"
