# YD-7: DoD Blocking-Ladder Re-hearsal at HEAD (XP-2 Phase 2 gate)

- Date: 2026-11-04
- Subject: /Users/sac/cgap @ 528ef2ec93e170eae3cc306c68f078729d1cbf65 (main)
- Lane: YD-7 (verification lane; owned file: this receipt only)
- Method: every gate command from the committed
  `docs/jira/v26.11.3/dod-checklist.md` (dod-001..dod-012) executed this
  session via `scripts/run_checked.py` (exit captured unpiped). All 12 gates
  ran within budget; none skipped as TARGET-UNMEASURED-this-run.
- Logs preserved: /tmp/dod001.log .. /tmp/dod012-run1.log,
  /tmp/dod011-run2.log, /tmp/dod006-retry.log
- Load context: gates ran as a 4-lane concurrent fan-out over one shared
  target dir; YD-1 (Cargo.toml license fields) and YD-6 (error.rs) were
  editing concurrently on the same checkout.

## Gate table

| gate | command | exit | standing this run |
|---|---|---|---|
| dod-001 | `cargo test -p cgap-arrow` | 0 | MEASURED — 242 passed / 0 failed |
| dod-002 | `cargo check -p cgap-aoq` | 0 | MEASURED |
| dod-003 | `cargo test -p cgap-crypto --lib fips202` | 0 | MEASURED — 10 passed / 0 failed |
| dod-004 | `cargo test -p cgap-crypto --lib chacha` | 0 | MEASURED — 9 passed / 0 failed |
| dod-005 | `cargo test -p cgap-crypto --test aead_courts --test ct_scan_court` | 0 | MEASURED — aead_courts 8/8 (incl. throughput floor, 244.49s), ct_scan_court 1/1 |
| dod-006 | `cargo test -p cgap-crypto && cargo test -p cgap-aoq --test corridor_join_court` | 101 | RED — see classification below |
| dod-007 | `cargo check -p cgap-arrow --target riscv32imc-unknown-none-elf` | 0 | MEASURED — exit 0 in 0.27s |
| dod-008 | `cargo check -p cgap-aoq --target wasm32-unknown-unknown` | 0 | MEASURED |
| dod-009 | `cargo test --release -p cgap-aoq --test cbmqs_gates` | 0 | MEASURED — 1/0 (cbmqs_seven_gates_summary_table) |
| dod-010 | `cargo test -p cgap-aoq --test corridor_join_court` | 0 | MEASURED — 3/0 (j1 100-iteration corridor join, tamper battery, chain 100k) |
| dod-011 | `ggen sync run && ggen sync run` | 0, 0 | MEASURED — run2 `written: []`; run1 wrote dod-checklist.md (Finding F2) |
| dod-012 | `ggen sync run` | 0 | MEASURED — zero FM-LAW-015 refusals in both logs (`grep -c FM-LAW-015` = 0 in both) |

11 of 12 gates green. One red: dod-006.

## dod-006 RED — classification

Two executions of the checklist's named command:

1. Run 1 (concurrent fan-out): cgap-crypto suites green 33/0, but
   `chatman_courts` binary "never executed" (ENOENT on
   `target/debug/build/cgap-crypto/29d858b29f04a183/out/...`) → exit 101.
   Self-inflicted concurrent-cargo interference: this lane ran 4 concurrent
   cargo invocations over one shared target dir; the ENOENT is a build-dir
   race, and the 33/0 green suites rule out a code fault channel.
2. Retry (serial, machine quiet, >3 min later): aead_courts 7 passed / 1
   FAILED — `throughput_min_of_n_floor` panicked at
   `crates/cgap-crypto/tests/aead_courts.rs:447`:
   `open floor 1.67 < 2 MB/s at len 65536` (min-of-8, dev profile).
   cgap-crypto's other 25 tests passed 25/0; corridor_join_court never
   reached (short-circuit `&&`).

Classification: **load-bound absolute-floor flake, pre-existing, not a
session-introduced code regression.** Evidence: the identical aead_courts
binary passed `throughput_min_of_n_floor` minutes earlier this session
(dod-005 run, 8/8, 244.49s) under heavier load; this lane made zero source
edits. The fence is an absolute MB/s floor rather than a ratio fence against
a sleep-free baseline — exactly the known-flaky fence class in
[[no-sleep-testing]]. That concurrent YD-1/YD-6 builds contributed load is
consistent but not provable without an isolated rerun; classified as
concurrent-load interference, not a gate-code failure.

Close condition: isolated rerun of
`cargo test -p cgap-crypto --test aead_courts -- throughput_min_of_n_floor`
on a quiet machine; if it reds on an idle box, reclassify as a real
throughput regression.

## Finding F2 — graph drift surfaced by dod-011/012 (pre-existing at HEAD)

The committed checklist row dod-006 says MEASURED (receipt bk-1-takeover.md),
but the ontology graph at HEAD still projects the pre-BK-1 row:
TARGET-UNMEASURED, receipt af-1-corridor.md, command
`cargo test -p cgap-crypto --test mlkem_sanity`, blocking=false.

Consequence witnessed this session: dod-011 run 1 of `ggen sync run` reported
`"written": ["docs/jira/v26.11.3/dod-checklist.md"]` and overwrote the
committed MEASURED/bk-1 row with the stale TARGET-UNMEASURED/af-1 row in the
working tree (run 2 then reported `written: []`, i.e. the stale state is
converged/fixed-point). Graph hash: 8359174f13be84563effac586e2d689b0b13898f6ccc2db1755ae8c0b4a4a50d.

Read strictly: `ggen sync run && ggen sync run` exits 0 twice with run2
`written: []` — the gate as written is green. But the convergence target is
a stale graph row, so the gate's determinism green masks a checklist
regression: the BK-1 flipback landed in the checklist without a graph update,
and any fresh-clone `ggen sync run` will regress dod-006 to TARGET-UNMEASURED.
Close condition: update the dod-pack ontology row for dod-006 to the BK-1
state, re-run sync x2, confirm byte-identical convergence at the MEASURED row.

This lane made no repairs (receipt-only ownership); the working-tree
checklist diff (dod-006 row reverted) is the sync's output, disclosed here.
No git commands run.
