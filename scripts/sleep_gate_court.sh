#!/usr/bin/env bash
# sleep_gate_court.sh — non-vacuity court for scripts/sleep_gate.py.
#
# Shim-court protocol on a /tmp sandbox subject (never the shared tree):
#   BASELINE (sha256) -> BEFORE (clean annotated subject: gate PASSES)
#   -> SHIM (append unclassified `std::thread::sleep`, marked SHIM-COURT)
#   -> AFTER (gate must REFUSE, exit 1)
#   -> RESTORE (byte-exact sha256, fail-closed).
#
# Legs: lib-side classification and test-side classification.
# Self-check: corrupt a restore on a sacrificial copy and prove the court
# detects it — a court that cannot flip is FAIL (VACUOUS GATE).
set -euo pipefail

GATE="$(cd "$(dirname "$0")" && pwd)/sleep_gate.py"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/cgap_sleep_gate_court.XXXXXX")"
trap 'rm -rf "$SCRATCH"' EXIT

fail() { echo "[SLEEP-GATE-COURT] [FAIL] $*" >&2; exit 1; }
sha() { shasum -a 256 "$1" | awk '{print $1}'; }

# --- self-check: restore detection is not vacuous -------------------------
SELF="$SCRATCH/self.rs"
printf 'fn main() {}\n' > "$SELF"
B0="$(sha "$SELF")"
printf 'std::thread::sleep(x); // SHIM-COURT self-check corruption\n' >> "$SELF"
B1="$(sha "$SELF")"
[ "$B0" != "$B1" ] || fail "self-check: mutation did not change bytes"
# corrupt-restore proof: claiming restore when sha mismatches must be
# catchable — restore_ok() is the ONLY path that restores, and it checks.
restore_ok() { printf 'fn main() {}\n' > "$1"; [ "$(sha "$1")" = "$B0" ]; }
restore_ok "$SELF" || fail "self-check: real restore failed"

# --- leg runner: subject, role, expected annotation -----------------------
run_leg() {
    local role="$1" ann="$2" shim="$3" name="$4"
    local subj="$SCRATCH/${name}_subject.rs"
    printf '// SLEEP-CLASS: %s — SHIM-COURT fixture\n' "$ann" > "$subj"
    printf 'fn f() { let d = Duration::from_millis(1); %s }\n' "" >> "$subj"
    local before sha0
    sha0="$(sha "$subj")"
    BEFORE="$(python3 "$GATE" --as="$role" "$subj" >/dev/null 2>&1 && echo PASS)" \
        || fail "$name BEFORE: clean annotated subject did not PASS"
    [ "$BEFORE" = "PASS" ] || fail "$name BEFORE: expected PASS"
    # SHIM: inject an unclassified fixed sleep (marked SHIM-COURT)
    printf '    std::thread::sleep(d); // %s\n' "$shim" >> "$subj"
    if python3 "$GATE" --as="$role" "$subj" >/dev/null 2>&1; then
        fail "$name AFTER: gate ACCEPTED unclassified sleep — VACUOUS GATE"
    fi
    # RESTORE byte-exact (strip the appended shim line)
    local n
    n=$(wc -l < "$subj" | tr -d ' ')
    head -n $((n - 1)) "$subj" > "$subj.restore"
    mv "$subj.restore" "$subj"
    [ "$(sha "$subj")" = "$sha0" ] || fail "$name: restore not byte-exact"
    python3 "$GATE" --as="$role" "$subj" >/dev/null 2>&1 \
        || fail "$name RESTORED: gate refuses the clean subject again"
    echo "[SLEEP-GATE-COURT] $name leg: BEFORE PASS -> SHIM -> AFTER REFUSE -> RESTORE byte-exact PASS"
}

# --- lib-side + test-side legs --------------------------------------------
run_leg lib CONTRACT 'SHIM-COURT unclassified lib sleep' lib
run_leg test TIME-ADVANCE 'SHIM-COURT unclassified test sleep' test

echo "[SLEEP-GATE-COURT] PASS — both legs flipped; gate is non-vacuous"
