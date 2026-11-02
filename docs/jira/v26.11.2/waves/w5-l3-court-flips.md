# W5-L3 Court Flip Receipt (M-L3-1 / M-L3-2 kernel mutation shims)

- Lane: A2 (Analyze), repo /Users/sac/cgap, branch main, 2026-11-02.
- Sole write: this receipt. Repo tree byte-untouched: `crates/cgap-bn3/src/unify.rs`
  sha256 before == after == `d662efd924e50fcfcaadbf694fbdd25d03330a4b594a464736989c723835c785`
  (witnessed at lane start, between mutants, and at lane end). No git writes.
  All mutations ran on the /tmp sandbox copy (`/tmp/a2-sandbox`, workspace shim
  `Cargo.toml` + crates `cgap-bn3` + `cgap-blake3` + LICENSE.md), build root
  `CARGO_TARGET_DIR=/tmp/a2-target`.
- Closes the UNVERIFIED row at `docs/jira/v26.11.2/waves/w2-12-l3-court.md:57`.

## Baseline (real kernel, no mutation)

- `cd /tmp/a2-sandbox && CARGO_TARGET_DIR=/tmp/a2-target cargo test -p cgap-bn3 --test lever_courts`
  -> **7 passed; 0 failed; exit 0** (67.64 s). Baseline is green against the
  byte-identical real kernel.

## M-L3-1: bound-slot join probe dropped — **FLIP (exit 101)**

- Mutation (SHIM-COURT, sandbox copy of `unify.rs` only): in BOTH join arms
  (radix probe `unify.rs:1230-1237`, nested probe `:1256-1262`), the
  `for &(si, vidx) in &sc.bound_slots { ok &= lane_eq_mask(...) }` accumulation
  is deleted, leaving `ok == MASK_ALL` unconditionally — candidates pass
  regardless of bound-slot equality.
- `cargo test -p cgap-bn3 --test lever_courts` -> 5 passed, 2 FAILED
  (`l3_differential_join_court`, `l3_noalloc_radix_differential`).
- Focused rerun `... -- l3_differential_join_court` -> **exit 101**,
  `N=64 chunk=1: match count  left: 238  right: 9` — the mutated kernel
  set is strictly larger than the naive oracle, exactly the in-test
  anti-vacuity arm's predicted direction. `l3_radix_vs_nested_differential`
  and `l3_radix_vs_nested_100k` stay green (both arms mutated identically —
  expected; the flip lands on the oracle differential).
- sha256 restore after mutation: sandbox `unify.rs` ->
  `d662efd9...35c785` (byte-exact, verified) + post-restore court rerun
  -> 2 passed, exit 0.
- Verdict: **FLIP CONFIRMED** — the differential court binds the real kernel.

## M-L3-2: comparator aliased (lane_eq_mask constant-equal) — **FLIP (exit 101)**

- Mutation (SHIM-COURT, sandbox copy): `fn lane_eq_mask` body
  (`unify.rs:571-580`) replaced with constant `u64::MAX` — every value pair
  compares equal (the binding-alias comparator class, doe-4 Section 8).
- `cargo test -p cgap-bn3 --test lever_courts -- l3_differential_join_court
  l3_antivacuity_probe_drop_flips` -> **exit 101**: `l3_differential_join_court`
  FAILED at `N=1 chunk=1: match count  left: 1  right: 0` (kernel accepts a
  non-join pair the oracle rejects); `l3_antivacuity_probe_drop_flips` stays
  green (in-test oracle arm, kernel-independent). Non-empty corpus + aliased
  comparator => kernel over-matches on the smallest corpus already.
- sha256 restore after mutation: byte-exact `d662efd9...35c785`, verified.
- Verdict: **FLIP CONFIRMED** — the differential court binds `lane_eq_mask`
  arithmetic, not a shadow replica.

## Standing

- `l3_differential_join_court`: ALIVE (baseline exit 0 + both mutants flip
  exit 101, all run this session against the real kernel in sandbox).
- M-L3-1 kernel flip: **ALIVE** (was UNVERIFIED, w2-12-l3-court.md:57).
- M-L3-2 kernel flip: **ALIVE** (was UNVERIFIED).
- W2-12 receipt UNVERIFIED row: **CLOSED** by this receipt.

## Divergences

- Dispatch cited "drop the radix probe (force `radix: false` default) at
  unify.rs:612-666" for M-L3-1. Forcing `radix: false` is a micro-architecture
  selection, not a semantic mutation — it cannot produce a strictly larger set
  (the `l3_radix_vs_nested_differential` court pins radix/nested identity and
  stayed green). The W2-12 receipt (`w2-12-l3-court.md:41-52`) already
  corrected the locus to the bound-slot probe drop (radix `:1230-1237`,
  nested `:1256-1262`), which is the mutation whose semantics the in-test
  anti-vacuity arm replicates and the only mutation satisfying the dispatch's
  own flip condition ("strictly larger set"). The bound-slot drop was applied.
- Sandbox build needed a workspace shim Cargo.toml (bn3's crate manifest uses
  workspace-inherited keys); no repo manifest read-modified.
- Mutant M-L3-1 run cost 213.93 s (debug build of mutated kernel, full test
  binary); focused-filter runs used for the recorded exit codes.

## Cleanup

- `/tmp/a2-target` build-root lease deleted via
  `python3 -c "import shutil; shutil.rmtree('/tmp/a2-target', True)"`
  (confirmed gone, `ls` exit 1). `/tmp/a2-sandbox` retained only as court
  evidence? No — deleted in the same step; the repo-tree sha256 triple above
  plus this receipt is the replay record.
