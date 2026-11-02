# Bench-Ingress Receipt (wave F2, item 11 — lane F2-11)

| Field | Value |
|---|---|
| Lane | F2-11 bench-ingress |
| Date | 2026-11-01 |
| Subject | `crates/bench/benches/ingress.rs` (NEW, this lane) |
| Subject sha256 | `e6ff8f58e2dd183f9c954500d52b264cfe7747d831897b1e032df33db991e4e3` |
| Consumed, not written | `crates/bench/Cargo.toml` — [[bench]] ingress + cgap-core/ |
|  | cgap-gate deps landed by item 10 this session |
| Standing | ALIVE (command + exit 0 witnessed this session) |

## Commands (this session, with exits)

1. `cargo build --release --offline` (in `/tmp/f2_11_scratch`, bench file
   copied as `src/main.rs`) — exit 0 (compile check; 1 fix: `region_mut`
   needed `first_chunk_mut` — `&mut [u8]` has no `TryFrom<&[u8; 128]>`).
2. `./target/release/f2_11_scratch --bench --sample-count 3` — exit 0; all
   five bench instances ran; every timed iteration asserted the admission
   checksum against the preflight fold.
3. Mutation M-F2-11-1 (anti-vacuity, scratch copy only, pristine restored
   after): corrupt the A04 manifest (expect `BadMagic` on a frame left
   valid) — exit 101, `panicked ... A04 corpus frame 0: verdict drifted from
   manifest (left: 0, right: 1)`. The preflight court FLIPS; the checksum
   court is not vacuous.
4. `CARGO_TARGET_DIR=/tmp/lane-f2-11-target cargo bench -p bench --bench
   ingress` from `/Users/sac/cgap` — exit 0 (first full run, pre-wrap bytes).
5. Same command re-run on the FINAL bytes (line-wrap-only edits) — exit 0;
   the table below is THAT run (verbatim stdout).
6. `python3 shutil.rmtree('/tmp/lane-f2-11-target')` — lane build root
   deleted (twice: post-run-4 and post-final-run).

Host: Apple M3 Max, 16 cores, 48 GB, macOS 26.2 (Mac15,9). Divan-reported
timer precision: 41 ns. Load disclosure: the wave ran concurrently with
other F2 lanes on this host; the slowest/mean columns absorb it.

## Full Divan table (final run, verbatim stdout)

```text
Timer precision: 41 ns
ingress                    fastest       | slowest       | median        | mean          | samples | iters
+-- a14_penstock_admission                |               |               |               |         |
|  +-- 1048576              6.072 ms      | 8.066 ms      | 6.159 ms      | 6.598 ms      | 100     | 100
|  |                        172.6 Mitem/s | 129.9 Mitem/s | 170.2 Mitem/s | 158.9 Mitem/s |         |
|  +-- 10485760             59.74 ms      | 77.1 ms       | 67.17 ms      | 67.32 ms      | 100     | 100
| |                         175.5 Mitem/s | 135.9 Mitem/s | 156 Mitem/s   | 155.7 Mitem/s |         |
+-- cgap_admit_frame                      |               |               |               |         |
   +-- 1024                 2.04 us       | 6.957 us      | 2.124 us      | 2.177 us      | 100     | 100
   |                        501.8 Mitem/s | 147.1 Mitem/s | 481.9 Mitem/s | 470.1 Mitem/s |         |
   +-- 65536                238.8 us      | 373.9 us      | 240.2 us      | 247.6 us      | 100     | 100
   |                        274.4 Mitem/s | 175.2 Mitem/s | 272.7 Mitem/s | 264.6 Mitem/s |         |
   +-- 1048576              5.424 ms      | 5.775 ms      | 5.466 ms      | 5.481 ms      | 100     | 100
                            193.2 Mitem/s | 181.5 Mitem/s | 191.8 Mitem/s | 191.3 Mitem/s |         |
```

## MEASURED vs TARGET (verbatim; a GAP is named, never rounded)

| Bench | Const | MEASURED ns/frame (fastest = 1000/Mitem-per-s) | vs TARGET < 4.0 ns/frame |
|---|---|---|---|
| a14_penstock_admission | 1048576 | 5.79 ns/frame (172.6 Mitem/s; 6.072 ms) | **GAP** (1.45x) |
| a14_penstock_admission | 10485760 | 5.70 ns/frame (175.5 Mitem/s; 59.74 ms) | **GAP** (1.42x) |
| cgap_admit_frame (A04) | 1024 | 1.99 ns/frame (501.8 Mitem/s) | under TARGET (info) |
| cgap_admit_frame (A04) | 65536 | 3.64 ns/frame (274.4 Mitem/s) | GAP (info row) |
| cgap_admit_frame (A04) | 1048576 | 5.18 ns/frame (193.2 Mitem/s; 5.424 ms) | GAP (info) |

