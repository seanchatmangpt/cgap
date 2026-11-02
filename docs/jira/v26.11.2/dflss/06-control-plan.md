# 06 Control Plan

DFLSS Control Plan for the v26.11.2 starDF campaign. Companion to the
charter `docs/jira/v26.11.2/14-dflss-charter.md` (SMART goals G1-G6,
CTQ flowdown, tollgates) and CTQ tree `15-ctq-tree.md`. The control
plan binds each campaign gate to a frequency, a fresh this-session
exit witness, and an OCAP (out-of-control action plan) — a gate row
without a run exit is UNVERIFIED, never PASS.

All exits below were run this session (2026-11-02) against
`/Users/sac/cgap` @ main, unless marked cited. Exit code law:
0=pass / 1=gate fail / 2=subject-missing or no-data / 3=loud skip.

## GATE matrix

| Gate | Subject / command | Frequency | Exit (witness) | OCAP |
|------|-------------------|-----------|----------------|------|
| GATE-01 | `python3 scripts/verify_disassembly_invariants.py` (arms C/D/E: census + linked witness + binary mutant) | per kernel change + nightly | 0 PASS — "AE4 disassembly-invariants court: CLEAN"; anti-vacuity M-G03-E flip witnessed in-run (synthetic bnez injected into scratch copy, flagged; original archive sha256 unchanged) | on fail: halt merge of the offending kernel fn; re-census via REP1 arm; if flip arm cannot fire, gate is VACUOUS (exit nonzero required) |
| GATE-02 | `python3 scripts/zero_string_gate.py` (+ `--self-check`) | per build + nightly | 0 (scanned=92 crates=22 hits=0 emitter_hits=2 allowed) ; self-check 0 — poison artifacts=1 hits=2 fired=True, clean hits=0, RESULT PASS | on fail: refuse landing; strings route through the vocab source `graph-derivation:ontologies/*/src/lib.rs` (9 tokens) |
| GATE-03 | `python3 scripts/verify_cscb_sram.py` | per kernel change | 0 PASS — 36048 B <= 131072 B, headroom 95024 B; report-only baseline delta -3136 B vs 39184 B | on fail: freeze ingress surface; re-run freestanding witness; frame bound stays UNVERIFIED as worst-case chain bound (static analysis only) |
| GATE-04 | `cd crates/csct && cargo run --release --bin cscb_c4_alloc` (+ `CSCB_MUTATE=alloc` flip) | per csct change + nightly | base exit 0 "C4 PASS: steady-state 0 allocs / 0 bytes over 1000000 frames"; flip exit 1 "ALLOC BUDGET BREACH ... -> exit 1" — both arms run fresh | on fail: typed breach exit 1; court self-reports VACUOUS if mutate mode saw no breach |
| GATE-05 | anti-vacuity battery: `bash .claude/hooks/flip-tests.sh` ; `python3 scripts/receipt_lint.py --strict` ; `python3 scripts/boundary_gate.py --self-check-deps` | per lane receipt + nightly | flip-tests exit 0, 9 PASS / 0 FAIL; boundary --self-check-deps exit 0 (clean=0, poison=1, both arms flipped); receipt_lint --strict exit **1** — `KeyError: 'rule'` crash (OUT-OF-CONTROL, see log) | flip-tests: a court that cannot flip is FAIL; receipt_lint crash is triaged below, floor not touched |
| GATE-06 | `make gate0` (ggen sync x2 byte-identical replay + freestanding C99 header compile) | per regen + release | 0 — header compile PASS, replay held | on drift: `rm ggen.lock && ggen sync run` twice + cmp (FM-PACK-008 fix) |
| GATE-07 | `python3 scripts/test_census.py --crate cgap-parquet` | per test-wave + nightly | 0 — CENSUS: 14 targets, PASS 14, FAIL 0, COMPILE-ABORT 0, TIMEOUT 0 | compile-abort is not a failure count; census distinguishes exit-1-abort from real FAIL per W672 law |
| GATE-08 | flip-tests hook wiring: `.claude/settings.json` hook entry -> `bash .claude/hooks/flip-tests.sh` | per session (hook fires on lane events) | wired at `.claude/settings.json:65`; script run fresh this session exit 0, 9/9 PASS | if wiring is removed or script returns nonzero on a clean tree, the hook arm is VACUOUS -> restore and re-run flip-tests |
| GATE-09 | `python3 tests/gates/enforce_coverage_floor.py` | per scripts-wave + nightly | **2 (RED, no-data)** — "FLOOR: no coverage data at /tmp/c06-cov/.coverage.*"; no fresh coverage run this session | OCAP per C06: raise coverage, never lower floor; regenerate `.coverage.*` under /tmp/c06-cov, then re-run; exit 2 (no data) vs C06's exit 1 (measured red 24.4/40.8/52.6/64.9% vs floor 90) — both are honest RED, neither rounds up |

## Out-of-control log (this session)

| Trip | Evidence | Disposition |
|------|----------|-------------|
| octopus mutation court intermittent FAIL (`octopus_merge.rs:356` region) | re-run this session: `cargo test -p cgap-git --test octopus_merge_court` -> exit 0, 5 passed / 0 failed — intermittent trip did NOT reproduce | stays on watchlist as intermittent; a single green run does not clear an intermittent, it only records a non-reproducing sample |
| C3 SLA coordinate FAIL (b02) | `lanes/lane-b02-cscb-c3.md:13,19` — min 42.4 s vs 25.0 ms target; correctness + anti-vacuity courts ALIVE | honest FAIL standing recorded; MV-5 re-scope pending — no `lane-mv-5` receipt exists on disk yet (checked this session), so re-scope is UNLANDED |
| C6 throughput typed FAIL (b05) | `lanes/lane-b05-cscb-c6.md:34,49` — 248M facts/s vs 800M target, exit 1; mutant flip witnessed | honest FAIL standing recorded; re-scope same MV-5 dependency |
| receipt_lint --strict crash | `python3 scripts/receipt_lint.py --strict` -> exit 1, `KeyError: 'rule'` (schema mismatch inside the linter, not a receipt defect signal) | OUT-OF-CONTROL; owner: scripts gate surface; fix is a linter repair, not a floor change |
| GATE-09 coverage floor RED | exit 2 no-data this session; C06 measured exit 1 at 24.4/40.8/52.6/64.9% vs floor 90 (`lanes/lane-c06-scripts-gate-coverage.md:46-53`) | RED is the designed state until coverage data is regenerated; floor stays at 90, never lowered |

## Escalation law

- A gate exit 1 on the shared tree freezes landing for the owning
  surface (compile-freeze SLA, ~10 min to fix or revert).
- A gate that cannot flip is FAIL; a green run with no flip arm is
  VACUOUS. Every row above names its flip witness.
- Perf numbers in the charter remain TARGETs until measured with the
  stated method; measured FAILs (C3, C6) are recorded as FAILs, never
  re-scoped by prose — re-scope lands only via a receipt (MV-5).
- Exit-2 (no-data) rows are RED, not PASS: absence of coverage data is
  itself out-of-control for the coverage floor.
