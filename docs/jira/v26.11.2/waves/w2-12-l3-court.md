# W2-12 / L3 Court Shell — Receipt

- Lane: W2-12 (lane-runner), repo /Users/sac/cgap, branch main
- Date: 2026-11-01
- Sole writes honored: `crates/cgap-bn3/tests/lever_courts.rs` + this receipt. No git
  commands. /Users/sac/stardf untouched. lib.rs/Cargo.toml untouched.

## What was built

`crates/cgap-bn3/tests/lever_courts.rs` (205 lines, 2 tests) per
`docs/jira/v26.11.2/07-court-corpus.md` Section 2 (L3 differential shell):

1. `l3_differential_join_court` — real public surfaces only
   (`RuleCompiler::compile` → `unify_batch`), comparing the nested-loop join path
   (unify.rs:793-880) against a naive raw-word hash-join oracle at
   N ∈ {1, 64, 4096} (100_003 deferred to W3 lever bench, per dispatch).
   Full cross-product chunk sweep chunk ∈ {1, 64, 4096} per N. Asserts
   match-for-match identical binding sets (canonical 5-tuple key sets:
   atom-0 row, atom-1 row, ?x/?y/?z bindings).
2. `l3_antivacuity_probe_drop_flips` — in-test non-vacuity arm: the oracle is
   re-run with the M-L3-1 semantics (bound-slot probe dropped, prior binding
   kept) and MUST produce a strictly larger set on the N=4096 corpus, else the
   court is VACUOUS. This flips without touching the shared tree (kernel
   mutations land in W3).

## Seed law

CourtRng = canonical splitmix64, seed `0xBF58_476D_1CE4_E5B9` only.
`0xA20C_0FFE_E5A7_E000` is the bitslice corpus-spread constant, never used as a
seed.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/w2-12-target cargo test -p cgap-bn3 --test lever_courts`
  → 2 passed; 0 failed (exit 0), twice (after compile fixes). Method: cargo test
  profile, debuginfo.
- Build root lease deleted: `python3 -c "import shutil; shutil.rmtree('/tmp/w2-12-target', True)"`
  → confirmed gone (`ls` exit 1 = No such file or directory).

## W3 mutation loci (exact, verified this session)

- **M-L3-1 drop-probe**: the bound-slot join probe,
  `crates/cgap-bn3/src/unify.rs:858-864` — `let mut ok = MASK_ALL;` /
  `for &(si, vidx) in &sc.bound_slots { ok &= lane_eq_mask(...); }` /
  `if ok != MASK_ALL { continue; }`. Semantics replicated in-test by the
  anti-vacuity oracle arm (flips: strictly larger mutated set).
- **M-L3-2 alias comparator**: `fn lane_eq_mask` body,
  `crates/cgap-bn3/src/unify.rs:560-563` (fn at :565; the mutate-able body
  lines `let d = a ^ b;` / `let nz = (d | d.wrapping_neg()) >> 63;` /
  `nz.wrapping_sub(1)` are :563-574). The dispatch-cited :560-563 span covers
  the `#[inline(always)]` attribute + fn signature; the W3 shim targets the
  value-compare arithmetic at :563/:573-574.

## Standing

- lever_courts differential court: **ALIVE** (command + exit 0 this session).
- M-L3-1 / M-L3-2 kernel mutation flips: **UNVERIFIED** — shims land in W3;
  the shell proves the assertion harness binds (in-test oracle flip), not the
  kernel flips.

## Divergences

- MatchSet `len` is a field, not a method (compiler caught; fixed in-file).
- lane_eq_mask body is at :563-574, not :560-563 exactly; recorded above.
- Corpus constants used `BASE = 0x000A_0000_0000_0000` with SlotConst sigil
  words — a court-local choice within the A11 word ABI, no spec conflict.
