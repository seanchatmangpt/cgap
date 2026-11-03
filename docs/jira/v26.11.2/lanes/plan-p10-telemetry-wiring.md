# P-10 — Telemetry Wiring, Design (d) (PLAN)

PLAN (session-delivered, transcribed 2026-11-02; not yet executed).

Design (d) of four considered; (a)-(c) rejected for manifest
edge / unsafe / borrow-split costs (below).

## 1. Design (d): per-ring plain-u64 RingStats INSIDE cgap-git

- A plain `u64` field set (`RingStats`) living INSIDE the ring
  structs in `crates/cgap-git` — no cross-crate manifest edge,
  no `unsafe` (the crate is `#![forbid(unsafe_code)]`), and the
  borrow checker enforces single-writer (stats mutate only
  through `&mut` ring methods; observers read after).

## 2. Call sites (push / refusals / quota)

- Push path: ingress push counter at ingress_ring.rs:334.
- Refusal paths: ingress_ring.rs:307-320 and :375-378
  (refusal reasons counted distinctly).
- Quota: publish path plus an `acknowledge_quota` wrapper so the
  ack leg is counted, not inferred.

## 3. cgap-ffi side: global() keeps sovereign/receipt paths only

- `crates/cgap-ffi` global() gains FOUR hook lines feeding the
  stats out; the sovereign and receipt-chain paths are the only
  wired surfaces. Fleet-ops consumes via an `FfiSloSource`
  feature-gated PATH dependency (gate keeps it out of default
  builds and out of publish).

## 4. Courts

- New `telemetry_court.rs`:
  - flood court: N pushes -> push counter == N (final state).
  - ack court: quota publish + acknowledge_quota -> ack counter.
  - mapping court: each refusal reason maps to its distinct
    counter (no conflation).
  - alloc-purity court: stats path mints zero heap allocations
    (counting global allocator — hand-written real impl, not a
    mock).
  - Anti-vacuity mutation: bump a counter WITHOUT the ring op
    (or drop a call site) in a /tmp sandbox copy -> the mapping
    court FAILs. sha256 restore + mtime bump.

## 5. Honest scope (disclosed residuals)

- XIP paths unwired.
- Lease counters producerless (no producer increments them yet).
- No C-ABI per-ring export (stats are Rust-internal this
  milestone).
- No production ring driver — courts drive the rings directly.

## 6. Standing

UNVERIFIED (plan transcribed, not executed). Line cites
(ingress_ring.rs:334, :307-320, :375-378) are the session plan's
cites — re-verified against disk only as file existence this
lane; line-level verification is the implementing lane's first
step. `forbid(unsafe_code)` attribute claim per CLAUDE.md
crate-profile note.
