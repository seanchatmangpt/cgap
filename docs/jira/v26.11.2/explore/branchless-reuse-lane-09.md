# Branchless reuse map — lane BR9 (dred mask propagation)

Subject: `crates/cgap-storage/src/dred.rs` (NOT cgap-bn3, as dispatched).

## Divergence vs dispatch (recorded, not repaired — read-only lane)

Dispatch said REP4 is "in flight" adding the cascade fixpoint pass. It has
ALREADY LANDED: `cascade_retract` (dred.rs:321), `cascade_retract_mutant_nocascade`
(dred.rs:417, BFS-dropped anti-vacuity witness), `cascade_oracle` (dred.rs:437,
rounds-based least-fixpoint referee), plus `build_csr` + `CascadeError`. REP4's
residual work is court wiring, not kernel design.

## (1) Word-width verification + remaining per-lane loops

- Masks are u64, 64 lanes/word (`BITS_PER_WORD`, dred.rs:22); `suspect_mask`
  (dred.rs:44-51) is pure word-parallel AND — clean.
- REMAINING PER-LANE LOOP: `keep_mask` (dred.rs:70-74) walks lanes one at a
  time with a data-dependent branch:
  `if alt_support[lane] != 0 { k |= 1u64 << (lane - lo) }`.
  64 branchy iterations per word. This is the one residual scalar loop in the
  static hot path.
- Primitive to substitute: a SWAR nonzero-lane-bitmask fn, `u32` pairs packed
  into `u64` — `(v | (v | v>>32).wrapping_add(HI_BIT))` per pair → two
  high bits, then compress to a lane mask. Fits `cgap-branchless/src/penstock.rs`
  (sibling of `swar_all_zero_vl`, penstock.rs:64) as
  `swar_nonzero_u32x2_mask`. Gain class: branch-count reduction (64 → 2 per
  word) on the FA5-proven data-independent kernel; ~276.5 µs is a TARGET
  refinement, not a promised gain.

## (2) compact_lanes as the existing pattern; branchy siblings

- `compact_lanes` (dred.rs:98-109) is the canonical branchless pattern:
  `run += set`, unconditional store with `set`-guarded cursor — cite it as
  the in-repo exemplar.
- Branchy siblings: the del-mask SEED loop in `cascade_retract`
  (dred.rs:362-377) walks set bits via `trailing_zeros` — fine — but the BFS
  body (dred.rs:383-397) tests `purged[d]==0 && alt_support[d]==0` with a
  data-dependent branch per edge. Sibling-primitive:
  `pred_inc` (cgap-branchless/src/select.rs:60) covers enqueue-count
  predication; the purged-bit set is single-word OR (already branchless
  once `d` is decomposed). Edge iteration itself is inherently serial
  (CSR walk); the branch REMOVAL target is only the enqueue predication.
- No other compaction sites exist outside `compact_lanes` — checked
  dred.rs:44-470.

## (3) REP4 primitive spec (census-compatible)

Cascade already landed, so this is a spec for the residual court/fixpoint
wiring: the fixpoint pass needs word-parallel predicate accumulation, i.e.
`alt_support_count_word(alt: &[u32], word: usize) -> u64` — OR of
`(x != 0)` per lane into the word's bitmask, semantically identical to
`keep_mask`'s per-lane loop (dred.rs:70-74). New fn (or the penstock
`swar_nonzero_u32x2_mask` above); do NOT restate per-fact counting — the
word form is the census-compatible primitive so REP4's keep recomputation
per fixpoint round consumes one call per word instead of 64 branches.

## (4) Court reference

FA5 probe lives at `/tmp/fa5-probe` (deps: cgap-storage path dep; release
opt-level 3). Its mutant battery (`retract_unchecked`, `retract_inverted`,
`cascade_retract_mutant_nocascade`) is the cascade-flip witness. Anti-vacuity
already embedded: `cascade_retract_mutant_nocascade` on the N=10,000 chain
corpus yields 9,999 phantom survivors — a court that cannot distinguish it
from `cascade_retract` is VACUOUS (exit nonzero).

Standing: PARTIAL_ALIVE — map written from source read this session;
no commands beyond reads (read-only lane). UNVERIFIED: any perf claim.
