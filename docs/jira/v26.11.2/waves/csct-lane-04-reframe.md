# CSCT Lane 04 — Stage 2 THE REFRAME (RFC-0042 §4)

Date: 2026-11-02 · Lane: CSCT-L4 · Repo: /Users/sac/cgap (canonical, no git writes)

## Files owned / written

1. `crates/csct/src/reframe.rs` — Stage 2 THE REFRAME:
   - `pub struct BaselineParams { facts, raw_mb, json_mb, heap_mb, shards }`
     with Default = RFC baseline (1,000,000 / 24 / 160 / 800 / 1360).
   - `pub fn evaluate(&BaselineParams) -> (Vec<StateAssertion>, bool)` —
     wire amplification ratio computed as json_mb/raw_mb, threshold 6.66;
     heap/json ratio; sharding entropy penalty derived at runtime from
     `shard_channels(n) = n*(n-1)/2` = 924,120 at n=1360 vs 0 at n=1.
     No metric value is a string literal in an assertion.
   - `pub fn reframe() -> StageResult` — stage_index 2,
     stage_name "THE_REFRAME", exit Pass/GateFail from the computed
     aggregate, BLAKE3 receipt over the assertion stream (cgap-blake3,
     real digest).
   - `#[cfg(test)]` 4 tests including the contracted ANTI-VACUITY MUTATION:
     `BaselineParams { json_mb: 24.0, ..default }` must flip the recomputed
     result to pass=false (`mutation_no_wire_amplification_flips_to_fail`),
     plus a second mutation axis (shards=1 -> penalty 0) and a
     receipt-not-a-stub court.
2. This receipt.

## Commands + exits (this session, exact subject)

| command | exit | note |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/csct-l4-target cargo test --manifest-path /tmp/csct-l4-scratch/Cargo.toml` | 0 | 9 passed / 0 failed (4 reframe + 5 types), 3 runs total, final run clean |
| `cp .../reframe.rs /tmp/csct-l4-scratch/src/` | 0 | sandbox copy, shared tree untouched |

Sandbox: /tmp/csct-l4-scratch mirrors types.rs (L2, byte-copied) + reframe.rs
with a 2-module lib.rs, because the shared `crates/csct/src/lib.rs` declares
six sibling modules (warmer/drowning/emotional/new_way/solution/... ) whose
bodies are lanes L3..L8 and have not landed — the crate in-tree cannot
compile yet (expected sequencing, per csct-lane-01-scaffold.md).

## Standing

UNVERIFIED:compile-pending — the stage logic is test-ALIVE in the sandbox
(9/9 passing against the exact owned file bytes, 3 runs, exit 0), but the
in-tree standalone build (`cargo test --manifest-path crates/csct/Cargo.toml`,
this session, exit 1) aborts on `error[E0583]: file not found for module
'court'` — court.rs is lane L10's file, not yet landed (not a failure count:
compile-abort, census-class). Flips to ALIVE when L10 lands court.rs and the
crate builds; csct is also not yet a root workspace member (coordinator owns
membership).

## Anti-vacuity

- Named mutation: json_mb: 160.0 -> 24.0 (kills wire amplification).
  Witnessed flip: wire assertion pass=true -> false, aggregate false.
- Second axis: shards: 1360 -> 1 (kills sharding entropy). Witnessed flip.
- Receipt court: reframe() receipt is deterministic, non-zero, and moves
  when a parameter moves (not a stub digest).

## Fences

- No shared file touched (lib.rs, types.rs, Cargo.toml all read-only).
- cgap-blake3 consumed as a dependency only; its tests unaffected.
- No gates/courts on other crates touch this diff. mtime: shared tree
  file created fresh; no sandbox restore into the shared tree occurred.

## Divergences

- Contract said "alloc_count: one Vec growth per assertion push, witnessed
  below" in my draft — landed as `alloc_count = assertions.len()` (count of
  assertions, deterministic; a growth-count would be allocator-dependent).
- REQUIRED_ROI_X = 352.0 verdict-falsity concern already on record in
  csct-lane-01-scaffold.md — untouched, coordinator decision pending.

## Divergence from lane doc

None beyond the above; the lane doc matched types.rs exactly.
