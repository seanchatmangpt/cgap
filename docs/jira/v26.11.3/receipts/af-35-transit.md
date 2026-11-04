# AF-35 — Unified Transit Budget, Measured Not Summed

Standing: ALIVE — Verdict: REFUTED as stated

- **Subject**: `crates/cgap-aoq/tests/macro_transit_bench.rs` (new)
- **Status**: ALIVE — `cargo test -p cgap-aoq --test macro_transit_bench --release` exit 0, green ×2 on the final file (2026-11-03)
- **Claim audited**: dossier arithmetic `<10 ns admission + 41.5 ns decode + 34.7 ns slab push + 167 ns BLAKE3 ≈ 253.2 ns transit (3.9M frames/s)`
- **Verdict**: REFUTED as stated. Composed transit measures ~475 ns/frame (~2.1M frames/s). Two independent defects: the "<10 ns admission" was unsupported (first real measurement: **2.1 ns**, timer-free — the claim was conservative, not wrong, but it was never measured), and the additive model misses by **+44%** because the dossier's 167 ns BLAKE3 row does not correspond to any landed surface (raw 128 B hash is 276.8 ns; the landed AB-7 chain receipt is 456.3 ns because it performs TWO BLAKE3 invocations plus a record write).

## Method

Composed hot loop over LANDED surfaces only, 1M pre-built 128 B frames (build excluded from timing), release profile, `black_box` on every stage output, per-frame pipeline:

1. admission gate — AF-26 margin-ledger pattern (fixed `[u64; 10_000]`, wrapping u64 notional, raw little-endian wire loads, pre-decode)
2. header decode — AB-6 `codec::decode` (FNV-1a32 digest verify included)
3. slab push — AB-19 pattern, fixed 131,072-slot ring of 128 B frames
4. BLAKE3 receipt — landed AB-7 `AoqChain::append` via the `ChainDigest` seam over real `cgap-blake3` (payload hash + record hash)

Stage attribution: each stage isolated in an identically-shaped loop, timer-free (whole-loop wall / n). Percentiles from a separate composed pass with per-frame `Instant` (timer-inflated; measured floor 49.9 ns). Toxic frames: every 10th carries `u64::MAX` notional → typed margin refusal, counted. Digest substrate: real cgap-blake3, no fake.

## Composed-vs-summed table (measured 2026-11-03, aarch64 Darwin, release)

| Row | Dossier (TARGET-UNMEASURED) | Measured here | Delta |
|---|---|---|---|
| admission gate | <10 ns (unsupported) | **2.1 ns** | first real number; timer-free |
| header decode | 41.5 ns | **45.0 ns** | close (+8%) |
| slab push | 34.7 ns | **6.0 ns** | 5.8× faster than claimed |
| BLAKE3 receipt | 167 ns | **276.8 ns** raw 128 B hash; **456.3 ns** landed chain append | chain = 2× hash + record write |
| TOTAL | 253.2 ns (3.9M/s) | **475.0 ns composed** (2,105,153 frames/s) | **+87% over dossier** |
| summed(stage-measured, raw-hash row) | — | 329.9 ns | composed is +145 ns (+44%) over its own sum |
| summed(gate+decode+slab+chain receipt) | — | 509.4 ns | composed 475.0 is 34 ns FASTER than its sum (cache warmth) |

## Latency (composed, per-frame timer pass — timer-inflated)

- p50 500 ns · p95 584 ns · p99 625 ns · max 77,292 ns (one scheduler hiccup in 1M)
- measured `Instant` pair floor on this machine: **49.9 ns** — any per-call wall figure below ~50 ns is timer-limited, which is exactly why the isolated gate number (2.1 ns) is derived timer-free (whole-loop wall / n) and why the dossier's "<10 ns" could never have been measured per-call
- headline composed number is timer-free: 475.025 ms wall / 1M frames = 475.0 ns

## Courts (all witnessed this run)

- **0-alloc**: counting GlobalAlloc census across the composed loop AND every stage loop — 0 calls / 0 bytes (asserted)
- **chain integrity**: `verify_records` independent re-walk of all 900,000 landed links — OK
- **toxic refusal in-loop**: every 10th frame margin-refused with typed `Decision::MarginExceeded`; census 100,000 == plan; admit+refuse conservation 900,000 + 100,000 = 1,000,000; chain links == admitted
- **gates green ×2**: `cargo test -p cgap-aoq --test macro_transit_bench --release` exit 0 twice on the final file

## Findings

1. **The additive model is structurally wrong at the receipt row.** The dossier's 167 ns corresponds to neither landed BLAKE3 surface. The landed chain receipt (AB-7 law: `H(payload)` + `H(prev‖epoch‖idx‖payload_digest)`) is 456.3 ns — 2.7× the dossier figure. Any transit budget must pick its receipt surface explicitly.
2. **Summing errs in BOTH directions**: composed (475.0) exceeds its raw-hash sum (329.9) by +44% (pipeline dependencies, two chained hashes sharing nothing), yet is 34 ns FASTER than the chain-receipt sum (509.4) — cache warmth from stages sharing the same frame bytes. Neither direction validates the additive model.
3. **Admission measured for the first time: 2.1 ns** (raw-wire ledger check, timer-free method). The <10 ns claim is now supported — but by this receipt, not the dossier's.
4. Slab push measured 6.0 ns vs the dossier's 34.7 ns — the dossier stage-minima are not stage-minima.
5. Dossier total understates composed reality ~1.9× (253.2 vs 475.0 ns; 3.9M/s vs 2.1M/s derived) when the receipt is the landed chain. Against a raw-hash receipt the composed cost is ~330-475 ns — still 1.3-1.9× the dossier figure.

## Falsifier / replay

```
cargo test -p cgap-aoq --test macro_transit_bench --release -- --nocapture
cargo test -p cgap-aoq --test macro_transit_bench --release -- --ignored  # 10M mode
```

A refutation of this receipt is a composed run reporting ≈253.2 ns/frame with the same four landed stages. Numbers are machine-relative (aarch64 Darwin 25.2.0); the composed/summed divergence structure, not the absolute ns, is the durable finding.
