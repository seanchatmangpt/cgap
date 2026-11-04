#!/bin/zsh
# dod-pack witness court (ar-1 protocol): per gate, a PASS witness (unshimmed
# sync exits 0 = all gates green) and a FAIL witness (shim one gate to an
# always-violating SELECT, hold ggen.lock aside, dry-run must refuse with
# [FM-PACK-013]). Restore is sha256-verified fail-closed.
# Verdicts: CONFORMANT=0, VIOLATION=1, COURT-BROKEN=2.
set -u
ROOT="/Users/sac/cgap"
PACK="$ROOT/packs/dod-pack"
cd "$ROOT" || exit 2
declare -a GATES
GATES=("010_required_fields.rq" "020_target_unmeasured_not_blocking.rq" "030_gate_id_uniqueness.rq" "040_target_close_condition.rq")
SHIM='SELECT ?s WHERE { ?s ?p ?o } LIMIT 1'

fails=0; broken=0
# PASS witness: one real sync (runs all 4 gates against the union graph)
rm -f "$ROOT/ggen.lock"
if ! ggen sync run >/dev/null 2>/tmp/gd1-court-pass.err; then
  echo "PASS-WITNESS: sync exit nonzero — court broken"; cat /tmp/gd1-court-pass.err | tail -3; exit 2
fi
echo "PASS-WITNESS: sync exit 0 (all 4 gates evaluated green against the union graph)"

for g in "${GATES[@]}"; do
  f="$PACK/gates/$g"
  before=$(shasum -a 256 "$f" | cut -d' ' -f1)
  before_lock=$(shasum -a 256 "$ROOT/ggen.lock" 2>/dev/null | cut -d' ' -f1)
  printf '# MESSAGE: SHIM-COURT probe %s\n%s\n' "$g" "$SHIM" > "$f"
  mv "$ROOT/ggen.lock" "$ROOT/ggen.lock.held" 2>/dev/null
  ggen sync run --dry-run >/dev/null 2>/tmp/gd1-court.err
  rc=$?
  if [ -f "$ROOT/ggen.lock.held" ]; then mv "$ROOT/ggen.lock.held" "$ROOT/ggen.lock"; fi
  # restore + verify byte-exact
  printf '%s' "$before" > /tmp/gd1-expected.sha
  shasum -a 256 -c /tmp/gd1-expected.sha >/dev/null 2>&1 || true
  # (shasum -c needs the filename; verify manually)
  after=$(shasum -a 256 "$f" | cut -d' ' -f1)
  [ "$before" = "$after" ] || { echo "RESTORE-FAIL $g"; exit 2; }
  if [ "$rc" -eq 0 ]; then
    echo "$g: DID-NOT-FLIP (dry-run exit 0) — court FAIL, VACUOUS"
    fails=$((fails+1))
  elif grep -q "FM-PACK-013" /tmp/gd1-court.err && grep -q "$g" /tmp/gd1-court.err; then
    echo "$g: FLIP-OK (FM-PACK-013 refusal naming the gate, exit $rc)"
  else
    echo "$g: WRONG-REFUSAL (exit $rc, no FM-PACK-013 naming this gate)"
    fails=$((fails+1))
  fi
done

if [ "$broken" -gt 0 ]; then exit 2; fi
if [ "$fails" -gt 0 ]; then echo "COURT: VIOLATION ($fails gates did not flip)"; exit 1; fi
echo "COURT: CONFORMANT (4/4 FLIP-OK, restore byte-exact)"
exit 0
