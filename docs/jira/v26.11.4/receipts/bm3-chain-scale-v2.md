# BM3 — Chain/Receipt Throughput (v2), cgap-ffi RingReceiptChain plane

Date: 2026-11-04 (UTC 20:36Z close). Lane: benchmark lane 3 (staggered vs lanes 1/4).

## Plane law block

This receipt measures ONLY the cgap-ffi RingReceiptChain plane
(`crates/cgap-ffi/tests/receipt_chain_scale.rs`). Never cross-cite onto
AoqChain (0.291 µs/link, af-21) or arcus (Go membrane, unmeasured). Each
plane's numbers are its own.

## Commands + exits

```bash
# stagger: lock touched /tmp/bm-lane3.lock at 2026-11-04T20:00:24Z;
# poll-waited for /tmp/bm-lane1.done + /tmp/bm-lane4.done — neither appeared;
# 30-min cap hit at 2026-11-04T20:35Z, proceeded per contract.
python3 scripts/run_checked.py --out /tmp/bm3_runs_v2.log -- \
  cargo test --release -p cgap-ffi --test receipt_chain_scale -- --nocapture
# exit=0 (run_checked propagates the command's own rc)
```

Raw log: `/tmp/bm3_runs_v2.log`. Verbatim run output:

```text
    Finished `release` profile [optimized] target(s) in 5.75s
     Running tests/receipt_chain_scale.rs

running 3 tests
[run 0] append 100k: 201.2ms | verify_all: 183.3ms | span(0..10k): 14.599ms
test tamper_at_50k_prefix_integrity_at_scale ... ok
[run 1] append 100k: 190.8ms | verify_all: 170.4ms | span(0..10k): 14.627ms
[run 2] append 100k: 184.1ms | verify_all: 169.3ms | span(0..10k): 14.430ms
[MIN-OF-3] append 100k: 184.1ms | verify_all: 169.3ms | per-1k wall first 0.001875/last 0.001708 ms (x0.91) | per-1k allocs first 2 -> last 2 | per-1k bytes first 266 -> last 266
test scale_append_verify_100k_census ... ok
[SPAN-CURVE] span 1000: best 1.336ms (20 checkpoints, min-of-3)
[SPAN-CURVE] span 10000: best 12.649ms (20 checkpoints, min-of-3)
[SPAN-CURVE] span 50000: best 63.330ms (20 checkpoints, min-of-3)
test verify_span_from_checkpoints_o_span_curve ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.24s
```

Source of record for run output: `/tmp/bm3_runs_v2.log` (lines 11-23);
the block above is a byte-faithful copy of those lines.

## Per-link derivation (from raw ms)

50k-span: 63.330 ms ÷ 50,000 links × 1000 = 63,330 µs ÷ 50,000 = **1.26660 µs/link**

(showing the division: 63.330 ms = 63,330 µs; 63,330 / 50,000 = 1.26660)

Curve cells, same division by their own span: 1.336 ms → 1,336 µs / 1,000 =
1.3360 µs/link; 12.649 ms → 12,649 µs / 10,000 = 1.2649 µs/link.

## Noise band (same-day samples)

Prior today (zp-17-gate10-settling.md 7-run table, same test, same machine):
R3-class 50k-span samples 63.197–63.619 ms (7 runs); V3 64.479 ms cited by
the lane contract. This run (8th sample): **63.330 ms**.

- Full same-day set of 50k-span bests: {63.197, 63.262, 63.306, 63.330, 63.501, 63.517, 63.606, 63.619} ms
- min 63.197 / max 63.619 → half-range 0.211 ms on mid 63.408 ms = **±0.33%**
- This run sits inside the prior band; no fence movement, no re-derivation.

Note on V3 64.479 ms: this run's test binary prints verify_all only for the
100k census (169.3 ms min-of-3); it has no 50k verify_all cell, so 64.479 ms
cannot be directly compared against this run's output cells. Reported as
cited, not re-measured.

## Fence witness

- `assert_release()` gate passed — **the SKIP line is absent**: `grep -c SKIP /tmp/bm3_runs_v2.log` = **0**.
- Compile line reads `Finished \`release\` profile [optimized]` — release profile witnessed.
- Anti-DCE fences (`profile_guard::fence`) ran on digest inputs/outputs (test source lines 180-200, 267-277, 310-311).

## Method

Release profile; min-of-3 internal per cell (test-internal, per its own
print); 1 measurement run this lane (8th same-day sample). Stagger: lanes
1/4 did not post done markers within the 30-min cap; measured under
elevated load (see witness) — disclosed, not hidden; the result landed
inside the same-day band anyway.

## Machine witness (verbatim)

```text
12:36  up  2:27, 3 users, load averages: 19.24 27.11 23.21
Apple M3 Max
16
51539607552
ProductName:		macOS
ProductVersion:		26.2
BuildVersion:		25C56
```

Fleet disclosure: this is benchmark lane 3 of a multi-lane benchmark fan-out
on the one canonical checkout `/Users/sac/cgap`; lanes 1 and 4 were
contracted to stagger (lock `/tmp/bm-lane3.lock` held; their done markers
never appeared before the 30-min cap). Load 16–40 during the window — lanes
1/4 activity on the same machine is the presumed cause; measurement proceeded
at cap per contract.

## Status

ALIVE: command + exit run this session against the exact subject
(`cargo test --release -p cgap-ffi --test receipt_chain_scale` → exit 0,
3 passed / 0 failed). 50k-span = 63.330 ms = 1.26660 µs/link, min-of-3
internal, inside the same-day ±0.33% band.
