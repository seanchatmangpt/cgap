# lane-g18-g7-patent-gate — receipt

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. Files written:
this receipt only (no source edits). No git commands.

## Commands + exits (this session)

| command | exit |
|---|---|
| `python3 scripts/verify_ost_design_around.py` | 0 |
| `python3 scripts/verify_ost_design_around.py --help` | 0 |
| `/tmp` mutant bin: budget 4 vs 12 on the 199-edge chain | 0 (both outcomes printed) |
| `/tmp` mutant, court premise inverted (asserts Err at budget 12) | 101 (andon) |
| `sha256sum docs/jira/v26.11.2/patent/claims.txt` | 0 (`963230bb...`) |
| ost_report.json re-read (python json.load) | fields below |

## 1. Gate run (verbatim verdict block)

```
US10,817,467 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
US11,216,456 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
US11,409,698 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
  src hits: 1
US11,645,278 -> DISTINCT-MECHANISM (zero src hits for patented mechanism; landed cite verified)
  src hits: 1
G7-PATENT: PASS 4/4 DISTINCT-MECHANISM + behavioral court (exit 0)
```

The two non-zero `src hits` lines are the gate's own disclosed
non-mechanism classes (verify_ost_design_around.py:135 names them);
verdicts stay DISTINCT-MECHANISM.

Behavioral court matrix (3 rows, pins in
crates/cgap/tests/patent_behavioral_court.rs):

- Row 1 (US10,817,467): citation court — pins
  `CGAP_ERR_CONCURRENT_WRITER_DENIED` + `0x4347_0006` markers in
  crates/cgap-ffi/tests/fa9_commit_gating.rs.
- Row 2 (US11,216,456): mask pushdown / bound-key pruning — 256-row
  baseline keeps all, half-key `bound_keys` prunes to exactly 128;
  mask words asserted on final state (`masks[0..2]` full,
  `[2..4]` zeroed).
- Row 3 (US11,409,698 family): cyclic derivation over a 199-edge
  chain refused TYPED `FixpointBudgetExceeded { rounds: 4 }` under
  budget 4 — never hang, never panic.

## 2. Exit-101 andon + anti-vacuity (budget-flip mutant)

The gate script has NO flag to run Row 3 standalone or with a mutated
budget (only `--claims/--corpus/--report`), so the mutant lives in a
/tmp sandbox bin (`/tmp/g18-budgetcourt`, absolute path dep on
crates/cgap-bn3, CARGO_TARGET_DIR=/tmp/g18-target; shared tree
untouched).

Same corpus (200 nodes / 199 edges), caller-supplied budget is the
only variable:

```
budget 4:  TYPED FixpointBudgetExceeded { rounds: 4 }
budget 12: Ok, 9 delta batches derived
```

This reproduces the countermeasures doc's premise witness (budget 12
converges) against today's tree. Inverted-premise mutant (court
asserts the typed Err where the corpus converges):

```
panicked at src/main.rs:19:25: cyclic derivation converged within
budget - court premise broken (9 batches)
MUTANT_EXIT=101
```

The andon flips, so the budget-4 typed refusal is load-bearing, not
vacuous.

## 3. Receipt/hash form — REAL form recorded (work-order divergence)

The work order claims a "signed BLAKE3 receipt". The script emits NO
signature and NO BLAKE3: verify_ost_design_around.py:15 imports
hashlib, :266 defines sha256(), and the report (:334-347) carries
`claims_sha256` (plain SHA-256 of claims.txt), plus
`duplicate_mechanism_groups`, `all_distinct`, a `behavioral_court`
block (command/exit/pass), and `gate_exit`. Re-read from
docs/jira/v26.11.2/patent/ost_report.json this session:

- claims_sha256 = 963230bbb7167761de5bf0436bf2386a493d7e2b1e2df4bd676f0c53cdad35ac
- behavioral_court = {cargo test -p cgap --test patent_behavioral_court, exit 0, pass True}
- all_distinct = True, gate_exit = 0

The sha matches w6-g7-patent-gate-flip.md:73-74. If a signed BLAKE3
receipt is required, it is a MISSING capability, not an implemented
one.

## 4. Cross-check vs G7 court receipts (re-read at use time)

- w6-g7-patent-gate-flip.md:41 "PASS 4/4 DISTINCT-MECHANISM exit 0" —
  matches today's run (same 4 verdicts + court PASS, exit 0).
- w6:55 "FAIL (2/4 DISTINCT) ... FLIP_EXIT=1" — that flip was the
  duplicate-mechanism shim court; today's equivalent flip witness is
  the budget mutant (exit 101), above. The gate's own fail path
  (exit 1 on non-distinct or court fail) is wired at
  verify_ost_design_around.py:348-356.
- countermeasures/lane-06-behavioral-patent-gate.md:35,59-61 — today's
  run reproduces the doc's table (gate 0, court 0) and its
  budget-12-converges premise witness (9 delta batches this run vs the
  doc's 3,294,666 delta rows — different accounting: this bin counts
  DeltaBatch values, the doc counts rows; both show Ok-at-12 vs
  typed-Err-at-4). Row-2 mutation flip (exit 101, sandboxed
  `m_key = MASK_ALL`) was witnessed by lane AE6 per the doc; this lane
  re-witnessed the Row-3 budget flip, not the Row-2 mask flip.
- claims.txt sha256 re-read today = w6:73-74 value (byte-exact).

## Standing

ALIVE — gate (exit 0, 4/4 + court PASS) and the budget-flip mutant
(Err@4 / Ok@12 / andon@101) both run this session against the exact
subject. Divergences: (1) "signed BLAKE3 receipt" in the work order
is actually an unsigned SHA-256 `claims_sha256` field; (2) no CLI flag
exists for standalone/mutated Row 3 — mutant done via /tmp sandbox
bin. Sandbox debris: /tmp/g18-budgetcourt, /tmp/g18-target (lease;
removed at lane end).
