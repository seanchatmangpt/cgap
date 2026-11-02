# Lane B4 — Eytzinger Branchless Primitives — Receipt

Lane: B4 (branchless, eytzinger) · Repo: /Users/sac/cgap · Branch: main ·
Date: 2026-11-02 · Files: `crates/cgap-branchless/src/eytzinger.rs` (+ this receipt)

## Standing: ALIVE

## Files written
- `crates/cgap-branchless/src/eytzinger.rs` (306 lines; ONLY owned file)
- `docs/jira/v26.11.2/waves/w6-b4-eytzinger.md` (this receipt)

## API delivered
- `eytzinger_next(curr, is_less) -> usize = 2*curr + (is_less as usize)` — arithmetic only
- `eytzinger_restore(index) -> usize = index >> (index.trailing_ones() + 1)` — spec formula, verbatim
- `eytzinger_search(eyt: &[u64], target: u64) -> Option<usize>` — reference driver (natural loop,
  documented as such; the per-step primitives are the branchless units)
- `to_eytzinger_layout(&[u64]) -> Vec<u64>` — host-side convenience (allocates; documented)
- `EYTZINGER_PAD = u64::MAX` — pad sentinel; precondition: inputs must not contain it

## Semantics (derived, not assumed)
Prescribed restore formula is EXACT upper_bound under the orientation
`is_less = (element <= target)` (descend right):
- restore(final_k) = layout slot of the FIRST element STRICTLY GREATER than target
- restore = 0 when target >= every element ("end" sentinel)
- search returns None iff that slot holds the pad sentinel
Verified exhaustively in a python derivation first (n=1..199, all in-range targets,
distinct + duplicate inputs, zero failures), then in Rust.

## Divergences (recorded per contract)
1. B1 oracle absent at lane time (only lib.rs + Cargo.toml existed when I polled; the
   sibling files landed mid-lane) — no competing restore rule existed to match; the
   prescribed formula was used verbatim and its exact semantics derived empirically.
2. Dispatch's suggested orientation `is_less = "target < element"` is degenerate under
   the prescribed restore; exact orientation is `is_less = (element <= target)`.
3. Layout PADS the sorted input with `u64::MAX` up to `m = 2^L - 1 >= n` BEFORE the
   midpoint layout. Without padding the restore is only exact on complete trees
   (n = 2^k or 2^k-1); pads placed at the in-order tail keep the implicit tree a
   valid BST, which is what makes the restore exact for ALL n. Verified: without
   structural padding the mid-recursion overflows slot numbering for n=5,6,9,...;
   with tail padding, exhaustive pass n=1..199.
4. `to_eytzinger_layout` returns length m+1 (not n+1): sentinel + real slots 1..=n +
   pad fillers n+1..=m.

## Courts run (all in /tmp sandbox copy, sha256 fail-closed restore)
- M1 (anti-vacuity): mutated `eytzinger_next` to `2*curr + 1` (predicate killed) —
  5/6 tests FAILED (next_is_pure_arithmetic, classic_traversal, edge_shapes,
  differential, property) → court FLIPS. Restore sha256-exact:
  6e335e7adff57c5754fc096890c46d09805cc86de9fa5f5894344cbc7e28c80e before, during
  shared tree, and after restore. Shared tree never mutated.
- M2 (asm witness, release, aarch64, `--emit=asm`): traversal loop body is exactly
  `ldr; cmp; cset ls; orr x8, x10, x8, lsl #1` — the step is ONE orr (2k+bit), zero
  predicate branches; restore compiles to `mvn/rbit/clz/add/lsr` — pure arithmetic.
  The only branch in `eytzinger_search` is the loop back-edge (`b.ls`), which is the
  documented reference driver, not a primitive. The two primitives are
  `#[inline(always)]` and have no standalone asm symbols (fully inlined — honest
  record: the witness is the inlined form at the two call sites).
- B1's own `branchless_census.rs` integration test also passes with my file in tree
  (full crate suite exit 0).

## Commands + exits (this session)
- sandbox build+test (stub siblings): `cd /tmp/b4-sandbox && cargo test` → 6 passed, ok
- mutation court M1: `cargo test eytzinger` on mutated copy → 5 tests FAILED (exit 1), restore exact
- asm emit: `cargo rustc --release --lib -- --emit=asm -C debuginfo=0` → ok, .s inspected
- shared-tree gate: `cargo test -p cgap-branchless eytzinger` → **EXIT 0**
  (6 passed, 0 failed, 20 filtered)
- full crate suite: `cargo test -p cgap-branchless` → **EXIT 0** (26 passed, 0 failed)

## Cleanup
Build lease `/tmp/b4-target` and sandbox `/tmp/b4-sandbox` deleted (python rmtree).
Scratch derivations removed. No git writes; coordinator owns transitions.
