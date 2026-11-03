# Pre-publish reconciliation — lane C-10 (=G-17, final reconciliation)

Date: 2026-11-02. Repo /Users/sac/cgap (main, canonical). Read-only
verification lane; writes ONLY this file; no git commands. Dispatch
references re-read from disk: gap-04-final-verification.md (PARTIAL,
superseded items per its ERRATA), lane-r05-gate-battery-recount.md,
pub-01-dry-run-matrix.md.

## Commands + exits (all this session)

| # | Fence | Command | Exit | Evidence |
|---|---|---|---|---|
| 1 | Workspace build | `cargo build --workspace` | 0 | "Finished dev profile"; warnings: 11 cgap-git lib (matches post-WARN-01 delta 12→11) + 3 pre-existing bench manifest = 14 |
| 2a | cgap-ffi | `cargo test -p cgap-ffi` | 0 | all green incl. fa9; zero FAILED lines in captured log |
| 2b | cgap-git | `cargo test -p cgap-git` | 0 | all green |
| 2c | cgap-storage | `cargo test -p cgap-storage` | 0 | all green |
| 2d | cgap-bn3 | `cargo test -p cgap-bn3` | 0 | all green |
| 2e | cgap-p28 | `cargo test -p cgap-p28` | 0 | all green |
| 3a | Gate battery | `python3 scripts/gate_battery.py` | 0 | 4/4 PASS (boundary E=0; zero-string one loud rmeta [SKIP]; cscb-sram report-only REFUSED-as-designed; ffi-surface 12==12) |
| 3b | Battery self-check | `python3 scripts/gate_battery.py --with-self-check` | 0 | 8/8 PASS; boundary both-arms flipped; cscb bn3-pin mutant refused exit 1 (NaMatches 1081352 B != pin 2162696 B); ffi re-restore byte-exact held |
| 4a | breach | `cargo test -p cgap-git --test breach_regression_court` | 0 | 5/5 |
| 4b | fa9 | `cargo test -p cgap-ffi --test fa9_commit_gating` | 0 | 7/7 |
| 4c | quota | `cargo test -p cgap-git --test quota_alloc_purity_court` | 0 | 1/1 |
| 4d | posh | `cargo test -p cgap-git --test posh_alloc_sentry_court` | 0 | 1/1 |
| 5a | riscv no-flag | `cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features` | 101 | typed refusal — compile_error! at `crates/cgap-git/src/ref_store.rs:232` ("no CAS-capable atomic... rebuild with RUSTFLAGS=--cfg implcell_critical_provided") — DESIGNED flip |
| 5b | riscv with flag | `RUSTFLAGS='--cfg implcell_critical_provided' cargo build -p cgap-git --target riscv32imc-unknown-none-elf` | 0 | builds clean |

## Standalone legs (step 6) + metadata (step 7)

| # | Fence | Command | Exit | Evidence |
|---|---|---|---|---|
| 6a | CLI build | `cd crates/cli && cargo build` | 0 | standalone crate (own workspace) |
| 6b | CLI test | `cd crates/cli && cargo test` | 0 | 44 tests total: 12 lib + 25 cli_new_verbs + 6 mutation_kills + 1 zero_code_cli (44-class count matches dispatch) |
| 6c | fleet-ops | `cd crates/cgap-fleet-ops && cargo test` | 0 | 4 passed / 0 failed |
| 7 | metadata | `cargo metadata --no-deps` | 0 | 23 workspace members; fleet-ops ABSENT, ocel2 ABSENT (both carry own `[workspace]`) |

## Known-cosmetic watchlist outcomes

- octopus FAILED-line child artifact: NOT observed this session (exit 0,
  zero FAILED lines captured; child-panic artifact absent). Not chased.
- wave9_a07 environmental refusal: NOT observed. Not chased.
- GAP-04's `fa9_rename_race_denied` 101: NOT reproduced — full
  cgap-ffi suite exit 0 incl. fa9 (consistent with GAP-04 ERRATA item 3
  supersession via lane-r-adv10fix).

## Courts run (anti-vacuity)

