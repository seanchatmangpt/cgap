# DoD-17 / FS-14 — gate matrix re-verification (cold run)

Date: 2026-11-02. Repo /Users/sac/cgap (main, canonical). Read-only
gate execution; writes ONLY this file; no git commands. Cold
re-verification of
`docs/jira/v26.11.2/audit/prepublish-reconciliation.md` (C-10) for
the certificate. Shared default `target/` used for cargo (matches
C-10 divergence 2 disclosure; no /tmp lease needed — no mutations
applied to any tree this session; courts mutate /tmp copies
internally, gate_battery self-checks proved that).

## Commands + exits (all this session)

| # | Fence | Command | Exit | Evidence |
|---|---|---|---|---|
| 1 | Workspace build | `cargo build --workspace --all-targets` | 0 | Finished dev profile 11.49s; warning census below |
| 2 | Five-crate fence | `cargo test -p cgap-ffi -p cgap-git -p cgap-storage -p cgap-bn3 -p cgap-p28` | 0 | 64 suites, 568 passed / 0 failed total |
| 3a | Gate battery | `python3 scripts/gate_battery.py` | 0 | 4/4 PASS (boundary E=0/15 crates; zero-string 1 loud rmeta [SKIP]; cscb-sram report-only refusal-as-designed; ffi 12==12 symbols) |
| 3b | Self-check | `python3 scripts/gate_battery.py --with-self-check` | 0 | 8/8 PASS; boundary both arms flipped; cscb bn3-pin mutant refused (NaMatches 1081352 != pin 2162696); ffi byte-exact re-restore held |
| 4a | breach | `cargo test -p cgap-git --test breach_regression_court` | 0 | 5/5 |
| 4b | fa9 | `cargo test -p cgap-ffi --test fa9_commit_gating` | 0 | 7/7 (run under owning crate cgap-ffi per C-10 divergence 1) |
| 4c | quota | `cargo test -p cgap-git --test quota_alloc_purity_court` | 0 | 1/1 |
| 4d | posh | `cargo test -p cgap-git --test posh_alloc_sentry_court` | 0 | 1/1 |
| 5a | riscv no-flag | `cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features` | 101 | typed compile_error! at ref_store.rs:232 (implcell critical-section refusal) — DESIGNED flip |
| 5b | riscv opt-in | `RUSTFLAGS='--cfg implcell_critical_provided' cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features` | 0 | builds clean, 11 lib warnings (same census as host) |
| 6a | CLI build | `cd crates/cli && cargo build` | 0 | standalone crate |
| 6b | CLI test | `cd crates/cli && cargo test` | 0 | 44 tests: 12 lib + 25 cli_new_verbs + 6 mutation_kills + 1 zero_code_cli |
| 6c | fleet-ops | `cd crates/cgap-fleet-ops && cargo test` | 0 | 4 passed / 0 failed |
| 7 | metadata | `cargo metadata --no-deps` (repo root) | 0 | 23 members; fleet-ops/ocel2/cli absent (own workspaces) |

## Warning census (step 1, --all-targets) — for FS-17

Baseline (lib targets) MATCHES the pre-existing expectation exactly:

| Source | Count | Status |
|---|---|---|
| cgap-git lib | 11 | PRE-EXISTING (matches post-WARN-01 delta 11) |
| bench (manifest) | 3 | PRE-EXISTING (unused deps cgap-ffi/cgap-git/cgap-storage, Cargo.toml:20-22) |
| cgap-storage test `journal_fault_matrix_court` | 1 | test-target-only (`tests/journal_fault_matrix_court.rs:359`, unused `mut`) |
| cgap-git test targets | ~120 (many duplicates of the 11 lib warnings under `cfg(test)`) | test-target-only, spread across 15 suites (soc_full_chip_court 48, quota_alloc_purity_court 35, andon_liveness_court 27, ...) |

NOTE: `--all-targets` compiles test targets, which C-10's lib-only
step-1 build did not. The ~120 test-target warnings therefore have NO
prior all-targets baseline in any receipt — pre-existence is
UNVERIFIED, visibility is `--all-targets`-only. No lib-target
regression: lib census is exactly 11 + 3.

Known-cosmetic outcomes:

- octopus child line: OBSERVED this run (unlike C-10). Child runner
  inside octopus_merge_court prints `test result: FAILED. 0 passed;
  1 failed` for `mutation_conflict_injection_flip` (panics by design:
  ANDON-REFUSAL exit 101 class, octopus_merge.rs:366), the parent
  suite asserts the panic and reports `... ok`, outer result
  `ok. 5 passed` — overall exit 0. Classified cosmetic per dispatch.
