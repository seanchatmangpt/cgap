# Lane GIT-04 Receipt: bounded Git delta resolver

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Standing: **ALIVE**
(scoped — see "Fence note" for concurrent-lane compile-aborts).

## Files written (lane-owned, all three dispatch-named)

- `crates/cgap-git/src/delta_resolver.rs` (289 lines)
- `crates/cgap-git/tests/delta_resolver_court.rs` (268 lines, #[path]-mounted; lib.rs untouched)
- this receipt.

## Reality fence (pack.rs audit — outcome: fresh applier, nothing duplicated)

`pack.rs` (130 lines) and `pack_write.rs` were read in full: they contain
NO delta-application primitives. Both explicitly refuse delta type codes
6/7 (`GitError::UnsupportedDeltaEncoding`, lib.rs:48-51/104-109). This
module is therefore the crate's FIRST delta applier — no reuse target
existed, and no second applier is being introduced silently.

## Scope conformance

- Real Git delta format: MSB-continuation varints (target size, source
  size); copy opcodes with bits 4-6 = offset byte count (3 bits — the
  first draft consumed bit 7, a real decoder bug the court caught; see
  Mutation log), bits 0-3 = size byte count, size 0 => 0x10000; insert
  opcodes bits 0-6 = literal length; opcode 0 = reserved => typed
  refusal.
- Fixed 11.6 KiB scratch: `SCRATCH_LEN = 11_878` (11.6*1024 floored),
  caller-provided `Scratch` value; chain resolution ping-pongs two
  caller-provided scratches via a side-tag (no heap, no swap-invalidate).
- Iterative resolution enforced: single flat `for` loop; no function in
  the module calls itself (court-asserted: `resolve_chain` appears
  exactly once, zero internal call sites).
- Max chain depth 16 (`MAX_CHAIN_DEPTH`), deeper => `ChainTooDeep{depth}`.
- Cycle detection: 64-bit FNV-1a fingerprint per delta, compared against
  all earlier fingerprints; repeat => `DeltaCycle{at: <index>}`.
- Zero heap: STATIC INVARIANT form chosen (stated per dispatch's "state
  which"): the module never references `alloc`/`std`/`Vec`/`Box`
  (source-scan court `module_source_has_no_heap_or_recursion`
  enforces); no tracking-alloc instrument was used.

## Courts (all with anti-vacuity flip twins)

| Court | Refusal/mutation | Flip twin |
|---|---|---|
| copy_insert_delta_is_byte_exact | — | full copy/insert delta, byte-exact vs independently computed target |
| empty_delta_reproduces_base | — | full-base copy op |
| chain_of_16_resolves_iteratively | — | depth-16 boundary resolves |
| cyclical_chain_is_refused_typed | A→B→A => `DeltaCycle{at:2}` | acyclic twin passes cycle detection and resolves |
| depth_17_chain_is_refused_typed | depth 17 => `ChainTooDeep{depth:17}` | depth-16 twin resolves |
| mutation_refusals_with_flip_twins | opcode 0 => `ReservedOpcode`; copy OOB => `CopyOutOfRange`; truncation => `TruncatedDelta`; produced≠declared => `TargetSizeMismatch` | injected-opcode twin applies byte-exact |
| oversized_target_is_refused_scratch_too_small | `ScratchTooSmall` | exactly-at-capacity (11_878 B) delta applies |
| module_source_has_no_heap_or_recursion | source scan: alloc/std/Vec/Box/recurse = 0; entrypoints defined once, zero internal call sites | — |

## Commands + exits (this session, exact subjects)

- `cd crates/cgap-git && CARGO_TARGET_DIR=/tmp/git04-target cargo test -p cgap-git --test delta_resolver_court` — exit 0, **8 passed / 0 failed**.
- `cargo test -p cgap-git --lib` — exit 0, **61 passed / 0 failed**.
- Remaining cgap-git binaries, exit 0 each: delta_refusal_court (4),
  envelope_decimal_court (2), fsck_court (2), large_offset (6),
  ladder_mutations (11), lineage_workflow (1), packfile_fuzz (2),
  git_interop (3); perf_baseline / ladder_timing_stress (1 ignored
  each, by design).
- `git status --short crates/cgap-git` — confirms delta_resolver.rs +
  court are untracked lane-owned files; no shared file edited.
- Full `cargo test -p cgap-git` currently compile-aborts (exit 1) in
  THREE OTHER LANES' in-flight binaries — see fence note.

## Fence note (must-not-regress surface; census law)

Compile-aborts are in OTHER lanes' untracked in-flight files
(admission_gate_court.rs — unclosed delimiter; retract_court.rs —
unterminated char literal / em-dash mojibake; egress_ring_court.rs —
`pop_tail` not yet defined on EgressRing). Census law: these are
compile-aborts (exit 1, 0 tests executed) in binaries owned by lanes
git-XX (admission gate / egress ring / retract), NOT test failures and
NOT this lane's diff surface. My diff is two new untracked files; it
cannot affect any other binary. Pre-existing binaries: all green above.

## Mutation log (defects the courts caught during manufacture)

1. **Offset-byte-loop bug (decoder)**: first draft consumed opcode bit 7
   (the copy marker) as a fourth offset-bit flag (`for i in 0..4`); the
   correctness court flipped with `CopyOutOfRange` on a valid delta —
   fixed to `for i in 0..3` (bits 4-6 only).
2. **Fixture arithmetic errors (tests)**: base offsets 5→4 ("quick"
   starts at index 4), copy past end-of-base, and a depth-16 chain whose
   insert bodies repeated every 10 hops (a REAL cycle in the fixture —
   cycle detection worked correctly and refused; fixture replaced with
   per-hop-distinct bodies).

# Standing: ALIVE (scoped)

All claims above are commands run this session against the exact
subjects. The "green crate" claim is scoped to everything that compiles
today; the three concurrent-lane compile-aborts are recorded, not
repaired (not lane-owned).
