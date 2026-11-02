# W5-ASM — post-D5 bn3 census + storage closure/dred census (lane D6)

Lane D6 (design/verify). Date 2026-11-02. Read-only except this receipt and the
dated addendum appended to `p2-asm-0blr-census.md`.

## Ladder

| Gate | Command (cwd /Users/sac/cgap) | Exit | Evidence |
|---|---|---|---|
| D5 precheck x1 | grep FixpointHasher crates/cgap-bn3/src/ | 1 (no match) | check 1: NOT landed |
| D5 precheck x2 | grep FixpointHasher crates/cgap-bn3/src/unify.rs | 0 | landed: unify.rs:105 struct, :109 seed 0xBF58_476D_1CE4_E5B9, :1379 BuildHasherDefault use |
| storage asm | CARGO_TARGET_DIR=/tmp/d6-target cargo rustc --release -p cgap-storage -- --emit asm | 0 | cgap_storage-981e9d39ae5e0460.s |
| bn3 asm | CARGO_TARGET_DIR=/tmp/d6-target cargo rustc --release -p cgap-bn3 -- --emit asm | 0 | cgap_bn3-72306c3797b03464.s (rebuilt post-fix, fresh compile) |
| census | python symbol-bounded region census (method of record w3-t2-asm-census.md) | 0 | tables below |

## Counts (blr / br xN / brk per named region)

### bn3 (post-D5-fix, fresh compile)

- `unify::unify_to_fixpoint_opt` (1143-line region): **blr=0 br_xN=0 brk=0** bl=45.
  The p2 CRITICAL blr (was .s line 10230, LBB68_43) is GONE — D5's static-seeded
  FixpointHasher flipped the cell to clean.
- `unify::unify_to_fixpoint` (6-line thunk): 0/0/0.
- `unify::emit_join_masks` 0/0/0 (bl=15); `unify::bucket_candidates` 0/0/0 (bl=6);
  `unify::unify_batch_probe` 0/0/0 (bl=115) — unchanged from p2.

### bn3 whole-file sweep (honesty remainder)

- Whole-file blr=2, @TLVPPAGE=2 lines, all in one cluster:
  `unify::RuleCompiler::compile` region (asm .s lines 3990-3994): std
  `RandomState::new` TLS once-init `blr x8` — SAME provenance class as the p2
  CRITICAL (std once-init, not a cgap dispatch site). Source: `HashMap::new()` at
  unify.rs:1672 (inside RuleCompiler::compile flow, binding map) — also
  unify.rs:1979. NOT censused in p2 (p2 covered 5 unify regions; compile was not
  a named region). New in-region remainder, typed, not waived.
- `hashbrown::RawTableInner::rehash_in_place` `blr x21` — std/hashbrown internal,
  outside any named cgap region.
- Remaining RandomState monomorphizations (22 lines) are direct `bl` call targets
  (BuildHasher::hash_one glue), no indirect dispatch.

### storage (adjacent MISSING cell, same census pass)

- `closure::semi_naive_closure` (624-line region): **blr=1** — std RandomState
  TLS once-init (identical @TLVPPAGE cluster), provenance
  `closure.rs:39 std::collections::HashSet::new()`. DIVERGES from p2, which
  recorded semi_naive_closure blr=0 — the toolchain/compiler instance now inlines
  the RandomState once-init here. Typed remainder: in-region std once-init, not
  a cgap dispatch site.
- `closure::naive_closure` (445 lines): 0/0/0 (bl=28).
- dred retract family: `dred::retract` (114 lines), `dred::retract_inverted`
  (91), `dred::retract_unchecked` (91), `dred::oracle_surviving` (119),
  `dred::compact_lanes` (46): **all 0/0/0**. The p2 MISSING anti-join cell
  remains MISSING: `grep -rli anti_join crates/cgap-storage/src` = 0 hits
  (exit 1) — no anti-join kernel exists in cgap-storage; loudly missing, not a skip.
- Adjacent (not named regions, listed for honesty, same blr class):
  `closure::SeenSet::new` blr=1, `SeenSet::default` blr=1,
  `closure::mutant_base_only_count` blr=1 — all the same std RandomState TLS
  once-init from closure.rs:39.

## Verdict

- bn3 unify hot path: 7/7 named regions 0 blr / 0 br xN / 0 brk post-D5 — the
  p2 CRITICAL is closed. FLIP evidence: p2 measured blr=1 at unify_to_fixpoint_opt
  (pre-fix tree); this lane measured blr=0 on a fresh post-fix compile.
- Typed remainder (new, for coordinator): bn3 `RuleCompiler::compile` carries the
  same std RandomState TLS once-init blr (unify.rs:1672/1979 `HashMap::new()`);
  storage closure family carries 4 of the same class (closure.rs:39
  HashSet::new()). Same provenance as p2's CRITICAL: std once-init, not cgap
  dispatch. If the 0-blr law extends to these regions, the D5-style
  static-seeded hasher should be applied at those sites.
- storage anti-join: still MISSING (typed, grep exit 1).

Standing: ALIVE (all counts from asm emitted this session, exits recorded).
