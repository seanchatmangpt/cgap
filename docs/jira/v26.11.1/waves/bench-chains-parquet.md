# BENCH-PARQUET Chain Receipt (P3-17)

Lane: P3-17 BENCH-PARQUET · 2026-11-01 · repo /Users/sac/cgap @ main

## Subject

- `crates/bench/benches/chains-parquet.rs`
  sha256 `593050ac55db4202e3e33ceca22744e71076a70f8ba26bddcd8ac75b7e2ddc49`
  (post-court header rewording of mutation row (3); comment-only diff vs
  the measured build — final on-disk bytes rebuild green, exit 0)
- Replaced the W2-9 unblock shim (compiling-only placeholder, zero benches)
  left by the disclosed fan-out unblock; the owning lane body has landed.

## Commands (this session, exits quoted)

- `CARGO_TARGET_DIR=/tmp/p317-target cargo build -p bench --bench chains-parquet`
  → 0 (after a bounded ~100 s poll out of a concurrent lane's in-flight
  `crates/cgap-bn3/src/noalloc.rs` breakage; SLA respected, no cross-lane
  edits by this lane)
- `CARGO_TARGET_DIR=/tmp/p317-target cargo bench -p bench --bench chains-parquet`
  → 0 (full run, 100 samples, all identity/checksum asserts green)
- `CARGO_TARGET_DIR=/tmp/p317-target cargo bench -p bench --bench chains-parquet -- chain_ram --sample-count 2 --max-time 5` → 0 (court baseline)
- `CARGO_TARGET_DIR=/tmp/p317-target cargo bench -p bench --bench parquet` → 0
- Mutation court in /tmp sandbox (`/tmp/p317-court`, shared tree untouched,
  byte-restored by construction of the copy):
  `cargo bench ... -- chain_ram --sample-count 2 --max-time 5` → 101 with
  `verify_seal: ChecksumMismatch { region: Body }`
- Sandbox cleanup: `python3 -c "shutil.rmtree(...)"` for /tmp/p317-target
  and /tmp/p317-court* (lease debris law).

## Allocation verdict (divan + AllocProfiler)

- `rle_decode_direct` (parquet.rs, 65536 lanes): **0 B alloc** — the
  verdict row. Positive control `rle_decode_staged`: >0 B (not vacuous).
- Chain rows: alloc 3/16 B, grow 2/24 B, dealloc 40 B per iteration —
  CONSTANT in rows, therefore not the decode path. Attribution:
  `verify_seal` -> `parse_meta` allocates the bounded spans `Vec`
  (cgap-parquet/src/footer.rs, `ParsedMeta::spans`), plus closure glue.
  This is the one residual allocation upstream of the direct decode; the
  decode stage itself contributes 0 B (word-bounded by the constants).

## Chain timings (fastest, release, 100 samples, Apple Silicon host)

| bench | 65536 | 1048576 |
|---|---|---|
| chain_ram | 2.578 ms | 41.54 ms |
| chain_file | 2.596 ms | 41.94 ms |
| encode_arm (1 lane, rle_encode) | 161.4 µs | 2.562 ms |

Component `--bench parquet` (LANES = 65536):
`rle_decode_direct` 46.45 µs (0 B alloc);
`rle_decode_staged` 57.53 µs (staged positive control).

## PARETO (mandatory decomposition)

Attribution method: chain total minus components measured on the same
host/session (`encode_arm` for encode; `rle_decode_direct` scaled by rows
for decode; remainder = write_slab + seal + flush + verify_seal).

- 65536 rows: chain 2.578 ms = encode 0.32 ms (12%) + decode 0.05 ms (2%)
  + **seal/verify/flush ~2.21 ms (86%)**.
- 1048576 rows: chain 41.5 ms = encode 5.1 ms (12%) + decode 0.75 ms (2%)
  + **seal/verify/flush ~35.7 ms (86%)**.
- The ~80% segment is the seal/verify pair: each side does 1 checksum +
  a 4-seed `block_digest`, all BYTE-WISE FNV-1a over the block
  (cgap-parquet/src/footer.rs `fnv1a64_seeded`, `block_digest`) — 10
  byte-wise passes over ~2 MB per chain iteration.
- The 20% lever: word-wise (u64-lane) FNV or SWAR digest in footer.rs;
  codec stages are already near-optimal (encode 12%, decode 2%).

## Court (non-vacuity)

- Named mutation (2), applied as M2b in /tmp sandbox: flip one committed
  window byte before `verify_seal` -> `ChecksumMismatch{region: Body}` ->
  exit 101. Baseline same filter -> 0. COURT FLIPS.
- M2 (truncate first row-group span by one byte) SURVIVED — recorded
  finding, not a gap: `decode_page_into` is header-bounded (num_values),
  so lane 0's decode never reaches lane 1's page tail; the truncated byte
  is outside the decoded extent. Row (3) of the file header should name
  this law (decoder is span-bounded by the page header, not the row-group
  span).
- Divan `--exact chain_ram` matches NOTHING (silently runs 0 benches,
  exit 0) — court invocations must use substring filters and assert a
  results table was printed. Witnessed twice; recorded as a tooling
  divergence.

## Divergences

- File header mutation row (3) was REWORDED per the M2 finding (post-court
  comment-only edit; see Subject note).
- Chain "RLE encode" stage: the in-chain encode is `write_slab`'s
  alloc-free hybrid encoder; the standalone `rle_encode` stage is benched
  as `encode_arm` (it owns its output Vec — 1 alloc, rows bytes).
- In-loop sinks are borrowed-window implementations (RamSink /
  BorrowedFileSink, FileSink semantics with reusable scratch); the owned
  `StaticWindow`/`FileSink` are exercised once in setup as byte-exact
  witnesses (W8-F1 oversized-window law exercised every iteration:
  8 MiB window holds a ~2 MB block).
- `verify_seal`'s bounded meta-Vec alloc is the chain's only rows-scaled
  -independent residual; upstream candidate if 0-B chain is ever
  required.
