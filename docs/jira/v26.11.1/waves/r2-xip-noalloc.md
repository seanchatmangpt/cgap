# r2-xip-noalloc — P3-11 W2-9-REDO

Lane P3-11-noalloc (lane-runner). Repo `/Users/sac/cgap`, branch main,
2026-11-02. Lease: `crates/cgap-bn3/src/noalloc.rs` (new) + ONE `mod`
line in `crates/cgap-bn3/src/lib.rs` + this receipt. No git writes.

## Standing

**ALIVE** — no-alloc XIP eval/unify arm landed with flipping courts;
full bn3 suite green this session (119 passed / 0 failed / 0 ignored,
`cargo test -p cgap-bn3`, exit 0, /tmp/p311-target deleted post-run).

## Subject

- `crates/cgap-bn3/src/noalloc.rs` (~840 lines): fixed-capacity,
  allocation-free eval/unify arm mirroring `unify.rs`'s u64-lane
  comparator semantics (`lane_eq_mask` XOR/neg/OR/sub arm, nested-loop
  join, first-occurrence binding order, atom-0 row-ascending match
  order). No `Vec`, no `HashMap`, no `alloc::` in the evaluation path;
  hard `const` bounds (NA_MAX_CONSTS=64 = the R0..=R63 band,
  NA_MAX_VARS=128 = the R64..=191 band, NA_MAX_ATOMS=8,
  NA_MAX_MATCHES=2048) exceeded only as typed `NaError::*Capacity`
  refusals, never reallocation or truncation. Const asserts pin the
  band widths to `crate::unify`'s frozen ABI.
- `crates/cgap-bn3/src/lib.rs`: one line, `pub mod noalloc;`.
- The alloc arm (`unify.rs`) untouched; it remains the semantics
  authority (differential court asserts binding-for-binding equality).

## Courts (in-module `#[cfg(test)] mod courts`, 8 tests)

1. `differential_court_noalloc_matches_alloc_arm_over_1200_rows` —
   seeded (0xBF58_476D_1CE4_E5B9) 1200-row corpus, full five-rule
   corpus of spec 7.1, both arms, two chunkings (4096 / 1): match
   count, per-atom store rows, and every variable binding equal.
   Flips under the M2 alias shim (below).
2. `alias_trap_court_register_number_never_stand_for_a_value` — trap
   store carrying raw register number 65 as a subject word; the value
   probe rejects it. Flips under M2.
3. `zero_alloc_court_na_unify_allocates_zero_after_setup` — 1200-row
   corpus, all five rules, measured window delta 0 against the
   counting allocator (`current_allocs`, same thread-local substrate
   the W2-9 unblock landed in `unify.rs`).
4. `zero_alloc_court_positive_control_counter_is_live` — anti-vacuity:
   fresh `Vec` MUST move the counter.
5. `compile_court_typed_refusals_mirror_alloc_arm` — VarExi sigil,
   FormulaRef-in-p, register payload > 255, unbound head var, empty
   body — all typed refusals identical to the alloc arm.
6. `capacity_court_consts_vars_atoms_are_typed` — AtomCapacity via
   9-atom rule. Recorded: with NA_MAX_ATOMS=8 the Const/Var interning
   guards (64/128 wide) are structurally unreachable through
   `na_compile` (max 24 distinct terms per rule); their typed variants
   exist as interner-level guards, recorded not silently dropped.
7. `capacity_court_match_overflow_is_typed_not_truncated` — a rule
   matching NA_MAX_MATCHES+10 rows refuses `NaError::MatchCapacity`,
   never truncates.
8. `corpus_court_same_seed_is_identical` — seeded generator
   determinism.

## Sandbox anti-vacuity flips (/tmp/p311-sandbox, sha256 fail-closed)

Sandbox = copies of `cgap-bn3` + `cgap-blake3` under a generated mini
workspace (`[workspace.package]` needed authors/repository/
license-file/description inherited by both manifests — added during
setup). Baseline: 8/8 noalloc courts ok, restore sha256
`033ef367be200b99…` byte-exact verified.

- **M1 (inject alloc)**: `Vec::with_capacity(8)` per `na_unify` call →
  `zero_alloc_court_na_unify_allocates_zero_after_setup` FAILED
  (delta != 0). FLIP.
- **M2 (alias comparator)**: bound-slot probe compares
  `VAR_BAND_FIRST + vidx` (the register NUMBER) instead of
  `engine.cur_vars[m][vidx]` (the bound VALUE) →
  `alias_trap_court...` FAILED and
  `differential_court...` FAILED. FLIP x2.
- Restore byte-exact (sha256 match), post-restore run 8/8 ok.

First M1 attempt ran OUTSIDE the sandbox (cwd-reset between Bash
calls compiled the real repo instead of /tmp — the court passed
because the shim was not in the compiled subject). Witnessed, fixed
by prefixing `cd /tmp/p311-sandbox`; the real tree was never mutated.

## Gate poll (task item 1)

- `p3-pub-bn3-gate.md`: bounded poll (10s cadence, ~17 min) — LANDED.
  Divergence (doc-vs-reality, recorded): the marker's standing is
  **BLOCKED** (cgap-gate publish HTTP 400 missing description/license;
  cgap-bn3 publish NOT attempted, budget preserved), not ALIVE.
  Proceeded on the operative condition — the bn3 SOURCE tree is
  quiescent (P3-4's receipt: noalloc residue absent, 111/0 green,
  publish consumed as that lane's own attempt) — the publish state is
  a registry concern, orthogonal to src editing.
- `f3-09-pub-blake3.md` witness: ALIVE (registry 26.11.1,
  num_versions 2 exactly, witnessed 2026-11-02T02:16:09Z,
  num_versions==2 no double upload).

## Defects found and fixed this session (all pre-receipt)

1. `push_match` copied the parent match from the nxt buffer instead of
   cur (join arm) — bindings and rows lost on join (probe: out
   vars [0,0,3] vs expected [1,2,3]). Fixed: push_match takes explicit
   parent/dst buffers.
2. `base + 1` overflow when base = usize::MAX (atom-0 sentinel) in the
   split_at_mut block. Fixed: split only on the join arm.
3. `Box::new(NaEngine::zeroed())` materialized ~2 MiB zeroed temporaries
   on the 2 MiB test-thread stack → stack overflow. Fixed:
   `Box::new_zeroed().assume_init()` (heap-zeroed, no stack temp).
4. Whole-array `out.rows = engine.cur_rows` drained via a ~1 MiB
   debug frame; replaced with row-wise copy loops.
5. Join match counts exceeded the initial 1024 capacity on the
   differential corpus; NA_MAX_MATCHES = 2048 + 10%-P2 selectivity in
   the generator (documented knob).

## Divergences (spec/doc vs reality)

1. Gate marker standing BLOCKED (publish), not ALIVE — see above;
   tree-quiescence was the operative lease condition.
2. Dispatch said "the prior file is crash residue" for the receipt —
   none existed on disk (clean write).
3. Dispatch projected the W2-9 attempt's residue in bn3 src — P3-4's
   receipt confirms it was already removed; the `pub(crate)
   current_allocs` unblock hook survived (used by court 3).
4. NA_MAX_ATOMS=8 makes the Const/Var capacity variants unreachable
   through `na_compile`; recorded in the court doc rather than
   widening the atom bound (widening would quadruple the engine
   buffer already sized by NA_MAX_MATCHES).

## Cleanup

/tmp/p311-sandbox, /tmp/p311-sb-target, /tmp/p311-target removed via
python3 shutil.rmtree, verified absent.
