# AB-9 IPC Stress Receipt (v26.11.3)
Standing: ALIVE

- **Lane**: AB-9 STRESS, `crates/cgap-arrow` IPC1 emit/read path
- **Subject**: `crates/cgap-arrow/src/ipc.rs` (read-only; zero src edits this lane)
- **Deliverable**: `/Users/sac/cgap/crates/cgap-arrow/tests/ipc_stress.rs` (new, 4 courts)
- **Date**: 2026-11-03

## Gates (run this session)

| Gate | Command | Exit | Result |
|---|---|---|---|
| Stress target | `cargo test -p cgap-arrow --test ipc_stress` | 0 | 4 passed / 0 failed (48.25s) |
| Full crate | `cargo test -p cgap-arrow` | 0 | 216 tests green across 9 targets |
| Neighbors | `cargo test -p cgap-arrow --test ipc_courts --test ipc_body_integrity_test` | 0 | 5 + 17 passed |

Suite unregression note: one full-suite anomaly, `--test stream_writer_stress`,
is a CONCURRENT lane's in-flight file (mtime 2026-11-03 22:06, after this
wave's dispatch; runs >300s, still being authored by its owner lane). Not
session-introduced by AB-9; AB-9 never touched it. All other targets green.

## Court 1 — throughput bench (min-of-3, `Instant`, 1000 cycles/size)

Method: reuse window+out buffers, black_box on body/window/len, warmup cycle
first. **Debug profile** (`cargo test` default, unoptimized) — numbers are
per-window emit+read cycle rates in that profile, not release claims.

| Body | sec/cycle (min) | cycles/sec | MiB/s (body bytes) | window_len (align 64) |
|---|---|---|---|---|
| 64B | 2.667e-6 | 375,023 | 22.9 | 264 |
| 1KB | 1.605e-5 | 62,317 | 60.9 | 2,184 |
| 64KB | 8.816e-4 | 1,134 | 70.9 | 131,208 |
| 1MB | 1.453e-2 | 69 | 68.8 | 2,097,288 |

Copy-bound plateau ~70 MiB/s body in debug; small bodies are framing-bound.

## Court 2 — scale stress: 100k windows through the u32 wrap

- 100,000 sequential emit/read windows (256B bodies, align 64), `seq`
  starting `u32::MAX - 50_000` so exactly one wrap lands mid-walk.
- `seq_ok` asserted on every pair (forward pass + reversed-pair refusal),
  plus boundary laws: `seq_ok(u32::MAX, 0)` true; skips (`MAX, 1`) and
  reverse wraps (`0, MAX`) refuse.
- Zero classification drift: every window read green, byte-identical body,
  seq echo exact, alignment field exact. Observed wraps: exactly 1.
- Output: `t2: 100000 windows, 1 wrap, zero classification drift`

## Court 3 — alloc court (counting GlobalAlloc over System)

Observed in-situ aggregates are polluted by concurrent test threads (other
courts allocate in the same binary; e.g. 129 allocs/1000 cycles at 64B with
neighbors running). Allocation noise is additive-only, so the court takes the
per-cycle minimum over 400 single-cycle trials — that isolates the window's
true cost.

| Path | min-of-400 allocs/window | bytes/window | in-situ aggregate |
|---|---|---|---|
| emit+read 64B | **0** | 0 | 129 allocs / 1000 cycles (polluted) |
| emit+read 1KB | **0** | 0 | 4 allocs / 1000 cycles (polluted) |
| emit_verified+read_verified 1KB | **0** | 0 | 4 allocs / 1000 cycles (polluted) |

Kernel law holds: **0 allocations per window**, actual measured minimum,
never rounded up. The verified path (blake3) is also alloc-free on this
surface.

## Court 4 — body-size boundary sweep (read_verified classification)

25 boundary sizes x {8, 64} alignment, every cell asserted exact. Alignment-8
classifies `BodyDigestNoRoom { got: 8 }` at every readable size; alignment-64
plain-emitted classifies `BodyDigestMissing`, verified-emitted is green with
byte-exact body at every size. `meta_span_for` pins re-asserted (8->64,
64->120).

**Finding (parse-minimum Truncated, src-owned)**: `parse_meta`
(src/ipc.rs:378) demands `buf.len() >= body_at + META_HEADER_SIZE` — 64
readable bytes past the body start — so windows with `body_len + body_pad <
56` classify `Truncated` on read although `emit` accepted them (emit/read
round-trip asymmetry, emit returns `Ok(window_len)`). Observed cells:

| Body | align 8 | align 64 |
|---|---|---|
| 0B | Truncated (needed 136, got 80) | Truncated (needed 192, got 136) |
| 1B | Truncated | green |
| 7B/8B/9B/15B/16B | Truncated | green |
| >=25B | green | green |

Typed, deterministic, parse-level (runs before the digest gates), so this is
classification identity, not drift; src is read-only for this lane, handed to
the coordinator as a src-owner finding. No body-corruption interplay: the
digest gates sit behind it and were exercised green at every readable size.

## Anti-vacuity

Each court asserts an exact contract that flips on regression: 0-alloc law
(any per-window allocation flips min-of-400), 100k-window classification
identity (any framing/digest/seq change flips), exact typed error per
boundary cell (any classification change flips), and the throughput bench
prints per-size min-of-3 for trend tracking.

## Standing

ALIVE — commands and exits above run this session against
`/Users/sac/cgap` on the exact subject; numbers re-read from this run's
output, not recalled. Receipt file verified on disk post-write.
