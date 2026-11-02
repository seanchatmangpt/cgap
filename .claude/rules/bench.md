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

Witnessed: `docs/jira/v26.11.1/waves/f2-15-bench-flips.md`
(FLIPPED-4/4, zero VACUOUS).
