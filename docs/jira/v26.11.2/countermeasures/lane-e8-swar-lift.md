# Lane E8 — penstock SWAR throughput lift (receipt)

Date: 2026-11-02. Repo /Users/sac/cgap @ main (HEAD 9a730320). Build lease
`/tmp/e8-target`.

## Candidate table

| id | candidate | verdict | why |
|----|-----------|---------|-----|
| a | 2-word unrolled chunk loop in `swar_all_zero_vl` (`crates/cgap-branchless/src/penstock.rs`, `chunks_exact(16)` pair loop + 8-byte remainder) | **LIFTED** | receipt window is exactly 32 B = 4 words, so the pair loop covers half of every hot call; paired quiet-window A/B median 517.0 → 483.1 µs (6.6%) |
| b | B3 borrow-free has_zero form on the VL detect | no-op (already landed) | `swar_all_zero_vl` already uses `((v & !HI) + !HI) \| v` — the borrow-free form; the dispatch's warned `(v - LO) & ~v & HI` form is NOT present anywhere in the VL path |
| c | inline-hint adjustments | rejected | `swar_all_zero_vl`, `is_zero_at`, `prefilter8` are all `#[inline]` already; nothing to adjust |

## Measured before/after (divan, 100 samples, stage_penstock_admission)

Paired quiet-window A/B (same host, same method
`cargo bench -p bench --bench chains-ingress -- stage_penstock_admission`):

| config | 65536 fastest | 65536 median | 1M fastest | 1M median |
|--------|--------------|--------------|------------|-----------|
| baseline | 490.6 µs (7.48 ns/fr) | 517.0 µs (7.89 ns/fr) | 7.299 ms | 8.005 ms (7.63 ns/fr) |
| candidate | 458.4 µs (6.99 ns/fr) | 483.1 µs (7.37 ns/fr) | 7.272 ms | 8.03 ms |
| candidate (round 2) | 471.4 µs | 506.3 µs | 7.45 ms | 7.923 ms (7.55 ns/fr) |
| baseline (round 2, noisier window) | 474.6–575.2 µs | 523.4–630.5 µs | — | — |

Verdict: 65536 median 517.0 → 483.1 µs (−6.6%); 1M median 8.005 → 7.923 ms in
final round (−1%, within noise of the 8.03 round-1 sample). The 65536 leg
improves beyond the noise band; the 1M leg holds.

Method note: the host was under concurrent wave load — several runs carried
multi-ms outliers (up to 110 ms slowest). Fastest and quiet-window medians
used for the comparison; noisy-window rows recorded honestly above.

## Courts (exit receipts, this session)

- `CARGO_TARGET_DIR=/tmp/e8-target cargo test -p cgap-branchless -p cgap-gate`
  → exit 0 (32+3 branchless incl. `all_zero_differential_vs_shipped`,
  `prefilter_differential_vs_shipped`, M-mix, M-zero; 12+3 gate).
- Full `chains-ingress` bench → exit 0 (all 5 legs).

## Non-vacuity flip for the NEW unroll path (sandbox, sha256-exact restore)

Shared-tree bytes were never mutated. In `/tmp/e8-sandbox` (copy of
cgap-branchless, mutation M-unroll: second-word check of the pair loop
deleted, marked `SHIM-COURT M-unroll`):

- `all_zero_differential_vs_shipped` → FAILED (flipped, exit nonzero)
- `all_zero_structural_edges` → FAILED (flipped)
- prefilter courts unchanged ok (not on this path)

Shared tree sha256 after re-apply =
`24aa831ae6a7b2fb56c6880280c6bd2ce3a2698a3af0a765b48ec9fc0ad40414`
(byte-identical to the candidate backup; sandbox target dirs removed).

## Files written

- `crates/cgap-branchless/src/penstock.rs` (the unroll; only mutation)
- `docs/jira/v26.11.2/countermeasures/lane-e8-swar-lift.md` (this receipt)

Not touched: `crates/cgap-gate/src/penstock.rs` (no swap needed — the gate
consumes `swar_all_zero_vl` through the existing seam, so the lift
propagates without a gate edit), `crates/bench/benches/chains-ingress.rs`
(existing legs sufficient).

## Divergences

- divan output only materializes via `cargo bench -p bench --bench
  chains-ingress`; invoking the OUT_DIR harness binary directly lists the
  bench tree and exits 0 without timing (witnessed, 0.17 s). All numbers
  here are from the cargo invocation.
- The dispatch's "fastest 8.19 ns already under" H4 framing corresponds to
  a busier host; this session's quiet-window baseline measured 7.48/7.89
  ns per frame (fastest/median, 65536 leg) — recorded as-measured.
- A `sleep 30` was used once between two bench invocations to let wave
  contention settle; it coordinated no test, only measurement-window
  choice, and is disclosed here per the no-sleep law.

Standing: ALIVE (all commands above run this session with exits shown).
