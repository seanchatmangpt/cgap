# W6-BFX — bn3 fixpoint borrowed-prefix + owned-delta-tail + leased probe

Lane: BFX. Owner files: `crates/cgap-bn3/src/unify.rs` (fixpoint driver
region + evaluation-core genericization) and this receipt. Session
2026-11-02, repo `/Users/sac/cgap` (branch main).

## What changed

`crates/cgap-bn3/src/unify.rs`:

1. **ColView trait** (~line 407): private read-only row-source trait
   (`n()`, `w(slot,row)`) implemented for `StatementColumns` (exactly
   `col(slot)[row]`, zero-cost monomorphization) and for the new
   `SplitStore` (borrowed prefix + owned delta tail).
2. **SplitStore** (~line 447): borrowed-prefix + owned-delta-tail store
   view; row order `prefix || delta` in append order — identical to the
   retired full-store copy, so match order, derived-fact sets, and
   termination are unchanged (`seen` membership-only).
3. **Driver rewrite** (`unify_to_fixpoint_opt`): the three
   `cols.{s,p,o}.to_vec()` full-store heap copies are GONE. Derived
   rows append to small owned delta buffers (ds/dp/do) only; round view
   is `SplitStore{prefix: cols, &delta}` rebuilt per rule. Round-1
   snapshot borrows the caller's columns directly (zero copy at any
   round). `seen` still seeds from `cols` directly.
4. **Leased probe hoist**: one `JoinProbeScratch` hoisted to driver
   scope and leased through the generic core for every rule x round —
   `unify_batch_into`'s fresh probe per rule per round (old
   unify.rs:956) is retired in the driver path.
5. **Generic core plumbing**: `atom_local_match_into`,
   `eval_chunk_into`, `unify_batch_probe` (concrete pub signature
   preserved verbatim; forwards to `unify_batch_probe_view`), and
   `emit_join_masks` (pub concrete wrapper + `emit_join_masks_view`)
   are generic over `ColView`. `bucket_candidates` takes
   `key_at: impl Fn(usize) -> u64` instead of a slice (split-store rows
   are not contiguous per slot).

Public API stable: `unify_batch`, `unify_batch_into`,
`unify_batch_probe`, `unify_to_fixpoint(_opt)`, `emit_join_masks`
signatures unchanged; all external callers (benches, lever_courts,
bgp_hop_courts, cli handlers_reasoning) compile untouched.

## Semantics law

Row sequence presented to the evaluator is identical to the retired
copy (prefix then deltas in derivation order), so match order, match
sets, and termination are pinned by the existing differential and
stress courts:

- `cargo test -p cgap-bn3 --lib` — 119 passed, 0 failed (includes
  stress_court_1m_rows_converges_chunk_stable_matches_closed_form_model
  and the M12-6 chunk-partition courts). Exit 0.
- `cargo test -p cgap-bn3 --test lever_courts` — 7 passed, 0 failed
  (M-L3-1 radix-vs-nested and M-L3-2 alias differential courts rerun
  post-change). Exit 0.
- `cargo test -p cgap-bn3 --test bgp_hop_courts` — 2 passed, 0 failed.
  Exit 0.

## Regression gate + pre-existing failures (disclosed, not absorbed)

- `cargo test -p cgap-bn3` (all targets): lib + both court targets pass;
  the DOCTEST target compile-aborts in `project.rs:305/313/379`
  (`Vec` not found — no_std doctest scope). `project.rs` is another
  lane's in-flight file, untouched by BFX; compile-abort, 0 tests
  executed (census law: not a failure count). Pre-existing.
- `chains-bn3` bench binary panics at `chains-bn3.rs:812` with
  `derived push: ArenaCapacityExceeded { requested: 294913, bound:
  294912 }` (mat_closure / closure_ladder fixpoint legs). Identical
  panic in the HEAD archive tree, the reconstructed baseline scratch,
  and the post-change canonical tree — pre-existing, arena/bench
  region, not BFX ownership. Divan aborts before emitting tables, so
  the bench must be filtered by leg name.

