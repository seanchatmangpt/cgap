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
GATES=("010_required_fields.rq" "020_target_unmeasured_not_blocking.rq" "030_gate_id_uniqueness.rq" "040_target_close_condition.rq")
SHIM='SELECT ?s WHERE { ?s ?p ?o } LIMIT 1'

fails=0
# PASS witness: one real sync (runs all 4 gates against the union graph)
rm -f "$ROOT/ggen.lock"
if ! ggen sync run >/dev/null 2>/tmp/gd1-court-pass.err; then
  echo "PASS-WITNESS: sync exit nonzero — court broken"; tail -3 /tmp/gd1-court-pass.err; exit 2
fi
echo "PASS-WITNESS: sync exit 0 (all 4 gates evaluated green against the union graph)"

for g in "${GATES[@]}"; do
  f="$PACK/gates/$g"
  cp "$f" /tmp/gd1-orig.rq || exit 2
  before=$(shasum -a 256 "$f" | cut -d' ' -f1)
  printf '# MESSAGE: SHIM-COURT probe %s\n%s\n' "$g" "$SHIM" > "$f"
  [ -f "$ROOT/ggen.lock" ] && mv "$ROOT/ggen.lock" "$ROOT/ggen.lock.held"
  ggen sync run --dry-run >/dev/null 2>/tmp/gd1-court.err
  rc=$?
  [ -f "$ROOT/ggen.lock.held" ] && mv "$ROOT/ggen.lock.held" "$ROOT/ggen.lock"
  mv /tmp/gd1-orig.rq "$f"
  after=$(shasum -a 256 "$f" | cut -d' ' -f1)
  [ "$before" = "$after" ] || { echo "RESTORE-FAIL $g"; exit 2; }
  if [ "$rc" -eq 0 ]; then
    echo "$g: DID-NOT-FLIP (dry-run exit 0) — VACUOUS court"
    fails=$((fails+1))
  elif grep -q "FM-PACK-013" /tmp/gd1-court.err && grep -q "$g" /tmp/gd1-court.err; then
    echo "$g: FLIP-OK (FM-PACK-013 refusal naming the gate, exit $rc)"
  else
    echo "$g: WRONG-REFUSAL (exit $rc, no FM-PACK-013 naming this gate)"
    fails=$((fails+1))
  fi
done

if [ "$fails" -gt 0 ]; then echo "COURT: VIOLATION ($fails gates did not flip)"; exit 1; fi
echo "COURT: CONFORMANT (4/4 FLIP-OK, restore byte-exact)"
exit 0
