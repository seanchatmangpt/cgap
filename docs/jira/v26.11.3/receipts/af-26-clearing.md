# AF-26 Receipt — Flash Crash Clearinghouse (macro-benchmark)
Standing: ALIVE

Lane: AF-26 IMPLEMENTATION. Date: 2026-11-03.
Bench: `crates/cgap-aoq/tests/macro_clearing_stress.rs` (new; scaled 1M default,
`#[ignore]` full 10M via `-- --ignored`).
Gates: scaled mode green x2 (release, below); full mode documented run below.

## Subject

End-to-end per-frame transfer function over a REAL unix-domain socketpair,
all landed surfaces, no mocks:

1. **Transport** — 128 B wire unit (64 B AOQ1 header + 64 B order body;
   oversize-toxic frames carry a 240 B body, 304 B wire unit) via
   `UnixStream::pair()`, blocking `read_exact`/`write_all` (backpressure,
   not loss).
2. **Decode** — AB-6 `codec::decode` of the 64 B envelope (magic, version,
   epoch, reserved tail, FNV-1a32 digest), typed `AoqError` refusals.
3. **Risk gate** — fixed `[u64; 10_000]` margin ledger (AB-19 slab
   pattern) indexed by `seq % 10_000`, wrapping arithmetic, seeded opening
   margin 1,000/instrument, zero alloc. Typed decisions:
   `MarginExceeded` (wrap detect) / `UnauthorizedCounterparty` (key_id
   != 7) / `OversizeBody` (> 64 B cap) / `HeaderRefused(AoqError)`.
4. **Receipt** — AB-7 `AoqChain::append` per admitted frame, cgap-arrow
   BLAKE3 as the real `ChainDigest` substrate (same seam as ab8_soak).

Stream: xorshift64* seed `0xAF26_2026`; every 10th frame toxic, cycling
{Margin, Unauthorized, Oversize}. The consumer derives each frame's
expected class and wire length from the index alone; the refusal census is
ASSERTED against that plan (typed refusals, never panics — reaching the
asserts IS that gate).

**AB-5 disclosure (typed)**: ML-KEM (AB-5) is in-flight with a known
round-trip defect; this bench binds no KEM — the clearinghouse frame path
is unencrypted AOQ1 header law + BLAKE3 receipt chain. Test-KEM fallback
not exercised; disclosed, not silently skipped.

## Method

- Release profile: `cargo test -p cgap-aoq --release --test macro_clearing_stress -- --nocapture`.
- Producer thread writes frames on the fly (`codec::encode` per frame);
  consumer thread runs the transfer function and samples per-frame wall
  with one `Instant` pair (~30-40 ns timer floor, disclosed — included in
  every percentile by construction).
- Alloc census: counting global allocator (ab8_soak pattern), delta across
  the whole consumer hot loop, after all setup preallocations. The
  allocator is wired `#[global_allocator]` (an earlier draft had it
  unwired — the 0-alloc assert would have been vacuous; fixed before any
  recorded run).
- Chain integrity: independent `verify_records` re-walk of every landed
  link post-soak.
- Census drift caught and fixed during development: margin-toxic frames
  initially shipped a non-overflowing notional (census assert failed:
  0 vs plan 33334) and first-touch zero slots admitted `u64::MAX`
  silently; fixed via MAX notional + seeded opening margin. The census
  gate demonstrably kills.

## Results (aarch64 darwin, release, 2026-11-03)

### Scaled gate (1M frames) — two consecutive green runs

| metric | run 1 | run 2 |
|---|---|---|
| wall | 2.065 s | 2.341 s |
| throughput | 484,234 frames/s | 427,196 frames/s |
| admitted / refused | 900,000 / 100,000 | same |
| p50 | 1,500 ns | 1,625 ns |
| p95 | 3,625 ns | 3,375 ns |
| p99 | 5,208 ns | 21,042 ns |
| max | 477,084 ns | 991,417 ns |
| refusal census | margin 33,334 / unauth 33,333 / oversize 33,333 / header 0 — exact plan match both runs | same |
| alloc census | 0 calls / 0 bytes | same |
| chain | 900,000 links, verify OK, head `8f1b9d26…` | same, head identical |

### Full mode (10M frames, `--ignored`, documented run)

| metric | value |
|---|---|
| wall | 45.084 s |
| throughput | 221,810 frames/s |
| admitted / refused | 9,000,000 / 1,000,000 |
| p50 / p95 / p99 | 1,542 / 3,792 / 6,708 ns |
| max | 769,597,083 ns (769.6 ms — socketpair backpressure stall when the producer thread was descheduled; one outlier, bounded, not a panic or loss: census conserved) |
| refusal census | margin 333,334 / unauth 333,333 / oversize 333,333 / header 0 — exact plan match |
| alloc census | 0 calls / 0 bytes |
| chain | 9,000,000 links (1.08 GB slab), verify OK, head `8df4a82b…` |

## MEASURED vs dossier (essay-claims law)

| dossier claim | status | measured reality |
|---|---|---|
| sub-10 ns risk gate | **TARGET-UNMEASURED** | the gate is measured only inside the full transfer function, which is transport-dominated (UDS socketpair syscall floor ~1.5 µs/frame). No gate-isolated ns/op number was produced by this bench. UNMET as measured end-to-end (p50 1.5 µs) |
| p99 < 1 µs | **TARGET-UNMEASURED / UNMET** | p99 = 5.2 µs (run 1) / 21.0 µs (run 2) / 6.7 µs (full) — 5x-21x the target; the number is transport floor + timer overhead, not gate cost |
| 20M+ frames/s | **TARGET-UNMEASURED / UNMET** | 484k frames/s best scaled run (UDS socketpair bound); ~41x short of the claim |

No dossier number is rounded toward its target. The sub-10 ns gate claim
is neither confirmed nor denied at the gate in isolation — this bench does
not fence it; it records that the end-to-end surface is orders of
magnitude off the dossier numbers under real transport.

## Standing

- ALIVE (scaled 1M x2 + full 10M, commands above, exit 0, receipts in-run
  asserts).
- Falsifier for this receipt: rerun the two commands; any census drift,
  alloc > 0, or `verify_records` failure reproduces as a loud assert.
- src/ untouched: zero lines changed outside the new test file and this
  receipt.
