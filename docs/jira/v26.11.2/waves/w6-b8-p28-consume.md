# w6-B8 — p28 consumes cgap-branchless::arithmetic (receipt)

# Verdict: PARTIAL — zero eligible swap sites; dispatch premise diverges from
# reality. Nothing swapped, dep NOT added (an unused dep on a publish=true
# crate is vacuous wiring; adding it without consumption is decoration).
# Regression baseline proven green this session (46 tests, exit 0).

## Census (grep, this session, exact)

- `saturating_sub|saturating_add|tripwire` in crates/cgap-p28/src/:
  ONE hit total — asc450.rs:1491, a TEST COMMENT describing an anti-vacuity
  mutation ("M3b chain-balanced saturating"), not arithmetic. Zero call
  sites.
- `0x5032_38FF`: trap.rs:36 only — `ERR_P28_SPOLIATION` trap constant
  (ABI-frozen, `const _:` assert at trap.rs:42-48 binds it to
  LICENSE_MAGIC). Not arithmetic; NOT swappable.
- The lease-counter decrement does not live in cgap-p28. The tick lease is
  monotonic INCREMENT (`Lease::tick()`, crates/cgap-crypto/src/lease.rs —
  A16 crate, outside this lane's ownership); p28 consumes it via
  `LeaseEpochSealer` (asc450.rs:364-415) which only calls `lease.tick()`.
- p28's only overflow-class arithmetic is `checked_add` → typed
  `Asc450Error::Overflow` (asc450.rs:824-834 `plan_event`). Swapping to
  saturating arithmetic would CHANGE behavior (Overflow refusal → clamp)
  and is explicitly forbidden by the crate's own anti-vacuity court
  comment (asc450.rs:1613-1614: saturating "masks the crossing near
  u64::MAX and CHANGES the" behavior). Per dispatch law ("mixed sites with
  side effects stay untouched" / "never absorb"), NOT swapped.

## Precondition state (polled this session)

- B1 scaffold landed: crates/cgap-branchless/ exists (untracked,
  `git status --porcelain` = `?? crates/cgap-branchless/`), lib.rs exports
  `arithmetic` with `saturated_subs_p28` / `poison_mask`.
- B2 branchless bodies NOT yet landed: arithmetic.rs:1-2 self-labels
  "NAIVE reference forms... branchless form lands in lane B2". Even if B2
  landed, no p28 site exists to consume it.
- cgap-branchless is not a root-workspace member (`??` untracked; no
  members entry) — irrelevant here since no dep was added.

## Commands (this session)

- `CARGO_TARGET_DIR=/tmp/b8-target cargo test -p cgap-p28` → exit 0.
  38 unit + 2 lever_courts (l5_upstream_receipt_flip_is_refused,
  l5_mixed_corpus_three_way_invariants) + 6 p28_invariant_courts
  (incl. bit_flip_sweep_640_integration) = 46 passed, 0 failed.
  asc450 byte-equality court (w3l5_fused_row_byte_equality...) inside the
  38-unit run — green. Wire bytes unchanged (zero source edits).
- Repeat run for count extraction: exit 0 again.

## Divergence rows (BLOCKED-class, never absorbed)

- D-B8-1: dispatch names "saturating lease-counter decrement / tripwire
  sites" in cgap-p28; census = zero such sites. The saturating/tripwire
  forms exist ONLY as produced functions in cgap-branchless::arithmetic
  (B1 naive reference). No lawful swap target.
- D-B8-2: dispatch step "add the dep" executed as NO-OP: cgap-p28 is
  `publish = true`; adding cgap-branchless (publish = false, untracked)
  as an unused path dep to a published crate is manifest debt with zero
  consumption. Deferred until a real site lands.

## Convergence path (for coordinator)

- Either a p28 site that genuinely wants saturating semantics lands
  (e.g. a quota/poison field on the meter) and then B8-redo adds the dep
  + swap + anti-vacuity mutation court, or
- The consumption lane's ownership should be re-scoped to a crate that
  actually owns saturating arithmetic (cgap-crypto lease tick ceiling
  logic is the nearest real decrement-class surface, owned by A16, not
  B8).
