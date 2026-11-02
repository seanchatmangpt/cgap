# Court-Audit Receipt: VAC-3 (chains-bn3 vacuity + value)

- **Lane**: VAC-3-bn3 (court-auditor), repo `/Users/sac/cgap`, branch main,
  tree as-found (alloc arm; P3-11 noalloc not measured).
- **Subject**: `crates/bench/benches/chains-bn3.rs` sha256
  `becee95d019c61c5aadfc737a8b080d850072ad35b2d0d5469efc819727cad40`;
  `crates/cgap-bn3/src/unify.rs` sha256
  `d40e2b8aa203175422dcc557a5c732a06a055239ae40341f88c68bd05a7b680e`.
- **Sandbox**: `/tmp/vac3-sbx/` (bench + cgap-bn3 + cgap-blake3 copies;
  manifests de-workspaced, deps repointed; all mutations /tmp-only).
  Shared tree never written; sha256 of both canonical subjects identical
  before/after (verified post-audit).

## Commands + exits (first-hand)

- `cargo bench --manifest-path crates/bench/Cargo.toml --bench chains-bn3`
  (CARGO_TARGET_DIR=/tmp/vac3-target) - **exit 0**, all asserts passed,
  100/100 samples.
- Sandbox baseline `cargo bench --bench chains-bn3 --no-run` - exit 0;
  `-- stage_scan_24b` / `-- stage_unify_join` - exit 0 (green).
- M1 skipped-row scan (`for st in stmts[1..]`) - **exit 101**, panic
  "scan closure count (skipped-row mutation flips this)". FLIPPED.
  (Two earlier mutation attempts failed to compile, E0599/E0277 - not
  flips, discarded and redone.)
- M2 dropped join body atom - **exit 101**, "join compiles to two body
  atoms". FLIPPED.
- M3 binding-alias comparator (SHIM-DOE4: `lane_eq_mask` d masked
  `& !0xFF`) in sandbox cgap-bn3 - **exit 101**, "join match count vs
  raw-word oracle". FLIPPED. Restore sha-exact.
- M4a deterministic `digest_fold` corruption (rotate 8 to 9) - exit 0,
  NO flip: the checksum assert guards determinism only, exactly as its
  own doc states. Not vacuity (primary differential = oracle counts).
- M4b stateful fold drift (atomic counter XOR) - **exit 101**, "selector
  digest checksum drifted (determinism)". FLIPPED.
- Post-restore sandbox re-run `-- stage_unify_join` - exit 0; sha256 of
  both mutated files byte-identical to pre-mutation.

## Courts run (flip evidence)

All four named mutation classes flip their asserts: skipped-row scan
(count), dropped join atom (compile shape), binding-alias comparator
(oracle differential), checksum drift (determinism). One honest scoping
note: the digest checksum assert is deterministic-corruption-immune by
construction (both passes fold identically); it kills only
nondeterminism. The oracle-count asserts carry the anti-vacuity weight
and flip under M3.

## Value audit

- **CTQ map**: scan stage 12.1-12.5 GB/s (>1 GB/s line-speed MET at
  stage level). Unify ns/pair: 7.311 ms / 65,536 pairs = 111.6 ns/pair
  (sub-us MET); at N=1M, 1.537 s / 1,048,576 pairs = 1466 ns/pair
  (sub-us NOT MET at scale). Composed chain at N=1M: 1.457 s / 25.17 MB
  = 17.3 MB/s - the end-to-end line-speed CTQ is NOT met at 1M; the
  bench honestly attributes this to the nested-loop unify.
- **Decision-relevance**: READ of `oracle_counts` confirms the oracle is
  a real hash join keyed on the (s,o) u64 word pair
  (`HashMap<(u64,u64), u64>` histogram + probe). The named lever
  (nested-loop to hash/trie join on bound (s,o)) IS the demonstrated
  shape; the oracle computes the same match count in setup time on the
  same corpus.
- **Reachability**: `unify_batch_into` + `EvalScratch` warm path is the
  production closure path (unify.rs module docs, F2-13 receipt) - the
  benched path is the shipped path.
- **PARETO reproduction**: my run - N=65,536: unify 7.311 ms of chain
  7.472 ms = 97.8%; N=1,048,576: unify 1.537 s vs chain 1.447 s (fastest)
  = ~99%+ (stage probe exceeds chain median; noise-bound, unify
  dominates). Receipt claimed 81.2% / 99.9%. The 1M share reproduces;
  the 65k share does NOT (97.8% vs 81.2%) - my chain total is also
  faster (7.472 ms vs receipt 8.385 ms). Direction identical: UNIFY is
  the ~80% segment and the hash/trie join lever stands.

## Divergences

1. Receipt PARETO 65k share 81.2% not reproduced (measured 97.8%);
   chain total 7.472 ms vs receipt 8.385 ms. Both are as-found-tree
   measurements on a dirty tree (concurrent lane edits to cgap-bn3);
   not evidence of fabrication - the 1M arm and stage absolutes
   reproduce within noise.
2. Checksum assert is determinism-scoped only (M4a), disclosed above.
3. Lease file `vac-bench-bn3-chain.md` did not exist at audit start;
   this receipt is written to that path.
4. First M1 restore attempt truncated the sandbox file to empty
   (e3b0c44...) - the known long-write corruption class, caught by
   sha256 check, restored byte-exact from canonical. Sandbox-only; the
   canonical tree was never at risk.

## Verdict

**VALUABLE.** First-hand green run (exit 0), four mutation courts all
flip (exit 101 each), sha-exact restores, oracle differential is a real
hash join, production unify path measured, PARETO attribution
reproduces in direction and at 1M in magnitude. Standing: ALIVE.
