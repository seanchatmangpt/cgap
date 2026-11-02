# Lane DF-5 / dflss-05 — Control Plan

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Status: ALIVE

## Files written

- `docs/jira/v26.11.2/dflss/06-control-plan.md` (created; cross-links
  `docs/jira/v26.11.2/14-dflss-charter.md`)
- `docs/jira/v26.11.2/lanes/lane-dflss-05-control.md` (this receipt)

## W1 Subject

The campaign control plan binding 9 gates to frequency / fresh exit
witness / OCAP, plus the session out-of-control log.

## W2 Commands + exits (all run this session)

| Command | Exit |
|---------|------|
| `python3 scripts/verify_disassembly_invariants.py` | 0 (court CLEAN; binary-mutant flip witnessed in-run) |
| `python3 scripts/zero_string_gate.py` | 0 (hits=0) |
| `python3 scripts/zero_string_gate.py --self-check` | 0 (poison fired=True, clean hits=0) |
| `python3 scripts/verify_cscb_sram.py` | 0 (36048 B <= 131072 B) |
| `cargo run --release --bin cscb_c4_alloc` (in crates/csct) | 0 (0 allocs / 1M frames) |
| `CSCB_MUTATE=alloc` same bin | 1 (breach flip: allocs=1 bytes=8 -> exit 1) |
| `bash .claude/hooks/flip-tests.sh` | 0 (9 PASS / 0 FAIL) |
| `python3 scripts/boundary_gate.py --self-check-deps` | 0 (clean=0, poison=1, both arms flipped) |
| `python3 scripts/receipt_lint.py --strict` | **1** (KeyError 'rule' crash) |
| `make gate0` | 0 (header compile PASS, replay held) |
| `python3 scripts/test_census.py --crate cgap-parquet` | 0 (14 PASS, 0 abort) |
| `python3 tests/gates/enforce_coverage_floor.py` | **2** (no coverage data at /tmp/c06-cov/.coverage.*) |
| `cargo test -p cgap-git --test octopus_merge_court` | 0 (5 passed / 0 failed) |

## W3 Standing

ALIVE — every gate row in 06-control-plan.md carries a this-session
exit. Two honest RED rows recorded as designed states (coverage floor
exit 2 no-data; receipt_lint exit 1 crash — both logged as
out-of-control, not passed over).

## W4 Verification ladder

Gates run directly against the shared tree (read-only gates) or in
their own crates (csct release run). Flip witnesses: GATE-01 binary
mutant in-run; GATE-02 --self-check poison; GATE-04 CSCB_MUTATE=alloc
breach; GATE-05 boundary --self-check-deps both arms; octopus court
non-reproduction recorded as a watchlist sample, not a clear.

## W5 Divergences

1. Dispatch named "GATE-03 sram-bloat" — no target of that name exists
   on disk (Makefile, scripts, justfile all absent); verify_cscb_sram.py
   run covers the SRAM row. Recorded, not invented.
2. GATE-09: dispatch suggested exit 1 (C06) or 2 (AUD-02) — this
   session's fresh run is exit 2 (no coverage data); C06's measured
   24.4/40.8/52.6/64.9% figures are cited, not re-measured (no fresh
   coverage run this session).
3. MV-5 re-scope: no `lane-mv-5*` receipt exists on disk (checked);
   C3/C6 FAIL dispositions cite b02/b05 receipts directly.
4. Dispatch said "14-dflss.md"; actual charter file is
   `14-dflss-charter.md` — cross-linked as-is.
5. receipt_lint --strict KeyError 'rule' crash is a NEW finding this
   session (prior lanes saw lint pass); logged out-of-control.

## W6 Fences (must-not-regress)

- Root workspace / lib.rs / Cargo.toml: untouched (no git commands, no
  shared-surface edits). Gates run were read-only against the tree.
- Existing gates could not regress: no source diff produced this lane.
- Build roots: cargo runs used the default shared target for csct
  (`crates/csct/target`) and the workspace `target/` for cgap-git —
  no per-lane lease dir minted; nothing to clean.

## W7 Identity / authority / courts

- Identity: lane DF-5 (dflss-05), repo /Users/sac/cgap @ main
  (9a730320), docs-only diff, 2 files.
- Authority: DF-5 dispatch prompt; owned files =
  `docs/jira/v26.11.2/dflss/06-control-plan.md` +
  `docs/jira/v26.11.2/lanes/lane-dflss-05-control.md` only.
- Courts: GATE-01..09 all run with flip witnesses as tabulated in
  06-control-plan.md (W2 table above carries each exit); anti-vacuity
  arms exercised: zero-string poison, CSCB_MUTATE breach, boundary
  deps-poison, disassembly binary mutant. No silent skips.
- Byte receipts: 06-control-plan.md sha256 fbd557a5cfe4e75e (6126 B);
  this receipt sha256 003e667a4508e2c5 (3454 B pre-W7; W7 append
  supersedes that count).
