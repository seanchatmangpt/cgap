# AU7 — Anti-Vacuity and Gates (Phase 1)

Measured facts unless labeled ANALYSIS. Paths relative to /Users/sac/cgap.

## 1. Census

Gate scripts: 23 Python scripts in scripts/ (12,321 LOC). C-wave harnesses:
- gate_cargo_test.py (194 LOC, AE3): parses `test result:` lines of `cargo test -p`;
  exit 3 = VACUOUS_TEST_EVASION (passed==0 AND filtered_out==0). Quotas (~80% measured): branchless 28 (meas. 35), parquet 50 (floor; UNMEASURABLE — compile abort), arrow 153
  (meas. 192), cgap 12 (meas. 15).
- verify_dead_library.py (243 LOC, AE5), BR12 consume-then-census: every cgap-branchless
  `pub fn` must appear in rcgu objects of a production consumer (gate/parquet/p28/ffi),
  else exit 1 DEAD_LIBRARY_SHOWROOM. Exceptions (dead_library_exceptions.json): per-entry
  justification + plan; stale entry exits 1; MUST_CONSUME {penstock_prefilter8,
  swar_all_zero_vl} never exceptable.
- verify_disassembly_invariants.py (444 LOC, AE4), 4-arm riscv32imc court: (A) opcode
  census, (B) llvm-objdump linked witness, (C) mutant court (if-injection into select.rs
  cmov_u64 must flip), (D) self-check (branchy naive_if_cmov must be flagged else
  VACUOUS). Residual allowlist: 3 symbols (select_slice, cmov_u64, eytzinger_restore)
  with typed RV32 ISA-ceiling cites; rows emitted, never silent.
- verify_ost_design_around.py (365 LOC, AE6/G7) + crates/cgap/tests/patent_behavioral_court.rs
  (3 tests, one per patent row, each naming its /tmp-sandbox flip mutation).
- crates/cgap/tests/alloc_boundary.rs: AE7 budget courts; Court 0 is a positive control
  labeled "anti-vacuity; MUST flip".
- riscv32imc LSP courts live in the lsp-max worktree (handoff seed: countermeasures/
  lane-f12-residual-seed.md). EXCLUDED here per scope; noted separately.

Tests workspace-wide (excl. target): 92 integration files in crates/*/tests with 457
`#[test]` fns; repo-wide 1,259 (159 `#[cfg(test)]` modules). Vocabulary (07-court-corpus.md,
01-release-gates.md): differential-oracle courts (KD-12/KD-14), anti-vacuity-mutant arms,
flip-witness law ("a level with no flip witness is not measured"), compile-abort census
law (exit 1 with 0 tests = compile-abort, not test failure), hostile-prosecution framing
in .claude/agents/gate-runner.md and run-gates/SKILL.md.

## 2. Anti-vacuity mechanisms

Shared discipline (07-court-corpus.md, five laws): /tmp-sandbox-only mutation;
sha256-exact restore (mismatch = typed RESTORE-MISMATCH exit 1); mtime-bump law
(byte-exact restore preserving mtimes does not trigger cargo rebuild — witnessed in
docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184); exit contract 0/1/2/3 with
"no mutation leg = VACUOUS, exits nonzero"; no Process.sleep (bounded polling).
- Flip-before-PASS: 01-release-gates.md:79 "A level with no flip witness is not measured;
  it is [SKIP] loud"; :85 demarks threshold-met-without-flip.
- REP1 blindness fix: exit 3 fires on passed==0 && filtered_out==0 (filter evasion
  otherwise looks green).
- Compile-the-mutant: AE4 Court C compiles the mutant in /tmp and requires the census to
  flip to exit-1; non-flip = typed VACUOUS; skip is loud.
- Vacuous refusals: dead-library MUT-B control (parquet objects must yield ZERO
  branchless tokens; over-match = VACUOUS); vacuous_targets counter; alloc Court 0 must
  fail. Two allowlists only (dead_library_exceptions.json, RESIDUAL_ALLOWLIST), justified
  per entry with plans.

Witnessed flips this wave (APPROXIMATE — grep of FLIPPED/flip-witnessed across v26.11.2
waves/repairs/countermeasures hits ~118 lines; explicit FLIPPED receipts in 14+ wave
files incl. repair-01 (select_slice 11x), repair-16 (3/3→0/3), repair-18/20, L1/L2 pareto
rows, measured-claims rows 23/25). Grep-derived, not verified unique courts.

## 3. Test density (measured)

Workspace .rs excl. target: 123,547 LOC; tests-path 28,923; non-tests 94,624 (ratio ~0.31).
| crate | src LOC | test LOC | ratio |
|---|---|---|---|
| cgap-core | 1,423 | 2,612 | 1.84 |
| cgap-branchless | 1,650 | 1,753 | 1.06 |
| cgap-parquet | 14,552 | 3,930 | 0.27 |
| cgap-arrow | 10,124 | 2,202 | 0.22 |
| cgap-ffi | 6,520 | 1,367 | 0.21 |
| cgap-bn3 | 10,039 | 914 | 0.09 |
| cgap-gate | 1,883 | 153 | 0.08 |

Low-density outliers: cgap-bn3, cgap-gate; quotas cover only 4 of 24 crates.

## 4. ANALYSIS — rigor shape vs safety-critical practice

- Requirements-tested: courts bind each requirement (G1-G8, L1-L5, patent row) to a named
  file:line subject + kill signal — DO-178C requirements coverage, enforced by parse.
- Mutation coverage: DO-178C has no mutation objective; cgap makes it a gate law ("a court
  that cannot flip is FAIL"), exceeding DO-178C and enterprise CI where green-with-dead-
  tests passes.
- Non-vacuity proofs: per-harness negative/positive controls (MUT-B over-match,
  naive_if_cmov, alloc Court 0, restore-corruption self-check) — each harness proves it
  can detect its fault class; approximates DO-178C tool qualification (TQL-5).
- Distinctive cgap element: anti-vacuity as first-class — a green gate WITHOUT a witnessed
  flip is FAIL (flip-before-PASS), plus evidence-hygiene laws (sha256-exact restore +
  mtime bump) covering the stale-mtime false-re-green class industry leaves implicit;
  stronger than typical enterprise CI.
- Gaps: quotas cover 4/24 crates; density uneven (bn3 0.09, gate 0.08); flip counts are
  receipt-grep evidence, not machine-aggregated; no trybuild/compile-fail tests found
  under crates/*/tests.