- wave9_a07 environmental: NOT observed. Not chased.
- fa9 101 from GAP-04: not reproduced; fa9 7/7 + full ffi suite green.

## Courts run (anti-vacuity, all flipped)

| Court | Mutation | Flip witness |
|---|---|---|
| boundary:self-check | poison dep injected | both arms flipped (0/1) |
| cscb-sram:self-check | bn3-pin mutant | refused exit 1 (1081352 != 2162696 B) |
| ffi-surface:self-check | byte-exact re-restore | held, exit 0 |
| zero-string | rmeta [SKIP] | loud, 1 row |
| riscv no-flag leg | missing implcell opt-in | compile_error! ref_store.rs:232, exit 101 (DESIGNED) |

## Fences audited (must-not-regress)

R-01 build gate (held, exit 0); WARN-01 delta (held, lib 11 exact);
R-05 battery rollups (held, 4/4 + 8/8 same rollup lines); F-01 riscv
legs (held, 101/0 pair — with the divergence below); fa9 post-
R-ADV10FIX (held, 7/7); CLI 44-class (held); metadata 23-member (held).

## Standing

**ALIVE.** Every cell carries a command + exit run this session
against the canonical tree. Zero repairs applied; zero failures; one
command-form divergence (below) resolved by bounded re-poll.

## Divergences (typed)

1. Opt-in riscv leg: C-10's command form (default features + cfg
   flag) now exits 101 with TWO typed refusals (E0463 no std + NEW
   compile_error! at `crates/cgap-git/src/ref_store.rs:725` refusing
   default `std` feature on an OS-less target). The lawful opt-in
   command now requires `--no-default-features` in addition to the
   cfg flag. Green leg re-run with both -> exit 0. This is a FENCE
   CHANGE since C-10's receipt (likely from commit a0431b70
   "ref_store width-gated atomics"); the C-10 5b row is superseded
   by this row. Errata against C-10, per ERRATA-never-rewrite law.
2. Wall-clock captured only where tooling reports it (build 11.49s
   Finished; court durations inside `test result` lines: posh 5.83s,
   breach 0.38s, fa9 0.49s, packfile_fuzz 47.58s, cli_new_verbs
   28.46s). Whole-command wall-clock not separately timed — recorded
   as disclosure, not silently rounded.
3. Metadata census first ran from crates/cgap-fleet-ops (own
   workspace, 1 member) by command-ordering error; re-run from repo
   root -> 23 members. The fleet-ops-dir result was discarded, not
   recorded as evidence.

JSON terminus: {"lane":"FS-14/DOD-17","files_written":["docs/jira/v26.11.2/dod/dod-17-gate-execution.md"],"commands":["cargo build --workspace --all-targets -> 0 (lib warnings 11 git + 3 bench manifest, pre-existing; ~120 test-target warnings UNVERIFIED pre-existence, --all-targets-only)","cargo test -p cgap-ffi -p cgap-git -p cgap-storage -p cgap-bn3 -p cgap-p28 -> 0 (568 passed / 0 failed, 64 suites)","gate_battery.py -> 0 (4/4)","gate_battery.py --with-self-check -> 0 (8/8)","breach_regression_court -> 0 (5/5)","fa9_commit_gating (-p cgap-ffi) -> 0 (7/7)","quota_alloc_purity_court -> 0 (1/1)","posh_alloc_sentry_court -> 0 (1/1)","riscv --no-default-features no-flag -> 101 typed ref_store.rs:232 (DESIGNED)","riscv opt-in RUSTFLAGS+--no-default-features -> 0","cd crates/cli && cargo build -> 0 ; cargo test -> 0 (44)","cd crates/cgap-fleet-ops && cargo test -> 0 (4)","cargo metadata --no-deps -> 0 (23 members)"],"courts_run":["boundary self-check (flipped)","cscb-sram bn3-pin mutate (refused exit 1)","ffi-surface byte-exact re-restore (held)","riscv no-flag compile_error refusal (DESIGNED flip, 101)"],"standing":"ALIVE","divergences":["riscv opt-in leg now requires --no-default-features (new std-feature compile_error at ref_store.rs:725); C-10 5b command form superseded; errata recorded","wall-clock only as reported by tooling, not separately timed","metadata first run from fleet-ops dir discarded; re-run from root"]}
