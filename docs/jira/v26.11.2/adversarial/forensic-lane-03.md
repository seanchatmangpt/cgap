# FA3 — hostile forensic lane: radix semi-join (unify.rs L3 probe)

Date: 2026-11-02. Repo: /Users/sac/cgap @ main 9a730320. Lane: FA3.
Method: /tmp/fa3-probe (Counting GlobalAlloc + real `cgap_bn3::unify`
kernels, `CARGO_TARGET_DIR=/tmp/fa3-target`, release). No repo source
edits; no sleeps; scratch keys never varied (bucket family built by
scanning for `radix_bucket(y)==0x777`, seeds untouched).

## Suspicion 1 — candidate copying outside the timed section / fixture allocation
VERDICT: PARTIALLY CONFIRMED (fixture allocs exist; steady-state hot path
is alloc-zero; a claim-precision defect found).

Counting-allocator per phase (radix=true, real kernels):
- fixture setup: 31 allocs / 58,996 B; compile: 7; scratch default: 0.
- first (warmup) run: 89 allocs.
- warm run, fresh MatchSet: 10 allocs; warm run, reused MatchSet: 3;
- warm runs 0-2 of a fresh lease: 215 / 4 / 3 allocs; runs 3+: **0 allocs**
  (steady state).
Verdict on `w5-l3-remeasure.md:56-57` ("divan emitted NO Alloc columns
= 0 allocs"): **VERIFIED as a steady-state property only** — it takes
~3 warm runs on a fresh lease to reach 0; run 1 pays 215. Divan's many
samples legitimately amortize this, so the receipt's measured-zero
stands, but the invariant is "zero after three warm runs", not "warm
scratch is alloc-zero from run 1".

## Suspicion 1b — HEADLINE DEFECT: the "radix" bench leg never ran radix
CONFIRMED. `JoinProbeScratch` is `#[derive(Default)]`
(`crates/cgap-bn3/src/unify.rs:729-730`) -> `radix: false`. Every
constructor in production and bench uses `::default()` without setting
`radix = true`:
- `crates/bench/benches/chains-bn3.rs:236` (Workspace::new) —
  `stage_unify_join_radix` (:544-574) therefore executed the RETIRED
  nested-loop arm;
- `crates/cgap-bn3/src/unify.rs:1504` — `unify_to_fixpoint_opt` also
  leases a `radix: false` probe: the production fixpoint driver runs
  the nested arm;
- :1022/:1042 likewise.
`unify.rs:722-724` doc comment claims "Default is empty buffers with
`radix: true` (the L3 fast path is the default)" — **doc contradicts
the derive**. Consequence: `w5-l3-remeasure`'s 0.97-0.98x measured
nested-vs-nested and its LANDED-NOT-MET verdict (22x-over-target) is
attribution-broken: the radix path has never been timed by that leg.
Witnesses this session: first probe run (radix not set) printed
`cache_misses=0 cache_hits=0`; after `probe.radix = true`:
`cache_misses=1 cache_hits=8` — the flag demonstrably selects arms.
Fix direction (owner's call): manual `Default` impl with `radix: true`
(matching the doc), or fix the doc + set the flag at all constructors.

## Suspicion 2 — collision storm: truncation / overwrite / degradation
VERDICT: NO TRUNCATION (alloc arm); QUADRATIC DEGRADATION (real, bounded).

Bucket family: all `y` with `((y ^ y>>16) & 0x0FFF) == 0x777` — 12,000
consecutive colliding subject payloads through the real radix probe
(`probe.radix = true`), parents x candidates joined on slot 0:
| colliding candidates/parents | store | emitted | exact vs naive oracle | warm elapsed | ns/parent |
|---|---|---|---|---|---|
| 1,000 | 2,000 | 1,000 | true | 3.8 ms | 3,792 |
| 4,000 | 8,000 | 4,000 | true | 58.6 ms | 14,638 |
| 8,000 | 16,000 | 8,000 | true | 239.4 ms | 29,930 |
| 12,000 | 24,000 | 12,000 | match-for-match exact | 535.8 ms | 44,650 |
Degradation: ns/parent grows ~linearly with bucket occupancy (~3.6 ns
per candidate visit) — total O(cur x cand_in_bucket), quadratic in the
degenerate one-bucket case. The probe loop's `O(cur + cand + hits)`
comment (unify.rs:1353-1355) is per-BUCKET-CHAIN complexity, not
constant time; 12k-in-one-bucket costs 4.2M visits / 536 ms.
12,000 consecutive collisions into one bucket, cross-shaped key reuse
(2048 x 2048, one bucket): 15.7 ms, emitted == truth exactly.

Truncation in the noalloc arm: NA_MAX_MATCHES=2048
(`crates/cgap-bn3/src/noalloc.rs:35`) refuses LOUDLY via
`push_match -> Err(NaError::MatchCapacity)` (noalloc.rs:356-357) and
`NA_MAX_ROWS=8192` (:383-385) refuses the store before any table write.
No silent truncation or overwrite in either arm. Note the alloc arm has
NO match cap at all (Vec growth) — under a 1e8-match storm it grows
unbounded; the noalloc arm is the bounded one.

## Suspicion 3 — tuple comparison depth
VERDICT: DEEP (full equality), with a named court gap.

Sites: the probe loop compares EVERY bound slot, not just the bucket
key — `lane_eq_mask` ANDed over `sc.bound_slots`
(`crates/cgap-bn3/src/unify.rs:1375-1378`); the bucket function is used
ONLY as a superset prefilter (:1356-1361 comment, payload-only window so
a sigil-bit difference can never mis-bucket). Const slots and
within-atom repetition are enforced in the candidate build via
`atom_local_match_into` (:1232, branch-free accumulator over all 3
slots). So full (s,p,o) payload equality holds cumulatively; no
pointer/shallow comparison anywhere in the chain.
Real-kernel proof (no mutation needed): 500 candidates sharing ONE
subject payload with distinct objects -> emitted exactly 500, not
250,000 (a first-slot-only comparator would emit 250,000). A const-slot
near-miss (o=0x301 vs const 0x300) emits 0.
Court gap: `lever_courts.rs::court_rule` (:67-77) joins on a SINGLE
bound slot (?y). A mutation dropping `bound_slots[1..]` from the probe
comparison (keep slot 0 only) would NOT flip any lever court; it flips
my P3b corpus. The comparator-level alias mutation (M-L3-2) IS courted
(lever_courts.rs:12; a12 receipt).

## Suspicion 4 — candidate walk: does the L1-resident zero-DRAM probe claim survive?
VERDICT: CLAIM BROKEN IN TWO WAYS (and never actually exercised by w5).

(a) w5's timed leg never enabled radix (Suspicion 1b), so the
"L1-resident zero-DRAM probe" was never on the clock in
`w5-l3-remeasure`; the 0.97-0.98x figure says nothing about the radix
probe at all.
(b) With radix=true measured this session (real kernels, release): one
batch pays `cache_misses=1` -> the candidate-cache build is a FULL-STORE
scan (`for j in 0..cols.n()` with `atom_local_match_into` per row,
unify.rs:1230-1243) that sits INSIDE the timed section, once per join
atom per `unify_batch_probe` call; `probe.cache_key` is invalidated at
EVERY call entry (:1091), so the bench's timed closure (two calls per
iteration) pays TWO full-store walks per iteration — all N rows x 3
words from DRAM. L1 residency covers only `bucket_head` (4096 x 4 B =
16 KiB, unify.rs:741-748); `cand_rows/spans/binds` caches are heap
arrays scaled to store-wide match counts. The nested-loop arm
(rebuild_per_chunk) is strictly worse (per-chunk rebuilds).
Measured anchor: 33.6 ms for one 33k-row batch (parents 32,768 x cands
256, radix on) — dominated by the walk, not the probe loop. The 22x-
over-target stage time w5 recorded is therefore plausible for the TRUE
radix path too, but is UNMEASURED; the leg must be re-run with
`radix = true` before any lever verdict.

## Commands + exits (this session)
- `cd /tmp/fa3-probe && CARGO_TARGET_DIR=/tmp/fa3-target cargo run --release` -> exit 0 (three iterations: debug output above quoted verbatim; final run all asserts green)
- intermediate compile checks -> exit 0 after two format-arg fixes (E0609/E0308 in the probe only, never in repo source)
- `grep -n "JoinProbeScratch::default()" ...` -> 4 sites, all radix=false (evidence above)

## Standing
ALIVE (probe executed this session, exact subject = crates/cgap-bn3
unify radix path via real kernels). Repo source untouched; shared tree
unchanged; build root /tmp/fa3-target deleted at lane end per lease law.

## Divergences
1. unify.rs:722 doc vs derive(Default) at :729 — doc claims radix:true
   default; code default is false. Also voids w5-l3-remeasure's verdict
   attribution (its bench leg measured the nested arm). NOT fixed here
   (no-repo-edit lane); owner must choose doc-fix vs manual-Default.
2. w5-l3-remeasure "alloc-zero": refined, not broken — steady state
   (3+ warm runs) is 0 allocs; first warm run pays 215.
3. Court gap recorded (single-bound-slot court corpus), not filed
   elsewhere; P3b corpus provided above as the flip witness.
