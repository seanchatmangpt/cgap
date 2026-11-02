# Lane AE6 receipt: behavioral patent-gate delta (G7-PATENT)

Date: 2026-11-02. Branch main. Lane-owned files only; no git writes.

## Files written

- `crates/cgap/tests/patent_behavioral_court.rs` (new, 3 tests)
- `scripts/verify_ost_design_around.py` (minimal wiring delta)
- `docs/jira/v26.11.2/countermeasures/lane-06-behavioral-patent-gate.md` (this receipt)

## Court content

Row 1 (US10,817,467): citation, not duplication — pins
`crates/cgap-ffi/tests/fa9_commit_gating.rs` on disk with markers
`CGAP_ERR_CONCURRENT_WRITER_DENIED` and `0x4347_0006`. The FA9FIX mutant
(`0x4347_0006 -> 0` in a sandbox copy flipping the fa9 race court) is CITED
from the FA9FIX receipt, not re-run (per dispatch; sandbox cost saved).

Row 2 (US11,216,456): BEHAVIORAL pushdown witness — `emit_join_masks`
over 256 rows: full-key baseline keeps 256; half-key `bound_keys` prunes
to exactly 128; mask words assert group-exact final state
(groups 0-1 set, 2-3 clear).

Row 3: cyclic derivation — transitive-closure rule over a 199-edge chain,
`unify_to_fixpoint_opt(.., 4, 4096)` refuses TYPED
`FixpointBudgetExceeded { rounds: 4 }` (never hang/panic). Premise
witnessed this session: budget 12 converges Ok (3,294,666 delta rows),
so the refusal is the budget boundary, not a dead rule.

## Commands + exits (this session)

| command | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/ae6-target cargo test -p cgap --test patent_behavioral_court` | 0 (3 passed) |
| `python3 scripts/verify_ost_design_around.py` | 0 (4/4 DISTINCT + court PASS) |
| sandbox baseline `cargo test --test mutant_row2` (/tmp/ae6-bn3-sandbox) | 0 |
| sandbox mutant (`m_key = MASK_ALL` bypass) same command | 101 (court FLIPPED) |
| sha256-exact restore of sandbox unify.rs | match=True (2e7d702f82a08059...) |
| post-restore sandbox rerun | 0 |
| gate fail-path wiring check (/tmp gate copy, court forced-fail `false`) | 1 |

## Gate wiring diff (scripts/verify_ost_design_around.py)

After row verdicts the gate now RUNS `cargo test -p cgap --test
patent_behavioral_court` (cwd repo), prints PASS/FAIL + tail, and
requires exit 0: `gate_exit = 0 iff (all_distinct AND court_ok)`;
failure -> exit 1. Report gains a `behavioral_court` block
(command/exit/pass). Fail-path wiring witnessed via the /tmp gate copy
with the court command stubbed to `false` -> exit 1 (disclosed: the
stub flips the command, not the gate logic).

## Mutant scenario (shim-court, /tmp only, shared tree untouched)

Sandbox `/tmp/ae6-bn3-sandbox` (copy of cgap-bn3, standalone manifest,
path dep to shared cgap-blake3). Baseline unify.rs sha256
`2e7d702f82a08059c0bf18d2c35ff93e6c45d47c4e88a2453fa4d5b813fc5a84`;
mutant sha `ccd81ef7d777557b...`; mutation `let mut m_key = 0u64;` ->
`MASK_ALL` (bound-key arm bypassed) in `emit_join_masks_view` ->
candidate counts identical (256==256) -> court FAILS (exit 101).
Restore copied from shared tree byte-exact (sha256 verified True) +
mtime bump (`touch`) per restore law; post-restore run exit 0.
Shared-tree `unify.rs` sha256 re-read at close: `2e7d702f82a08059...`
= baseline (never mutated).

## Standing

- Behavioral court: ALIVE (exit 0, 3/3, this session).
- G7-PATENT gate: ALIVE (exit 0, 4/4 DISTINCT + court PASS, this
  session); fail-path exit 1 witnessed via /tmp gate copy.
- Row-2 anti-vacuity mutation: FLIPPED (exit 101) in /tmp sandbox.
- Row-1 fa9 mutant (0x4347_0006 -> 0): CITED (FA9FIX receipt), not
  re-run this session.
- Build roots leased: /tmp/ae6-target, /tmp/ae6-sandbox-target,
  /tmp/ae6-gate — deleted at lane close.
