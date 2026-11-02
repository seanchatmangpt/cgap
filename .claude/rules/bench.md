---
paths:
  - "crates/bench/**"
---

# Bench membrane law (Divan, alloc-column semantics, PARETO)

`crates/bench` is the ONLY place `divan` exists (`=0.1.21` exact pin);
kernel crates never gain a `[[bench]]` — they stay zero-dep `#![no_std]`.

- `divan::black_box` every measured input; each bench asserts a final
  checksum — a bench returning into the void verifies nothing. Alloc-
  column semantics: value-only allocs are LLVM-elided; the flip channel
  is **retained allocations**.
- Every bench names its anti-vacuity mutation and flips it in a /tmp
  sandbox (sha256-exact restore); non-flipping = FAIL, skip = loud.
- The **80/20 PARETO split is mandatory** per bench: name the ~80%
  segment before optimization work.
- **Inventory (2026-11-02)**: chains-* legs (`chains-bn3` incl.
  `closure_ladder` — its ArenaCapacityExceeded guard at :704 is
  load-bearing, never widen the arena to silence it; ingress/parquet/
  seal/sovereign), `mat_closure`, `dred_churn`; side-bin
  `src/bin/cpk_study.rs`.
- **DREDP fence law**: the per-lane rescue loop in cgap-storage dred.rs
  was swapped to SWAR and MEASURED -11% (839M vs 944M facts/s, matched
  runs) — scalar autovectorizes, SWAR does not; scalar KEPT
  (`repairs/repair-16-dred-swar-wire.md`). Fence: any future
  SWAR-vs-scalar swap in a hot loop requires a matched-run measurement
  before landing; scalar is the default.

Witnessed: `docs/jira/v26.11.1/waves/f2-15-bench-flips.md`
(FLIPPED-4/4, zero VACUOUS).
