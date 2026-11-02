# VAC-7 — bench dead-surface audit (court-auditor, 2026-11-01)

Read-only audit; only this file written. All commands run with exit codes recorded.

## Commands (my exits)
- `CARGO_TARGET_DIR=/tmp/v7-target cargo bench -p bench --no-run` -> exit 0
  (all 15 `[[bench]]` targets + bin `bench` + 18 `src/bin/*.rs` binaries compile)
- `cargo bench -p bench --list` -> exit 0 (not a cargo flag; enumerated via
  `--no-run` Executable lines instead — noted, not a failure)
- receipt cite sweep: `grep -rl <name> docs/jira/v26.11.1 v26.10.31` (exit 0 per name)
- `cargo bench -p bench --bench challenger_git` -> exit 0, runs, prints divan table
- `diff src/benches/v26_10_31.rs benches/v26_10_31.rs` -> exit 1 (files differ)

## Inventory

| surface | declared | runs (my evidence) | cited-by-receipts | verdict |
|---|---|---|---|---|
| benches/bn3.rs | [[bench]] | compiled | 54 hits (F2 receipts) | valuable |
| benches/chains-bn3.rs | [[bench]] | compiled | 3 (bench-chains-bn3.md) | valuable |
| benches/chains-ingress.rs | [[bench]] | compiled | 3 | valuable |
| benches/chains-parquet.rs | [[bench]] | compiled | 4 | valuable |
| benches/chains-seal.rs | [[bench]] | compiled | 3 | valuable |
| benches/chains-sovereign.rs | [[bench]] | compiled | 5 | valuable |
| benches/challenger_arrow.rs | [[bench]] | compiled | 1 (challenger-sales-report.md) | valuable |
| benches/challenger_crypto.rs | [[bench]] | compiled; NO receipt run | 0 | DEAD WEIGHT (never run) |
| benches/challenger_git.rs | [[bench]] | compiled; I ran it: exit 0 (works; ladder 307 ns vs bsearch 25 ns — 12x SLOWER, unreported) | 0 | DEAD WEIGHT (never run) |
| benches/challenger_parquet.rs | [[bench]] | compiled | 1 | valuable |
| benches/ingress.rs | [[bench]] | compiled | 30 | valuable |
| benches/kernels.rs | [[bench]] | compiled | 63 (F2-10 scaffold) | valuable |
| benches/parquet.rs | [[bench]] | compiled | 142 | valuable |
| benches/seal.rs | [[bench]] | compiled | 60 | valuable |
| benches/v26_10_31.rs | [[bench]] | compiled | 21 | historical (prior campaign) |
| src/main.rs (bin `bench`) | default-run | compiled | 0 receipt invokes it | DEAD WEIGHT (just bench runs ctq_report instead) |
| src/bin/ctq_report.rs | bin | compiled; `just bench` runs it | 48 | valuable |
| src/bin/gage_rr.rs | bin | compiled | 26 | valuable (prior campaign) |
| src/bin/{arrow_accessors,neon_ab,phase_change,parquet_bench,digest_bench} | bin | compiled | 1-3 each | historical |
| src/bin/{blake3_profile,blake3_throughput,cpk_study,parquet_throughput,const_time_bench} | bin | compiled | 2-17 | historical |
| src/bin/{bitslice_bench,byteslice_bench,dict_bench,packed_filter_bench,packing_bench,page_bench} | bin | compiled | 0 in v26.10.31/26.11.1; dict_bench+packed_filter_bench 1 hit each in v26.10.29 AVEC-109 | historical |
| src/benches/v26_10_31.rs | NOT declared | never compiled | 0 | ORPHAN duplicate (differs from benches/ copy) — deletion candidate |
| tests/court_verdict.rs | [[test]] of cgap-court | n/a | yes | fine, not bench surface |

## Removal candidates (NEXT wave — recorded, not deleted)
1. `crates/bench/src/benches/v26_10_31.rs` — orphan stale duplicate, zero refs
2. `benches/challenger_git.rs` + `benches/challenger_crypto.rs` — declared+compile
   in every `cargo bench` invocation, no receipt cites either, no CTQ mapping in
   docs/DFLSS_CHARTER.md. challenger_git provably runs (exit 0) and its result
   (ladder 12x slower than binary search) is unreceipted.
3. `src/main.rs` bin `bench` — default-run, nothing invokes it; `just bench`
   runs `ctq_report` directly.
4. 18 `src/bin/*.rs` binaries recompile on every `just bench` release build
   despite being prior-campaign (v26.10.29/31) artifacts.

## Standing
ALIVE (audit receipt; all commands re-runnable as cited).