- TARGET row of record is the dam bench: MEASURED 5.79/5.70 ns/frame
  fastest = **GAP** against the < 4.0 ns/frame TARGET (A14 lane doc §8.4,
  the dispatch's ~4.0 ns figure). Median 1M const: 6.159 ms / 2^20 =
  5.87 ns/frame — also GAP. No rounding, no promotion.
- This is the FFI/admission-layer twin of A14's in-suite court
  (penstock.rs waves 7/8: 7.55/7.60 ns/frame full chain, min-of-11 paired
  rounds, release). The admission-layer number (5.70-5.79) sits between the
  4 ns TARGET and the dam's own full-chain number; both layers name GAP.
- 1.99 ns/frame at const 1024 is a cache-resident artifact (1,024 x 128 B =
  128 KB corpus), recorded as info, never promoted to a line-rate claim.

## Courts (Chicago: real kernel, real memory, final state asserted)

- Preflight manifest court (untimed, per bench instance): every frame's real
  verdict equals its by-construction manifest row; garbage must NOT be
  admitted; every admitted view is zero-copy at the frame's payload base and
  matches the header-declared geometry.
- Per-iteration checksum court (timed, every iteration of every sample): the
  pass fold of real A19 status codes (dam) / A04 err_codes must equal the
  preflight-recorded fold (dam pool cycled exactly `laps` times; A04 fold
  from the manifest). One misclassification by the real kernel flips the
  bench mid-run. No Process.sleep anywhere; no mocks; the corpus is a pure
  function of fixed seeds + frame index.
- Anti-vacuity mutation M-F2-11-1 FLIPPED the preflight court (command 3,
  exit 101) — the court compares real kernel verdicts, not itself. A court
  that cannot flip is FAIL; this one flips.

## Divergences (named, not silent)

1. Dam corpus pool: 65,536 seeded frames packed (~15 MB), cycled by
   `i & 65_535` across the N admissions of each const. A literal 10M-frame
   materialization is ~2.2 GB and hostile to fan-out lanes sharing this
   host. Every processed frame is still real packed memory through the real
   gate; the checksum math requires N % 65536 == 0 (asserted; both consts
   are multiples: 16x and 160x).
2. Dam generator garbage length capped at 384 B (the dam court draws up to
   4096 B) for the same memory bound; the class mix otherwise mirrors A14
   §8.1: 50% garbage / 25% single-law mutation / 25% valid, 9-class mutation
   wheel including the row-6 flags-reserved ADMITTED divergence pinned by
   penstock.rs.
3. The dispatch's bounded poll for `[[bench]] name = "ingress"` was moot:
   the condition was already true at first check (item 10 landed the target
   plus cgap-core/cgap-gate deps before the poll began). No polling loop ran.
4. Divan 0.1.21 invoked directly (not via `cargo bench`) silently defaults
   to Action::Test unless `--bench` is passed — it prints the bench tree
   with NO values and exits 0. Scratch-only learning; both receipt runs went
   through `cargo bench`, which passes `--bench` (values witnessed above).
5. The first scratch smoke witnessed the preflight court flipping on MY
   defect (admitted-view pointer compared to frame start instead of the
   frame's payload base) — the kernel was right, the assert was repaired,
   re-smoked green before any real run.

## Notes

- A04 corpus frames are header-only at 128 B stride: `project`+`validate`
  read only the 128 B header by design; `payload_byte_len` is generated
  0..255 and written through the real `CgapFrameHeader::encode_into`.
- SplitMix64 constants verbatim: increment 0x9E3779B97F4A7C15, multipliers
  0xBF58476D1CE4E5B9 (canonical, per dispatch) and 0x94D049BB133111EB.
- Seeds: A04 0x0A04_0000_C6A4_0001, A14 0x0A14_0000_5EED_0001 (fixed
  literals; corpus is a deterministic function of seed + frame index).
- `AlignedFrames` carries `unsafe impl Send + Sync` (owned allocation,
  read-only shared access after construction; divan bench closures are
  `Fn + Sync`).
