# DoD: Python Gates (AUD-02)

Lane: AUD-02 audit of the python gate suite at `scripts/` (repo
`/Users/sac/cgap`, session 2026-11-02). Every gate run FRESH this
session against the exact on-disk subject; exits are literal.
Receipts of record cross-referenced:
`docs/jira/v26.11.2/lanes/lane-g{03,04,10,16,17,18}`-class files plus
`lane-b04`/`lane-c06` (see per-gate rows; the dispatch named
`waves/`, but these receipts actually live in `lanes/` — recorded as
a divergence).

## Gate verdicts (fresh runs, this session)

1a. `boundary_gate.py --deps` — exit 0 — PASS.
    E=0 runtime externals across 15 covered crates (§6.4).
1b. `boundary_gate.py --strings` — exit 0 — PASS.
    0 hits across 15 crates; A07 register v2 consumed
    (sha 7778078dfe11); CROSS-GATE canonical A07 scan GREEN
    (scanned=92, hits=0, emitter_hits=2 exempt).
2a. `zero_string_gate.py` — exit 0 — PASS.
    SUMMARY scanned=92 crates=22 hits=0 emitter_hits=2 adv_hits=0
    gate=1.4.2; [SKIP] rmeta members are loud and typed.
2b. `zero_string_gate.py --self-check` — exit 0 — PASS.
    poison artifacts=1 hits=2 fired=True; clean hits=0;
    RESULT PASS. Anti-vacuity holds.
3. `verify_ost_design_around.py` — exit 0 — PASS.
    4/4 DISTINCT-MECHANISM: US10,817,467 / US11,216,456 /
    US11,409,698 / US11,645,278, each with landed cite verified;
    behavioral court 3 passed, 0 failed.
4. `verify_disassembly_invariants.py` — exit 0 — PASS (not the
    exit-3 arm-E skip state). AE4 court CLEAN: census + linked
    witness + mutant + self-check + binary mutant; M-G03-E
    synthetic bnez injected into the witness scratch copy was
    flagged, original archive sha256-verified unchanged.
5a. `verify_dead_library.py` — exit 0 — PASS.
    consume-then-census law satisfied: 2 consumed, 26 justified
    exceptions; must-consume set intact
    (penstock_prefilter8, swar_all_zero_vl).
5b. `verify_dead_library.py --self-check` — exit 0 — PASS.
    MUT-A FIRED (names unconsumed_stub, exit 1);
    MUT-B FIRED-proof (parquet census 0 branchless tokens —
    scanner selective, not all-pass).
6a. `verify_cscb_sram.py` — exit 0 — PASS.
    measured 36,048 B <= 131,072 B, headroom 95,024 B
    (riscv32imc-unknown-none-elf release). Stack estimate is
    static-analysis only, labeled UNVERIFIED by the gate itself.
6b. `verify_cscb_sram.py --mutate=sram-bloat` — exit 0 —
    PASS (flip). Mutant 136,068 B > 131,072 B — the overflow
    flips; anti-vacuity holds. Exit 0 here means the flip
    succeeded, matching the gate's own semantics.
7. `make gate0` — exit 0 — PASS.
    ggen byte-stable replay proof PASS (sha e962af18... for
    generated_version.rs) + freestanding c99 header compile PASS.
8. `flip-tests.sh` (CLAUDE_PROJECT_DIR=/Users/sac/cgap) —
    exit 0 — PASS. 9/9 [PASS]: publish-guard fire/quiet/
    sandbox-mutation, cwd-guard fire/quiet/quiet,
    mtime-restore-guard fire/quiet, rep1-census-gate rc=0
    passed=3 (arms M-REP1-1 / M-INL-1 / branchy-ref self-check).

## G10 fix witnessed on disk

`crates/cgap-branchless/tests/branchless_census.rs:57-61` carries the
per-process `WORK` fix: comment `G10 root-cause fix: per-process WORK
so concurrent census invocations` + `static WORK: OnceLock<String>`
keyed on `std::process::id()`. Present; not a src/ file (dispatch
said `src/branchless_census.rs`; it is `tests/` — divergence noted).
## Known OPEN items (re-verified this session)

1. **Coverage floor RED** — `tests/gates/enforce_coverage_floor.py`
   is still red, but the observed exit this session is **2 (no
   coverage data at /tmp/c06-cov/.coverage.*)**, not the lane-c06
   recorded RED exit 1 with branch coverage 24.4% (boundary_gate),
   64.9% (receipt_lint), 52.6% (zero_datafusion_gate), 40.8%
   (zero_string_gate) vs the 90% floor. The four per-file numbers are
   receipt-cited from `lanes/lane-c06-scripts-gate-coverage.md:46-49`
   and were NOT re-measured this session (coverage data absent) —
   they are as-of-lane-c06, not current. The floor was not lowered;
   the gate's refusal arm is intact; it refuses even harder now (no
   data => nonzero exit). Status: OPEN, exit-value drift recorded.
2. **`zero_datafusion_gate.py --self-check` exit 1** — still true.
   Fresh run exit 1 via `shutil.Error` copytree explosion through the
   vendor symlink loop
   (`vendor/nanoarrow/python/subprojects/arrow-nanoarrow/python/...`
   recursion). Pre-existing; unchanged.
3. **`boundary_gate.py --deps` has NO non-vacuity arm** — still true
   (G16). Header confirms modes are
   `("--mutate", "--shim-court-only", "--deps", "--strings")` and the
   poison/self-check machinery is documented as wired to `--strings`
   only; `--deps` is a plain census with no flip court. Poison spec
   recorded per lane-g16.

## Divergences from dispatch

- Receipt filenames named as `lane-gNN` live in
  `docs/jira/v26.11.2/lanes/`, not `waves/` (g10, g16, c06
  confirmed on disk; g03/g04/g17/g18/b04 were not located under
  either dir with those exact names — cross-reference limited to the
  three found).
- `branchless_census.rs` is under `tests/`, not `src/`.
- Coverage-floor gate observed exit 2 (no data), vs dispatch's
  expected RED exit 1; the open item's substance (floor not met,
  floor not lowered) is corroborated via the lane-c06 receipt, but
  the current exit is 2.
- `--mutate=sram-bloat` is not a failure exit: the flip court exits 0
  ON SUCCESS of the flip (expected 136,068 B, got it).

## Standing

ALIVE — every verdict above is backed by a command + exit run this
session against the exact on-disk subject. Receipt file only; no code
touched; shared tree unmodified by this lane.
