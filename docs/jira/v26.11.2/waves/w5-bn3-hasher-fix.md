# W5 Lane D5 Receipt — bn3 unify.rs seen-set hasher fix

- Date: 2026-11-02
- Lane: D5 (Design)
- Ownership: `crates/cgap-bn3/src/unify.rs` (seen-set hasher only) +
  this receipt. No other source edits; no git writes.
- Branch: main (coordinator owns git transitions).

## Change

`unify_to_fixpoint_opt` (crates/cgap-bn3/src/unify.rs): the termination
`seen` HashSet previously used the default hasher (std `RandomState`,
whose TLS once-init inlines as the `blr x8` flagged CRITICAL in
`waves/p2-asm-0blr-census.md`, bn3 .s:10230, LBB68_43). Replaced with a
deterministic static-seeded hasher:

- `pub(crate) struct FixpointHasher(u64)` with manual `Default` impl
  seeding `0xBF58_476D_1CE4_E5B9` (canonical splitmix64 seed, frozen).
- `impl Hasher` — FxHash-style multiply-xor
  `(state ^ v) * 0x517C_C1B7_2722_0A95` with a `write_u64` fast path and
  a correct (non-stub) byte fallback.
- Used via `HashSet<..., BuildHasherDefault<FixpointHasher>>`.
- Do NOT touch: test-only `HashMap::new` at :1635/:1942 (untouched).

## Membership-not-order verification (pre-edit read)

Before editing, the usage region was read in full: `seen` is only ever
used via `collect()` from the initial store and `seen.insert(t)` to gate
`store_*.push`. No iteration of `seen`, no index-order dependence, no
output ordering derives from it. The doc comment already pinned this
("termination machinery only; membership, never order"). Claim verified;
not BLOCKED.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/d5-target cargo test -p cgap-bn3`
  -> exit 0. 4 lib tests passed (642.97s — pre-existing heavy fixpoint
  tests), 7 lever_courts integration tests passed, 0 failed, 0 doc
  tests. Pre-existing failures: none.
- `CARGO_TARGET_DIR=/tmp/d5-target cargo rustc -p cgap-bn3 --release -- --emit asm`
  -> exit 0; asm at
  `/tmp/d5-target/release/build/cgap-bn3/72306c3797b03464/out/cgap_bn3-72306c3797b03464.s`.

## Asm result

- `unify_to_fixpoint` tail-calls `unify_to_fixpoint_opt` (line 9614);
  region [9853..11900) of `unify_to_fixpoint_opt` contains **zero `blr`**
  and zero `RandomState` symbol references. LBB68_43 survives only as a
  plain `b.eq` target, no indirect call.
- Residual blr in the FILE (2 sites, outside the flagged region, out of
  lane scope): `RuleCompiler::compile` (default-hasher HashMaps at
  :498/:500, lines 3961/3994) and one inside hashbrown's
  `rehash_in_place` (hashbrown-internal fallback RNG, line 5680).
  Full re-census is lane D6's job.

## Standing

ALIVE (command+exit run this session against the exact subject).