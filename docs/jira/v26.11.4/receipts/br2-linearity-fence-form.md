# BR-2: Linearity fence diagnosis (span flatness) — receipt

Standing: UNVERIFIED

- **Date**: 2026-11-04
- **Lane**: BR-2 (benchmark refinement)
- **Subject**: `crates/cgap-ffi/tests/receipt_chain_scale.rs` span-verify curve
- **Repo HEAD at compile**: working-tree measurement; base `ede3fe4d` (AOQ v3 wire law landing) — tree concurrently mid-edit by lane C1 (disclosed; measurement is of the working tree, not a pinned SHA)
- **Command**: `cargo test --release -p cgap-ffi --test receipt_chain_scale -- --nocapture --test-threads=1`, 5 invocations, serial
- **Exit**: 5/5 `test result: ok. 3 passed; 0 failed` (finished 3.80–4.04 s each)
- **Load witness (per invocation, `uptime` 1-min avg)**: 47.83 / 45.45 / 42.77 / 39.90 / 37.59 — machine under heavy concurrent load (lanes running); this is exactly the noise channel under test

## 1. Distribution table (per-span over 5 invocations, each cell min-of-3 in-test)

| span | min (ms) | median (ms) | max (ms) | spread (max−min)/min | per-link @ min-of-5 (µs/link) |
|---|---|---|---|---|
|---|---|---|---|---|---|
| 1 000 | 1.259 | 1.260 | 1.266 | 0.56% | 1.2590 |
| 10 000 | 12.598 | 12.646 | 12.669 | 0.56% | 1.2598 |
| 50 000 | 63.090 | 63.722 | 63.853 | 1.21% | 1.2618 |

Raw per-invocation SPAN-CURVE bests:

| inv | 1k | 10k | 50k |
|---|---|---|---|
| 1 | 1.266 | 12.662 | 63.814 |
| 2 | 1.260 | 12.644 | 63.722 |
| 3 | 1.260 | 12.646 | 63.640 |
| 4 | 1.259 | 12.669 | 63.853 |
| 5 | 1.259 | 12.598 | 63.090 |

## 2. Model check: T(links) = a + b·links, LSQ on min-of-5

- **b = 1.260 µs/link**, **a = −11.5 µs** (negative = zero within noise; no measurable fixed per-span overhead at these span sizes)
- Residuals vs the two-parameter model: 1k **+0.68%**, 10k **−0.08%**, 50k **+0.003%**
- Per-link flatness (min-of-5): mean 1.2602 µs/link, **max deviation 0.127%** — inside the ±0.5% fence with 4x margin
- The 50k-span invocation spread (1.21%) is wider than the 1k/10k spreads (0.56% each), consistent with a longer measurement window accumulating more scheduler noise, not span-dependence

## 3. Verdict: the OE-3 1.88% spread was measurement noise under load, not span-dependent cost

1. Per-link cost is FLAT: min-of-5 per-link costs 1.2590 → 1.2598 → 1.2618 µs/link, max deviation 0.127% from their mean. Span flatness passes ±0.5% at min-of-5 with 4x margin.
2. The two-parameter fit recovers a ≈ 0 (−11.5 ns fixed term, ~0.7% of the smallest span) — no amortization channel exists at these span sizes; a fixed-overhead model and a pure per-link model are indistinguishable here.
3. Max residual vs the linear model is +0.68% at 1k — the 1k point is the noisiest cell (shortest window) and sits on a −11.5 µs intercept, so residual-vs-linear-model at ≤0.5% would be the WRONG fence form here: it would fail on the intercept noise of a model whose intercept is provably zero.
4. **Fence form verdict: keep the flat per-link fence (±0.5% on per-link cost at min-of-N, N≥5), do NOT replace it with residual-vs-linear-model.** The replacement fence proposed for diagnosis (residual ≤0.5%) measures the model intercept's own noise at the 1k cell, not span-dependence.
5. Load is a real confounder at invocation granularity: invocation 4 took 271 ms for a 100k append (vs 173 ms typical) under load avg 39.9 — but min-of-N within and across invocations suppresses it (per-link flatness stayed at 0.127% despite 1-min load avg 47.8→37.6 across the wave).

## 4. Falsifier status

- OE-3's ±0.5% flatness fence at min-of-5: **PASSES** on this measurement (0.127% max dev).
- The 1.88% spread does not reproduce; classified as single-invocation load noise. If a future min-of-5 wave under load < 10 re-produces a > 0.5% per-link spread, this verdict is refuted and the drift is real — file it.

## 5. Replay

```bash
cargo test --release -p cgap-ffi --test receipt_chain_scale -- --nocapture --test-threads=1
# run 5x; collect [SPAN-CURVE] lines; per-link = best_ms / span
```
