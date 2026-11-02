# v2-10-cli-state — CLI re-witness receipt (gate-runner, V2-10)

- Subject: /Users/sac/cgap/crates/cli (cgap-cli 26.11.1), build root /tmp/v210-target (deleted via python3 shutil.rmtree)
- Date: 2026-11-01
- Lease: docs/jira/v26.11.2/waves/v2-10-cli-state.md — file was ABSENT on disk at lane start; proceeded under the dispatch contract (divergence recorded below).

## Ladder

| # | Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|---|
| 1 | build | `CARGO_TARGET_DIR=/tmp/v210-target cargo build` (crates/cli) | 0 | Finished dev profile in 14.49s |
| 2 | tests | `CARGO_TARGET_DIR=/tmp/v210-target cargo test` (crates/cli) | 0 | 12 lib + 23 cli_new_verbs + 6 mutation_kills + 1 zero_code_cli = 42 passed, 0 failed |
| 3 | version | `cgap self version` | 0 | "26.11.1" |
| 4 | verbs | 17 pairs, real handlers (no --help) | all 0 | see below |
| 5 | refusal | `cgap frobnicate` | 1 | unrecognized subcommand, structured error path fired |
| 6 | cleanup | shutil.rmtree('/tmp/v210-target') | 0 | lease debris removed |

## 17 new verbs — handler-green (all EXIT=0)

| Pair | Exit | Refusal flag witnessed |
|---|---|---|
| bont slot-lookup | 0 | rederived_agrees: true |
| bont pack-verify | 0 | all_slots_rederived: true |
| bn3 unify | 0 | matches 3/3 |
| bn3 eval | 0 | counters_latched 3 |
| gate dam-ingest | 0 | corrupt_refused: true (corrupt_status 0x43470001) |
| parquet pack-emit | 0 | digest chain emitted |
| parquet pack-check | 0 | tamper_refused: true |
| p28 attest-verify | 0 | over_ceiling_refused: true |
| crypto seal | 0 | tick_after_seal 1 |
| crypto receipt-verify | 0 | tamper_refused: true, verified: true |
| audit powl-stats | 0 | violating_log_refused: true |
| audit ocel-stats | 0 | monotonic_id_refused: true, reserved_activity_refused: true |
| git dag-verify | 0 | wrong_parent_refused: true |
| arrow slab-stats | 0 | reload_roundtrip: true |
| gpu info | 0 | available: true |
| beam info | 0 | harness cgap-beam spsc 2/0 |
| court verdict-run | 0 | 14/0 cgap-court verdict harness |
| self version | 0 | 26.11.1 |
| frobnicate | 1 | unknown-surface refusal |

## Standing

ALIVE. All commands run this session, first-hand, exit codes cited. No reds.

## Divergences

1. Lease file docs/jira/v26.11.2/waves/v2-10-cli-state.md did not exist at lane start (cat exit 1); this receipt now occupies that path.
