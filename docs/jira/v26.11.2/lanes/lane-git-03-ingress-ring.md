# Lane GIT-03: Ingress Ring (pack-ingress staging)

- Scope: `crates/cgap-git/src/ingress_ring.rs` (additive, lib.rs untouched),
  `tests/ingress_ring_court.rs` (self-contained `#[path]` wire-in),
  this lane doc.
- Module: 32,768-byte circular SPSC ring, inline `[u8; CAP]`, zero heap,
  `#![forbid(unsafe_code)]`, no_std-compatible (`core::` only).
- Concurrency: free-running `usize` head/tail counters in cache-line-padded
  atomics (`#[repr(align(64))] PaddedIndex`, size/align const-pinned at
  module scope); `&mut self` single-handle SPSC with Acquire/Release
  publish law; full ring distance==CAP, empty distance==0 (all CAP bytes
  usable).
- Refusals (typed, never silent): `Overflow{head,tail,frame_len}`,
  `FrameTooLarge{frame_len}`, `Empty`; all map to Andon exit class 101
  (`IngressError::andon_exit_class`, pinned by court).
- SWAR sieve: in-module re-implementation of the `cgap-branchless`
  `swar.rs` range-mask contract (`HI`-mask only; `LO` unused by the
  predicate and not replicated) — disclosed divergence from the dispatch's
  `#[path]` option: `cgap-git` is a zero-external-dep published kernel;
  compiling foreign source via `#[path]` would fake an undeclared
  dependency edge.
- Latency: batch-amortized Instant over 4096 sieved 64B pushes (drain-at-
  capacity included), REPORTED in release profile only; per-frame Instant
  literal is impossible on macOS (disclosed per dispatch). Measured this
  session: 4 ns/frame vs 10 ns/frame TARGET.
- Courts (7): wrap byte-exactness with pinned two-segment boundary
  geometry (300 bytes to the physical edge, 100 wrapped), overflow typed
  refusal + fail-closed state, non-tautological overflow flip twin
  (mutation: delete `would_overflow` guard), sieve vs naive scalar oracle,
  non-tautological sieve flip twin (mutation: swapped min/max band args on
  the real code path), tiny-ring wrap (CAP=16), release-report.
