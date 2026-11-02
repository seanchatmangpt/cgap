# w5-encode-repareto

Lane A4 (Analyze): re-PARETO of the chained persistence pipeline after the
FNV refutation — first-hand stage-share measurement @1M rows plus alloc-site
witnessing. No repo source edits; only this receipt. Repo
`/Users/sac/cgap`, branch main, subject tree post-`9a730320` (concurrent
wave lanes landed edits on `crates/cgap-parquet/src/scan.rs` mid-lane;
see divergences 3-4).

## Commands (this session, exits)

| command | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/a4-target cargo bench -p bench --bench chains-parquet` (full sweep, 100 iters) | 0 |
| `CARGO_TARGET_DIR=/tmp/a4-target cargo bench -p bench --bench chains-parquet -- chain_ram chain_file decode_arm digest_arm` | 0 |
| `CARGO_TARGET_DIR=/tmp/a4-target cargo bench -p bench --bench chains-parquet --no-run` (recompile after concurrent-lane scan.rs break, green) | 0 |

## Stage-share table (divan, mean of 100, /tmp/a4-target)

| leg | 65,536 | 1,048,576 | share of chain @1M |
|---|---|---|---|
| chain_ram (full chain) | 1.670 ms | 27.02 ms | 100% |
| chain_file (real file) | 1.678 ms | 27.50 ms | — |
| seal_arm (write+seal+flush) | 1.013 ms | 16.36 ms | ~61% |
| verify_arm (verify + leg decode) | 425.4 µs | 6.807 ms | ~25% |
| encode_arm (standalone rle_encode, 1 lane) | 160 µs-class | 2.559 ms | per lane |
| decode_arm (lane-0 page decode) | 54.8 µs | 888 µs | ~3.3% |
| digest_arm (fnv1a64 + block_digest, 2 MB) | 437.6 µs | 6.489 ms | ~24% |
| flush_arm (memcpy + commit) | 1.687 µs | 34.3 µs | ~0.1% |

Arithmetic: seal_arm(16.36) = writer encode 2 lanes ~9.8-10.2 ms + seal
digests 6.49 ms + flush 0.034 ms. chain_ram(27.0) ≈ seal 16.4 + verify 6.8
+ decode 0.89 + ~3 ms chain/noise residual (concurrent-wave contention;
chain_ram max 168.9 ms observed).

## Alloc sites (witnessed, code + divan alloc column)

- `crates/cgap-parquet/src/footer.rs:376` — `spans: summary.spans.to_vec()`
  in `FooterSealer::new` (not SealedBlock itself; the sealer feeds it).
  8 B for a 2-span block.
- `crates/cgap-parquet/src/footer.rs:447-475` — `build_meta_block`'s
  growing `Vec` (the other seal-side alloc; 16-24 B for 2 spans).
- `crates/cgap-parquet/src/footer.rs:704` — `Vec::with_capacity(n.min(1024))`
  in `parse_meta`, reached from `verify_seal` — this is the VERIFY-side
  alloc, not part of seal's two.

Measured alloc column: seal_arm = 2 allocs / 12 B allocated / 36 B max,
36 B dealloc per tx @1M (matches sites {376, 447}); verify_arm = 1 alloc /
4 B (matches site 704); flush_arm 0 allocs steady-state; chain arms 0
retained allocs in the timed loop. Dispatch's claim that sites 376+704
are the seal-path 2 allocs/36 B is corrected: 704 is verify-side.

## Top-3 remaining latency drivers @1M (ranked)

1. **Writer encode path** `encode_hybrid_into`
   (`crates/cgap-parquet/src/mmap_writer.rs:421-482`): ~9.8-10.2 ms
   (seal_arm 16.36 − digest_arm 6.49 − flush 0.034), ~36-38% of chain.
   ~1.9× the standalone `rle_encode` (2.559 ms/lane): byte-at-a-time
   `push_byte`/`push_varint` (mmap_writer.rs:335/359) plus an O(n·run)
   re-scan inner loop on the literal path.
   **Proposed lever (not implemented)**: word-wise literal gather —
   literal groups emit 8 contiguous low-bytes; build them word-wise
   (SWAR u64 gather + one 8-byte copy per group) and mirror
   `rle_encode_fast` run detection; keep the wire byte-identical (the
   differential court vs `rle_encode` pins it).
   **Falsifier**: a writer-encode-only bench leg @1M must drop below
   2× encode_arm (< ~5.2 ms); if it stays ≥ 2× standalone, the lever is
   refuted. Anti-vacuity mutation: flip one gathered literal byte in the
   sandbox copy — the encode wire-length/checksum asserts must fire.
2. **Verify-side digest recompute** (footer.rs verify_seal steps 4-5,
   footer.rs:631-647): ~5.9 ms (verify_arm 6.807 − decode leg 0.89),
   ~22% of chain. Two full passes over the same ~2 MB content: `fnv1a64`
   over `[0, B+M+80)` then `block_digest` over the same range.
   **Proposed lever**: fuse into one sweep (seal side already did this —
   `SealDigests::over_body`, p2 lever); verify side still pays 2 passes.
   **Falsifier**: a verify-only leg @1M drops ≥1.7× (5.9 → ≤3.5 ms); the
   corruption-localization second pass stays on the failure path, and
   `hostile_sealed_block_corpus_fails_closed_with_typed_verdicts` stays
   green with byte-exact wire unchanged.
3. **Read-back decode** `decode_page_into` (`decode_arm` 888 µs, ~3.3%):
   already header-bounded and caller-buffer direct. **Proposed lever**:
   word-wise literal expansion in the decode loop. **Falsifier**:
   decode_arm @1M drops below 500 µs with decode differential courts
   green; if it stalls above 700 µs the lever is refuted and decode is
   declared at its serial floor.

Below the Pareto cut: flush (34 µs), meta/parse allocations (36 B/tx —
real but nanosecond-class), chain residual ~3 ms (contention, not a
code driver).

## Courts

No new court needed for a measurement lane; the bench's four named
anti-vacuity mutations (chains-parquet.rs:21-37) remain the standing
flip battery for these rows (witnessed FLIPPED-4/4 in
`docs/jira/v26.11.1/waves/f2-15-bench-flips.md`).

## Divergences

1. chain_ram @1M measured 27.02 ms this lane vs p2-pareto's 23.21 ms —
   concurrent wave load (max 168.9 ms outliers). Stage ratios are stable;
   absolute chain means are load-sensitive. Both receipts stand.
2. Dispatch said footer.rs:376 alloc lives "in SealedBlock" — it is in
   `FooterSealer::new` (the struct SealedBlock carries is bytes+digest
   only, footer.rs:482-485). Dispatch said :704 is one of the seal-path
   2 allocs — it is verify-side (parse_meta). Corrected above.
3. Mid-lane, a concurrent lane's in-flight `scan.rs` edit broke the
   `cgap-parquet` lib compile (missing `bounds` field at scan.rs:553,
   E0063, exit 1) — observed, retried, and the owning lane had fixed it
   by the retry (~minutes); no SLA unblock fix applied by this lane.
4. This lane wrote no repo source and no bench-file lines: the existing
   bench already carries every leg needed (chain_ram/chain_file/seal/
   verify/encode/decode/digest/flush) — no harness gap.

{"lane":"a4-encode-repareto","files_written":["docs/jira/v26.11.2/waves/w5-encode-repareto.md"],"commands":[["cargo bench -p bench --bench chains-parquet (full)",0],["cargo bench -p bench --bench chains-parquet -- chain_ram chain_file decode_arm digest_arm",0],["cargo bench --no-run (post-scan.rs-fix recompile)",0]],"courts_run":["standing chains-parquet mutation battery (f2-15 FLIPPED-4/4, not rerun this lane)","no new court: measurement lane, zero source diff"],"standing":"PARTIAL_ALIVE (stage-share + alloc sites measured first-hand this session; three levers proposed with falsifiers, none implemented)","divergences":["chain_ram 27.02 ms vs p2's 23.21 ms — concurrent wave load, max outliers 168.9 ms","alloc-site attribution corrected: footer.rs:376 is FooterSealer::new (not SealedBlock); footer.rs:704 is verify-side parse_meta, not one of seal's 2 allocs/36 B","concurrent-lane scan.rs edit transiently broke the lib compile (E0063); fixed by owner before any unblock was needed","no bench-file edit: existing legs covered the whole share table"]}