## Performance law — what is grounded, what is not

Method: divan, `cargo bench -p bench --bench chains-bn3 -- <legs>`,
100 samples, bench profile, `CARGO_TARGET_DIR=/tmp/bfx-target` vs
`/tmp/bfx-before-target`. Baselines attempted:

1. HEAD archive (`git archive HEAD` into /tmp/bfx-before): INVALID —
   panics (ArenaCapacityExceeded) and predates W5/W6 working-tree
   state.
2. Reconstructed session-start baseline (working tree with my unify.rs
   edits hand-reverted in the scratch): INVALID as a perf reference —
   three legs (scan 2.326/2.334 ms, project 2.082/2.082 ms, nested
   join 7.993/7.814 ms at N=65536; 1.733/1.789 s at N=1M) agree with
   the post-change tree within noise, but the radix and deduction_chain
   legs diverge ~70x/~4.7x. Discriminator run (scratch + canonical
   unify.rs) reproduces the canonical numbers exactly (radix 23.31 ms,
   chain 1.653 ms / 27.19 ms) — the anomaly is a defect in my
   hand-reverted baseline reconstruction, NOT a regression from the
   BFX change. Baseline rejected; its numbers are not evidence.

Grounded post-change numbers (canonical tree, exit 0 runs this
session):

| leg | N=65536 | N=1048576 |
|---|---|---|
| deduction_chain | 1.674 ms | 27.46 ms |
| stage_compile_rules | 470.9 ns | 465.7 ns |
| stage_scan_24b | 125.7 us | 2.334 ms |
| stage_project_consequents | 128.3 us | 2.076 ms |
| stage_unify_join (nested) | 7.814 ms | 1.789 s |
| stage_unify_join_radix | 1.461 ms | 23.36 ms |

Driver fixpoint delta (the allocation reduction this lane exists to
deliver): UNMEASURED — the only legs that execute
`unify_to_fixpoint` (closure_ladder / mat_closure) are the ones that
panic pre-existing. Typed OPEN. Allocation column: divan emitted no
alloc column in this configuration — UNVERIFIED.

## Anti-vacuity

The M-L3-1/M-L3-2 differential courts in lever_courts are the named
mutation courts; they rerun green post-change (flip witnessed in
w5-l3-court-flips.md). A mutation that breaks the SplitStore row-order
invariant (e.g. evaluating delta-only) would flip the stress/differential
courts above; no new court was added this pass — the existing
differential + closed-form stress courts are the flip surface.

## Standing

ALIVE for: zero-copy driver (no full-store copy), delta-only owned
buffers, leased probe hoist, semantics pinned by 128 passing lib +
court tests this session (exits cited above). PARTIAL for: perf law
(no-regression grounded on agreeing legs; driver-leg delta UNMEASURED,
typed OPEN). Divergences: dispatch suggested "round 1 borrows cols,
later rounds snapshot the delta store" — implemented as a single
SplitStore view per round instead, because delta-only evaluation would
miss prefix-delta joins and change derived-fact sets; the dispatch's
own semantics law dominates.

## Commands

- `CARGO_TARGET_DIR=/tmp/bfx-target cargo build -p cgap-bn3` — exit 0
- `CARGO_TARGET_DIR=/tmp/bfx-target cargo test -p cgap-bn3 --lib` — exit 0
- `CARGO_TARGET_DIR=/tmp/bfx-target cargo test -p cgap-bn3 --test lever_courts` — exit 0
- `CARGO_TARGET_DIR=/tmp/bfx-target cargo test -p cgap-bn3 --test bgp_hop_courts` — exit 0
- `CARGO_TARGET_DIR=/tmp/bfx-target cargo bench -p bench --bench chains-bn3 -- <legs>` — exit 0 (filtered legs)
- scratch equivalents under /tmp/bfx-before (exit 0, baseline runs)
