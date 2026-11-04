# AF-22 Receipt — Per-Call Latency of `cgap_blake3::hash`

Lane: AF-22 IMPLEMENTATION. Date: 2026-11-03.
Bench: `crates/cgap-blake3/tests/percall_latency.rs`.
Closes: AF-16's named bench for the 18.1 ns TARGET (AF-2 audit row 1, ruled UNSUPPORTED).

## Subject

Single-call wall latency of `cgap_blake3::hash` over 64 B / 1 KiB / 64 KiB inputs.
Call overhead INCLUDED — that is the point (the 18.1 ns claim was a per-call
latency claim, not a throughput claim).

## Method

- Release profile (`cargo test --release -p cgap-blake3 --test percall_latency`).
- 3 rounds × 10,000 individually-timed calls each; `std::hint::black_box` on
  input and digest (no constant-folding, per house law).
- Per size: each round's median printed; best round (min-of-3 by median)
  supplies p50/p90/p99/min/max.
- Timer overhead (an `Instant::now()` pair, ~30-40 ns on this host class) is
  included in every sample by construction and is disclosed — the true kernel
  64 B median is therefore somewhat BELOW the 167 ns figure, but even the
  fastest single sample (83 ns) is >4x the claimed 18.1 ns, so the claim dies
  regardless of the timer's share.
- Determinism gate asserted before timing (Chicago: real kernel, asserted
  final state). No sleeps.

## Results (aarch64 darwin, 2026-11-03)

| size | rounds median (ns)          | best-round p50 | p90 | p99 | min |
|---|---|---|---|---|---|
| 64 B | 167.0 / 250.0 / 208.0       | **167.0**      | 250.0 | 334.0 | 83.0 |
| 1 KiB | 2375.0 / 2459.0 / 2541.0   | **2375.0**     | 2417.0 | 2458.0 | 2208.0 |
| 64 KiB | 35333.0 / 35292.0 / 35333.0 | **35292.0**  | 35541.0 | 64166.0 | 32791.0 |

## Coherence vs prior receipts

- 64 KiB p50 35,292 ns ⇒ 65536 B / 35292 ns ≈ **1768 MiB/s** — matches ab-12's
  measured 1768.8 MiB/s @64 KiB exactly. The bench agrees with the prior curve.
- 1 KiB p50 2375 ns ⇒ ~0.41 GiB/s — consistent with ab-12's small-buffer regime
  (its 64 B cell was 1.2 hash MiB/s throughput-equivalent, latency-noise
  dominated).

## Verdict

**18.1 ns scalar digest: REFUTED by measurement.** The honest per-call median at
64 B is **167.0 ns** (best of 3 rounds; p99 334 ns). The claim dies either way:
worse than claimed by ~9.2x at the median, ~18x at p99, and >4x at the single
fastest sample. No rounding up; numbers as printed by the bench.

Standing: ALIVE (this session, command + pass output above-equivalent in
`cargo test` log; receipt on disk).
