# AF-33 — Benchmark 03: Kinetic Mass-to-Velocity Density
Standing: BLOCKED

Date: 2026-11-03 (PST) · Lane: AF-33 IMPLEMENTATION · Repo: /Users/sac/cgap
Subject: `crates/cgap-aoq/tests/macro_density_bench.rs` (new)
Basis: AF-26/AF-32 pipeline pattern (decode → gate → AEAD seal → chain
receipt), same composition as `af30_cage_match.rs` Contender A.

## Commands + exits (this session, exact subject)

```
cargo test -p cgap-aoq --test macro_density_bench -- --nocapture        # exit 0 (x2)
cargo test --release -p cgap-aoq --test macro_density_bench -- --nocapture  # exit 0 (x2)
```

All four runs green (debug ×2, release ×2). Host: Apple M3 Max, 48 GB,
macOS 25.2.0 (Darwin 25.2.0).

## Density table — MEASURED rows

| Metric | Debug build | Release build |
|---|---|---|
| Sustained window | 5.00 s fixed | 5.00 s fixed |
| Frames processed | 218,112 / 180,224 | 7,615,488 / 8,030,208 |
| Throughput | 43,614 / 35,416 fps | 1,522,914 / 1,606,023 fps |
| Peak RSS | 119,488 / 119,472 kB | 119,360 kB both runs |
| Plateau RSS (median, 2nd half) | 119,472 kB | 119,344 kB |
| Peak RSS MB | 116.69 / 116.67 | 116.56 |
| Density (fps ÷ peak-RSS-MB) | 373.8 / 303.6 | 13,065.2 / 13,778.2 frames/s/MB |
| Slab (1M AoqChainRecord) | 114.4 MB | 114.4 MB |
| Slab resets (typed HeaderTooLong) | 0 | 7 / 8 |
| Bench binary | 1,418,392 B | 976,464 B |

Release density ratio: ~13.1–13.8k frames/sec/MB peak RSS. The footprint
is slab-dominated: peak RSS ≈ slab (114.4 MB) + ~2 MB pipeline baseline —
corroborates AF-30's alloc-free census with an OS-level bound.

## Density table — TARGET rows (TARGET-UNMEASURED, never stated as measured)

| Claim | Status | Named bench |
|---|---|---|
| "under 2MB RAM" | TARGET — REFUTED-AS-STATED for the pipeline-with-slab shape measured here: peak RSS is 116.56 MB, dominated by the 1M-link chain slab the receipt law requires | The pipeline without the chain slab (decode+seal only) is the 2MB-class subject; not yet built — named bench below |
| 1000× density | TARGET | vs AF-30 Contender B (Go encoding/json pipeline), same host, same N — exactly AF-30's Contender B scope; the multiplier is recorded ONLY if that honest baseline lands and yields it |
| Container / JVM fleet comparison | TARGET | AF-30 Contender B on same host, same N (no 32GB-heap JVM fleet on this host to measure) |
| Espresso-machine power | TARGET | `perf stat` energy counters or external wattmeter — no power meter on this host |

## Artifact footprint (the "compact footprint" claim, measured)

- Release rlib `libcgap_aoq-0e8c1e41d3e8a722.rlib`: **36,480 B** — caveat
  per repo LINKED-ELF law: audit-profile rlib, not footprint truth.
- **Linked truth** (repo law: /tmp no_std caller, `#![no_main]` +
  panic_handler, z/fat-LTO/cgu=1/panic=abort, wasm32-unknown-unknown):
  **1,343 B** `.wasm` (anti-vacuity: sealed wire black_box-ed before
  decode; a pre-fix build without the black_box measured 1,266 B — the
  1,266→1,343 delta under a real input mutation is the non-vacuity
  witness). Caller crate: /tmp/af33-footprint (scratch, not repo).
- Host release bench binary (incl. test harness + cgap-crypto dev-dep):
  976,464 B — NOT a clean kernel-size measure, disclosed as such.
- Host ELF linked truth: BLOCKED on this host (macOS `_start` not
  linkable); wasm is the linked artifact of record.

## Bench design notes (disclosed)

- RSS sampled externally via `ps -o rss=` every 100 ms (POLL-STEP;
  bounded by the load window deadline — no coordination sleep).
- Per-frame wire is built lawfully: fresh `AoqHeader` → `encode` →
  `decode`. AF-30's raw `wire[20]` poke breaks the current codec's
  FNV-1a32 digest over [0x00,0x3C) — offset 20 IS the digest-covered seq
  field (witnessed: DigestMismatch panic during development of this
  bench; AF-30's landed status is NOT re-litigated here).
- Slab-full at the 1M-link boundary is the typed `HeaderTooLong`
  refusal; the bench resets to a fresh chain over the same slab and
  counts resets (disclosed; no slab growth).
- Debug throughput includes per-frame encode; both profiles run the
  identical path.

## Standing

ALIVE — four green court runs against the exact subject this session.
Density multiplier claims remain TARGET-UNMEASURED until the AF-30
Contender B baseline lands on this host.
