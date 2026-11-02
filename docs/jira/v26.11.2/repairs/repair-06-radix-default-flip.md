# repair-06 — L3 radix default flip + honest re-measure (FA3)

Lane RADIXFIX (L3). Date 2026-11-02. Repo /Users/sac/cgap @ main 9a730320.
OWNERS touched: `crates/cgap-bn3/src/unify.rs` (JoinProbeScratch Default +
one constructor-site court), `crates/bench/benches/chains-bn3.rs`
(radix/nested leg guards + FA3 cache-counter probe), this receipt.
No git writes; no mocks; no Process.sleep.

## Change

1. `JoinProbeScratch`: `#[derive(Debug, Default)]` → `#[derive(Debug)]` +
   explicit `impl Default` with `radix: true` (single point of truth,
   matches the docstring at unify.rs ~:722). All other defaults identical
   to the derive EXCEPT `cache_key`: derive gave 0, the impl gives
   `u64::MAX` — the field's own documented invalidation sentinel
   ("u64::MAX = invalidated"). Strictly safer (a derived 0 could collide
   with a computed probe_cache_key); disclosed as a divergence from the
   "identical defaults" instruction, deliberate.
2. Bench guards (chains-bn3.rs): `stage_unify_join` asserts
   `!probe_nested.radix`; `stage_unify_join_radix` asserts `ws.probe.radix`
   (early error, never mis-measure). Nested arm stays reachable = M-L3-1
   differential.
3. FA3 cache-counter probe (radix-leg setup, untimed): one probe eval then
   `assert_eq!((misses, hits), (1, chunks-1))` — the class FA3 pinned
   (misses=1 / hits=N-chunks class), only observable on the radix arm
   (`use_cache` requires `probe.radix`). Counters reset before timing.
4. Consequential fix, disclosed: the in-src court
   `zero_alloc_court_scan_project_hot_path...` used `unify_batch_into`
   (fresh probe per call), so the radix tables allocated 45x inside the
   measured window once Default flipped. Court now leases a
   `JoinProbeScratch` (warm via its two warmup passes) and calls
   `unify_batch_probe` — the leased-form discipline the court already
   documents. debug_assert(lease defaults to radix) added.

## Probe evidence (this session)

- Default flip: `JoinProbeScratch::default()` → radix asserted true in the
  leased probe (debug_assert) and the bench guard; exit 0 on all runs.
- Cache-counter class: re-measure run exit 0 with the exact-equality
  assert (1, chunks-1) = (1, 15) @65,536 and (1, 255) @1,048,576.
- Alloc column: divan 0.1.21 emitted NO Alloc/Bytes columns on any leg =
  0 allocs steady-state (divan omits all-zero columns; same mechanism
  w5-l3-remeasure recorded). First-warm allocs are absorbed by the
  untimed counter probe (probe tables sized before the timed window),
  not inside it.

## Honest re-measure (supersedes w5-l3-remeasure)

Method: `cargo bench -p bench --bench chains-bn3 -- stage_unify_join`,
divan 0.1.21, release, 100 samples/const, timer precision 41 ns,
macOS Darwin 25.2.0. Exit 0. Output /tmp/rf-remeasure.txt (lane-scratch).

| leg | 65,536 fastest | median | 1,048,576 fastest | median |
|---|---|---|---|---|
| stage_unify_join (nested) | 8.22 ms | 8.299 ms | 1.858 s | 1.869 s |
| stage_unify_join_radix | 1.468 ms | 1.481 ms | 23.72 ms | 23.87 ms |

REAL radix:nested ratio: **5.6x @65,536** (8.22/1.468), **78.3x @1M**
(1.858 s / 23.72 ms fastest; 78.3x median). w5-l3-remeasure's 0.97-0.98x
verdict measured nested-vs-nested (derived default defect) and is
SUPERSEDED.

Verdict vs target @1M: <75 ms — **MET** (23.72 ms fastest, 3.2x under
target). Floor >20x — **MET** (78.3x). L3 lever standing: **LANDED-MET**
(previously LANDED-NOT-MET). Not rounded up: targets met with 3.2x and
3.9x headroom on the same method as the superseded receipt.

## Courts (exits this session)

- `cargo test -p cgap-bn3` (dev) → exit 0: 119 passed / 0 failed lib,
  2 passed / 2 release-ignored (bgp_hop non-ignored legs), 7 passed
  lever courts incl. l3_radix_vs_nested_differential + l3_radix_vs_nested_100k.
- `cargo test -p cgap-bn3 --release --test bgp_hop_courts -- --ignored`
  → exit 0 (2 passed): fixpoint-driver derived-set equality at scale on
  the radix default.
- Anti-vacuity flip (sandbox /tmp/radixfix-sb, shim workspace of
  cgap-blake3+cgap-bn3): BASELINE `--lib zero_alloc_court_scan_project`
  → ok, exit 0; SHIM mutate Default `radix: true`→`false` → court FAILED
  (debug_assert `probe.radix` fires), cargo exit 1 (1 failed / 0 passed);
  RESTORE sha256 `2e7d702f...5a84` byte-exact match, mtime bumped, rerun
  ok exit 0. A court that flips = non-vacuous.
- Regression guard honesty: the direct-binary divan invocation emitted no
  timing table (divan prints the table only under the cargo-bench entry);
  all recorded numbers come from `cargo bench` runs, exit 0.

## Divergences

1. cache_key default 0 → u64::MAX (above, deliberate, disclosed).
2. unify.rs working-tree diff vs HEAD is 315 lines: most of it is
   BN3NS/BFX lane work pre-existing this session; my lane edits are the
   Default impl block, the zero-alloc court lease (3 lines + comment),
   and nothing else in that file.
3. Build root /tmp/radixfix-target + /tmp/radixfix-sb-target are lane
   leases; deleted at lane end. Sandbox /tmp/radixfix-sb retains the
   sha-exact restored state.

Standing: ALIVE (every claim above carries a command + exit run this
session against the exact subject).