| Court | Mutation | Flip witness |
|---|---|---|
| boundary:self-check | poison dep injected | both arms flipped (clean=0, poison=1) |
| cscb-sram:self-check | bn3-pin mutant | refused exit 1 (NaMatches 1081352 B != pin 2162696 B) |
| ffi-surface:self-check | byte-exact restore court | held, exit 0 |
| zero-string | rmeta [SKIP] | loud, 1 row (rustc metadata member) |
| riscv refusal leg | no-flag build | compile_error! ref_store.rs:232, exit 101 (DESIGNED) |

## Fences audited

R-01 build gate (held), R-05 battery rollups 4/4 + 8/8 (held, same
rollup lines), WARN-01 warning delta 11 git-lib (held), GAP-04 ERRATA-2
lints fix (held: no unexpected_cfgs warning this build), F-01 riscv
width-gate legs (held: 101/0 pair reproduced), fa9 post-R-ADV10FIX
green (held: 7/7 + full suite 0).

## Standing

**ALIVE.** Every cell above carries a command + exit run this session.
Zero repairs applied; zero failures traced to wave-lane-owned files;
no cross-lane unblocks needed.

## Divergences (typed)

1. Dispatch named `cargo test -p cgap-git --test fa9_commit_gating` —
   fa9_commit_gating is a cgap-FFI test target
   (`crates/cgap-ffi/tests/fa9_commit_gating.rs`); under cgap-git cargo
   refuses exit 101 "no test target named". Run under the owning crate:
   `-p cgap-ffi` -> 7/7 exit 0. Doc yields; divergence recorded.
2. Step-1 build ran against the shared `target/` (default dir) rather
   than a /tmp lease — one-time deviation at lane start; all test
   fences used /tmp/c10-target, riscv legs /tmp/c10-rv-target.
3. Per-crate test logs captured tail-only (last 25 lines), so
   per-suite pass counts beyond the targeted courts are evidenced by
   exit code, not by captured count lines (fleet load: concurrent
   bench lanes on shared tree).
4. Ledger row for dispatch-ledger.jsonl NOT appended: contract scopes
   writes to this receipt only; coordinator owns the ledger append.

## Lease cleanup

- /tmp/c10-target and /tmp/c10-rv-target deleted via python
  shutil.rmtree at lane end (below command), rm -rf not used.

JSON terminus: {"lane":"C-10","files_written":["docs/jira/v26.11.2/audit/prepublish-reconciliation.md"],"commands":["cargo build --workspace -> 0 (11 git-lib + 3 bench-manifest warnings, all pre-existing)","cargo test -p cgap-ffi -> 0","cargo test -p cgap-git -> 0","cargo test -p cgap-storage -> 0","cargo test -p cgap-bn3 -> 0","cargo test -p cgap-p28 -> 0","gate_battery.py -> 0 (4/4)","gate_battery.py --with-self-check -> 0 (8/8)","cargo test -p cgap-git --test breach_regression_court -> 0 (5/5)","cargo test -p cgap-git --test fa9_commit_gating -> 101 no-such-target (divergence; correct: -p cgap-ffi -> 0, 7/7)","cargo test -p cgap-git --test quota_alloc_purity_court -> 0 (1/1)","cargo test -p cgap-git --test posh_alloc_sentry_court -> 0 (1/1)","cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features -> 101 typed implcell refusal (ref_store.rs:232)","RUSTFLAGS=--cfg implcell_critical_provided ... -> 0","cd crates/cli && cargo build -> 0 ; cargo test -> 0 (44 tests)","cd crates/cgap-fleet-ops && cargo test -> 0 (4)","cargo metadata --no-deps -> 0 (23 members, fleet-ops/ocel2 absent)"],"courts_run":["boundary self-check (flipped)","cscb-sram bn3-pin mutate (refused exit 1)","ffi-surface byte-exact re-restore (held)","riscv no-flag compile_error refusal (DESIGNED flip, 101)"],"standing":"ALIVE","divergences":["fa9_commit_gating lives in cgap-ffi, not cgap-git (dispatch command form wrong; run under owner, 7/7)","step-1 build used shared target/ not /tmp lease (disclosed)","per-crate counts evidenced by exit code only (tail-truncated logs)","dispatch-ledger.jsonl not appended (contract scopes writes to receipt only)"]}
