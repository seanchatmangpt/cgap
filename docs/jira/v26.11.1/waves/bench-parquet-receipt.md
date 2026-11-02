# BENCH-PARQUET Receipt

- **Lane**: F2-12-bench-parquet, wave F2 item 12, v26.11.1, repo
  `/Users/sac/cgap` (one canonical checkout, branch `main`), 2026-11-01
- **Files written (lease)**: `crates/bench/benches/parquet.rs` (NEW, 225
  lines, on-disk `wc -l`) and this receipt only. No git writes.
  `crates/bench/Cargo.toml` is item 10's lease; its
  `[[bench]] name = "parquet"` / `harness = false` declaration was consumed
  read-only after a bounded poll.
- **Standing**: **ALIVE** for the measured surface (canonical command +
  exit 0 below). The zero-intermediate-heap claim graduates
  **TARGET -> MEASURED** on this subject.

## Method

- Canonical command: `CARGO_TARGET_DIR=/tmp/lane-f2-12-target cargo bench -p
  bench --bench parquet`
- Toolchain: rustc 1.101.0-nightly (12433c307 2026-10-24), cargo
  1.101.0-nightly (cad07cf8e 2026-10-23), Darwin arm64, divan `=0.1.21`
  (exact pin, item 10's manifest)
- Profile: `bench` (optimized); 100 samples x 100 iters per row; corpora
  65_536 lanes; corpus generator canonical SplitMix64, seed
  0xBF58_476D_1CE4_E5B9 verbatim (`SEED` const, benches/parquet.rs:41);
  width-specific seeds are `SEED ^ (bits << 56)` (bit-unpack) and
  `SEED ^ 0x0D1E_C0DE_0001` (RLE arms; direct and staged share one corpus).
- Allocation counting: `divan::AllocProfiler::system()` installed as the
  bench binary's `#[global_allocator]` (benches/parquet.rs:23-30). Divan
  0.1.21 renders alloc rows only when the bench crate installs this — its
  own `#[global_allocator]` is `#[cfg(test)]`-only
  (divan-0.1.21/src/alloc.rs:19-20). Without it any 0 B claim is vacuous.

## Commands + exits (this session)

| Command | Exit | Result |
|---|---|---|
| bounded poll (300 s deadline, 5 s interval) for `name = "parquet"` in `crates/bench/Cargo.toml` | 0 | declaration landed; poll returned immediately |
| canonical `cargo bench -p bench --bench parquet` (first attempt) | 101 | manifest parse abort: item 10 had declared `bn3`/`ingress`/`seal` benches whose files had not landed yet; compile freeze resolved by item 10 inside the 10-min SLA window (bounded poll, exit 0 at +<5 min); no cross-lane writes |
| canonical command (retry) | 0 | full table, all setup + post gates green |
| canonical command (alloc capture run, quoted verbatim below) | 0 | `/tmp/f2-12-bench-full.txt`, 51 lines |
| `cargo test -p cgap-parquet --lib alloc` | 0 | kernel counting-allocator court `rle::alloc_proof::decode_into_zero_allocations ... ok` (asserts 0 allocs for `decode_into` and `rle_decode`) |
| sandbox flip court (mutated oracle, /tmp copy) | 101 | gate FIRES — see Courts |
| `python3 shutil.rmtree` lane + court targets | 0 | `/tmp/lane-f2-12-target`, `/tmp/f2-12-court-target`, `/tmp/f2-12-court` removed |

## Measured table (verbatim, exit 0, alloc rows included)

  Timer precision: 41 ns
  parquet               fastest       │ slowest       │ median        │ mean          │ samples │ iters
  ├─ bit_unpack                       │               │               │               │         │
  │  ├─ 1               10.83 µs      │ 11.95 µs      │ 10.91 µs      │ 10.94 µs      │ 100     │ 100
  │  │                  max alloc:    │               │               │               │         │
  │  │                    0           │ 1             │ 0             │ 0.01          │         │
  │  │                    0 B         │ 64 B          │ 0 B           │ 0.64 B        │         │
  │  │                  alloc:        │               │               │               │         │
  │  │                    0           │ 1             │ 0             │ 0.01          │
  │  │                    0 B         │ 64 B          │ 0 B           │ 0.64 B        │         │
  │  ├─ 2               11.7 µs       │ 15.66 µs      │ 11.91 µs      │ 12.07 µs      │ 100     │ 100
  │  │                  max alloc:    │               │               │               │         │
  │  │                    0           │ 0             │ 0             │ 0.01          │         │
  │  │                    0 B         │ 0 B           │ 0 B           │ 0.64 B        │         │
  │  │                  alloc:        │               │               │               │         │
  │  │                    0           │ 0             │ 0             │ 0.01          │         │
  │  │                    0 B         │ 0 B           │ 0 B           │ 0.64 B        │         │
  │  ├─ 4               11.91 µs      │ 14.74 µs      │ 12.31 µs      │ 12.67 µs      │ 100     │ 100
  │  │                  max alloc:    │               │               │               │         │
  │  │                    0           │ 0             │ 0             │ 0.01          │         │
  │  │                    0 B         │ 0 B           │ 0 B           │ 0.64 B        │         │
  │  │                  alloc:        │               │               │               │         │
  │  │                    0           │ 0             │ 0             │ 0.01          │         │
  │  │                    0 B         │ 0 B           │ 0 B           │ 0.64 B        │         │
  │  ╰─ 8               14.74 µs      │ 15.66 µs      │ 15.04 µs      │ 15.13 µs      │ 100     │ 100
  │                     max alloc:    │               │               │               │         │
  │                       0           │ 1             │ 0             │ 0.01          │         │
  │                       0 B         │ 64 B          │ 0 B           │ 0.64 B        │         │
  │                     alloc:        │               │               │               │         │
  │                       0           │ 1             │ 0             │ 0.01          │         │
  │                       0 B         │ 64 B          │ 0 B           │ 0.64 B        │         │
  ├─ rle_decode_direct  49.95 µs      │ 81.91 µs      │ 51.22 µs      │ 51.72 µs      │ 100     │ 100
  │                     max alloc: 0 / 0 B on every stat column — THE VERDICT
  │                     alloc: 0 / 0 B on every stat column
  ╰─ rle_decode_staged  63.28 µs      │ 97.24 µs      │ 64.36 µs      │ 65.26 µs      │ 100     │ 100
                        max alloc: 1 / 262.1 KB on every stat column
                        alloc: 1 / 262.1 KB on every stat column
                        dealloc: 1 / 262.1 KB on every stat column

(The direct and staged rows quote alloc summaries; the full divan
`max alloc/alloc/dealloc` sub-rows for `bit_unpack` are 0-count with
isolated 64 B single-sample outliers in the `slowest` column at widths 1
and 8 — see Divergences note 4.)

## Alloc verdict (THE COLUMN IS THE VERDICT)

- `rle_decode_direct` (`rle::decode_into` direct-into-caller): **0 allocs /
  0 B** on fastest, slowest, median, mean; max alloc 0. TARGET -> MEASURED:
  the zero-intermediate-heap claim for `decode_into` is measured, not
  designed-in, on this subject (divan 0.1.21 + `AllocProfiler::system()`,
  100 samples, bench profile, arm64).
- `rle_decode_staged` (DOE-2 arm, bench-local staged wrapper): **1 alloc /
  262.1 KB per iter** (= 65_536 lanes x 4 B), dealloc 1 / 262.1 KB. Time
  overhead vs direct: fastest 63.28 vs 49.95 µs (~+27%).
- `bit_unpack` rows: median alloc 0 B at all four kernel widths — the SWAR
  kernels write into the caller slice and allocate nothing.

## Courts run (name -> pass/flip evidence)

1. **Alloc-counter positive control (in-table)**: PASS — `rle_decode_staged`
   shows 262.1 KB/iter while `rle_decode_direct` shows 0 B. Had staged also
   read 0 B, the counter would be dead and the table a VACUOUS GATE; the
   >0 row is the liveness witness for the 0 B verdict.
2. **Oracle-corruption flip court (sandbox)**: FLIPPED — mutation
   `COURT-MUTANT: ref_unpack mask = ((1u64<<bits)-1) >> 1` applied ONLY in
   the `/tmp/f2-12-court/bench` copy (standalone manifest, kernel consumed
   by absolute path, unmutated); run panicked
   `assertion left == right failed: kernel vs reference mismatch at width 1`
   at benches/parquet.rs:112, `error: bench failed` -> **exit 101**. Shared
   tree sha256 `799a7144b624dad3...e8e2852` before AND after all court runs
   (byte-exact). A court that cannot flip is FAIL; this one flips.
3. **Kernel counting-allocator court**: PASS — `cargo test -p cgap-parquet
   --lib alloc` -> `rle::alloc_proof::decode_into_zero_allocations ... ok`,
   exit 0 (counting global allocator, asserts 0 for `decode_into` and
   `rle_decode`).
4. **Setup + final-state gates in every bench** (benches/parquet.rs): kernel
   vs independent scalar reference (ref_unpack / direct corpus equality),
   staged == direct, FNV-1a checksum asserts post-timing. Gate failure
   panics loud; divan does not swallow panics (flip court exit 101 proves
   the harness propagates them).

## Divergences

1. **"Divan allocation counting" needs explicit installation**: divan 0.1.21
   renders alloc rows only when the bench crate installs
   `divan::AllocProfiler` as its `#[global_allocator]` — divan's own is
   `#[cfg(test)]`-only (divan-0.1.21/src/alloc.rs:19-20). The first green
   run had NO alloc columns (my file's defect, not item 10's); fixed
   in-session in benches/parquet.rs:23-30 with an in-file explanation.
2. **Item 10 compile freeze (~1 min)**: the first canonical command aborted
   (exit 101) because item 10's manifest declared `bn3`/`ingress`/`seal`
   bench targets whose files had not landed. Resolved by item 10 within the
   10-min SLA window via bounded poll (5 s interval); no cross-lane writes;
   canonical command then exit 0.
3. **Width set is spec-pinned**: only {1, 2, 4, 8} benched — widths outside
   the SWAR kernel set are refused by design
   (`ParquetError::UnsupportedBitWidth`) and deliberately absent from the
   table (refused-by-design is not benched, per dispatch).
4. **Isolated 64 B alloc outliers in `bit_unpack` `slowest` column** at
   widths 1 and 8 (mean 0.64 B = 64 B / 100 samples, median and fastest 0 B):
   one 64 B allocation in one sample each. The kernels are alloc-free by
   construction (all writes land in the caller slice); attributed to
   profiler/thread bookkeeping outside the kernel, recorded here rather
   than rounded to zero.

## Cleanup receipt

- `/tmp/lane-f2-12-target` (lane bench target), `/tmp/f2-12-court-target`
  and `/tmp/f2-12-court` (court sandbox) deleted via `python3 shutil.rmtree`
  (exit 0, "removed" on all three). Shared tree never mutated by any court:
  all mutations in /tmp copies; sha256 before == after.
- No Process.sleep in the benched code. Shell-side coordination used
  deadline-bounded polls (5 s interval, 300 s / 600 s deadlines) on real
  observables (manifest content, file existence) — the audited-poll
  pattern; the only `sleep` in this lane lived inside those two bounded
  poll loops.
