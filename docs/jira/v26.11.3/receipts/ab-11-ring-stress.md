# AB-11 — wire-shm ring stress courts

## Subject
- `crates/wire-shm/src/ring.rs` (read-only; no ring.rs changes made)
- New file: `crates/wire-shm/tests/ring_stress.rs` (269 lines, 4 courts)
- Standalone workspace; run via `--manifest-path crates/wire-shm/Cargo.toml`

## Courts

### Court 1 — churn (100k fill/drain cycles)
100,000 write+read cycles over a 1024-slot ring in 98 fill→drain waves.
Per frame: exact payload byte-compare + crc32 match + ordinal-match refusal
law exercised. Zero corruption. Eviction law asserted: reading an evicted
ordinal (`o < next - cap`) refuses `StaleOrdinal` typed, never silent-stale.
- Anti-vacuity: every frame byte-compared; a corrupt pass is impossible
  without the court failing.

### Court 2 — ordinal-wrap walk (u32::MAX → 0, 3 passes)
Ring semantics force a real constraint: `stamp_frame` stores next_ordinal as
u64, so writing u32::MAX advances next to 2^32 and refuses ordinal 0 — the
ring does NOT wrap on its own. The court therefore writes at the u32::MAX-c
edge, resets the u64 bookkeeping to 0 with real file bytes (real segment,
no mocks), and drives the boundary 3 times. Ordinal-match law held across
every wrap: pre-boundary frame (slot 255) intact after wrapped writes to
slots 0-2; post-wrap frames byte-identical; StaleOrdinal law survives wrap.
- Anti-vacuity: a wrap that changed slot addressing would fail read_frame
  on stored-ordinal mismatch.

### Court 3 — backpressure / overflow typed refusal
Fill to capacity → capacity+ overflow writes all refuse `StaleOrdinal
{ expected: cap, found }`; oversize (MAX_PAYLOAD+1) refuses `FrameTooLarge`.
Post-refusal drain of all 64 slots is byte-identical (state intact), refill
after drain passes, evicted reads refuse typed.
- Anti-vacuity: silent drop or slot corruption fails the drain compare.

### Court 4 — throughput (min-of-3 fill+drain, frames/sec)
| payload | frames/sec |
|---|---|
| 64 B | 49,283 |
| 4080 B (MAX_PAYLOAD) | 4,465 |
| 4 KB / 64 KB | NOT BENCHMARKED — exceeds MAX_PAYLOAD (4080); asserted as typed `FrameTooLarge` refusal instead. Loud skip, no fabricated numbers. |

Method: min-of-3 fresh-segment runs, per-frame integrity asserted in-run
(fast-but-corrupt fails the court). Debug build, macOS arm64, file-backed
(no mmap), std::fs pwrite. 64B number dominated by per-frame syscalls
(create does a full-segment zero-fill per run inside the timed section —
the 20k-frame fill+drain dominates, but the zero-fill is untimed? no: it is
timed; treat numbers as lower bounds).

## Gates (run this session, exact commands + exits)
- `cargo test --manifest-path crates/wire-shm/Cargo.toml --test ring_stress` → exit 0, 4 passed (run 1: 8.18s/14.72s/18.30s; run 2: 14.72s) — GREEN ×2
- `cargo test --manifest-path crates/wire-shm/Cargo.toml` (full suite) → exit 0: lib 1 passed, ring_stress 4 passed, wire_courts 8 passed — unregressed

## Session corrections (test-side, ring.rs untouched)
1. Churn eviction assert: evicted iff `o < next - cap`, so evicted ordinal
   is `next - cap - 1`; original `next - cap` was the freshest frame in the
   slot (read succeeded, unwrap_err panicked).
2. Throughput drain window: after 20k writes to a 4096-slot ring only the
   last `cap` frames are readable — ordinal-match law correctly refuses
   overwritten slots; drain window fixed to `(n - cap)..n`.

## Standing
- ring_stress: ALIVE (commands + exits above, this session)
- ring.rs: UNMODIFIED, UNREGRESSED (13 pre-existing tests green)
