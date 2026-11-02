# Mega-Bench Lane 3 — L3 radix semi-join

Date: 2026-11-02. Repo: /Users/sac/cgap @ main 9a730320 (hot tree —
sibling lanes landed edits mid-session; named below). Lane contract:
READ-ONLY source; only write = this file. Build root: /tmp/mb3-target
(deleted at end of session).

## Real names (mission de-scoping recorded)

- No `semi_join_radix_probe_kernel` symbol exists. The kernel is
  cgap-bn3's `JoinProbeScratch` radix path
  (`crates/cgap-bn3/src/unify.rs:612-780`, probe loop `:1226-1310`).
- Bench legs: `crates/bench/benches/chains-bn3.rs` `stage_unify_join`
  (nested, `radix: false`, :479-513) and `stage_unify_join_radix`
  (:540-568), divan consts 65,536 / 1,048,576. Command:
  `cargo bench -p bench --bench chains-bn3 -- stage_unify_join`.
- The mission's B(y,z)/ΔR(x,y) delta tiers (1K/16K/64K) do NOT exist
  as separate consts anywhere in this bench. Recorded corpus shape
  (emitted by the bench itself per row):
  - N=65,536: parents=256, cands=256, pairs=65,536, expected_sel=256,
    expected_join=1031.
  - N=1,048,576: parents=4096, cands=256, pairs=1,048,576, expected_sel=4096,
    expected_join=16611 (single join tier; no separate delta tiers exist).
    separate delta tiers exist — not fabricated).

## Non-vacuity FIRST

`CARGO_TARGET_DIR=/tmp/mb3-target cargo test -p cgap-bn3 --test lever_courts`
- Session start (pre-repair kernel): 7/7 ok, 36.94s, exit 0.
- Post-repair kernel (unify.rs sha256 2e7d702f82a08059..., current):
  7/7 ok, 19.71s, exit 0. Includes `l3_antivacuity_probe_drop_flips`,
  `l3_noalloc_radix_differential`, `l3_noalloc_radix_capacity_refusal`.
- M-L3-1 / M-L3-2 kernel flips: NOT re-run (courts green). Witnessed
  in docs/jira/v26.11.2/waves/w5-l3-court-flips.md: M-L3-1 FLIP exit
  101; M-L3-2 FLIP exit 101; sha256 restore byte-exact
  `d662efd924e50fcfcaadbf694fbdd25d03330a4b594a464736989c723835c785`.

## Bench results (two passes, material drift BETWEEN them — recorded)

Run 1 (~10:12, pre-repair kernel): exit 0, 100 samples/leg, timer 41 ns.
| leg | 65,536 fastest | 65,536 median | 1M fastest | 1M median |
|---|---|---|---|---|
| stage_unify_join (nested) | 9.802 ms | 10.14 ms | 1.858 s | 1.879 s |
| stage_unify_join_radix | 8.128 ms | 8.376 ms | 1.811 s | 1.86 s |
radix:nested = 0.83x @65K / 0.97x @1M (fastest). Consistent with
w5-l3-remeasure.md (0.97-0.98x @1M): NOT a beat.

Run 2 (~10:45, exit 0): same command, 100 samples/leg.
| leg | 65,536 fastest | 65,536 median | 1M fastest | 1M median |
|---|---|---|---|---|
| stage_unify_join (nested) | 7.343 ms | 7.853 ms | 1.658 s | 1.739 s |
| stage_unify_join_radix | 1.475 ms | 1.486 ms | 22.99 ms | 23.65 ms |
radix:nested = 4.98x @65K / 72.1x @1M (fastest); 1M median 73.6x.
Target <75 ms @1M: MET on this run (22.99 ms fastest / 23.65 ms median).

Drift explanation: between Run 1 and Run 2 a sibling repair lane
(/tmp/radixfix-target observed running stage_unify_join) landed a
unify.rs edit in the shared tree (mtime 10:27, sha256 changed
d662efd9... -> 2e7d702f82a08059...). Run 2's bench binary (built
10:36) passed cargo's freshness check against the 10:27 source
("Finished in 0.03s", no rebuild). Numbers are kernel-exact for
unify.rs@2e7d702f. The repair lane's own court/receipt is the
authority on what changed; this lane records the drift, does not
claim the repair.

## Alloc column (zero-alloc join phase)

divan 0.1.21 emitted NO Alloc/Bytes columns on any leg in either run =
0 allocs across all samples (divan omits all-zero columns; same
method/finding as w5-l3-remeasure.md:50-53). Corroborated by courts
`l3_noalloc_radix_differential` + `l3_noalloc_radix_capacity_refusal`
(green, exit 0, this session).

## Candidate-walk attribution (w5-l3-remeasure, still standing)

w5-l3-remeasure.md attributed the dominant pre-repair residual to the
per-chunk candidate walk OUTSIDE the probe. Post-repair, the probe is
no longer the residual (72x @1M); whether the candidate walk now
dominates is UNDECIDED — no alloc column evidence locates it (probe
measured alloc-zero both eras; the walk shares the alloc-zero join
phase, so alloc columns cannot separate them).

## Compile-aborts (NOT failure counts; 0 tests executed each)

1. cgap-parquet/src/footer.rs:1008 — mid-write truncation in the
   owning lane's new `vec_meta_oracle` (`write_i32_field(&pub fn
   dummy() {}` spliced; unclosed delimiter). Compile-abort exit 1.
   Fixed by the owning lane within the ~10 min compile-freeze SLA
   (poll: compiled after 30s of a 600s bounded poll).
2. cgap-parquet/src/mmap_writer.rs:766 — `RingSink::with_dyn` not
   found (in-flight refactor by the active cgap-parquet lane, exit
   101). Not repaired by me (read-only contract). Named for the owner.

## Verdict

- L3 standing inherited: LANDED-NOT-MET (w5-l3-remeasure.md:40).
- This session: post-repair kernel measures 72.1x @1M and meets the
  <75 ms target on Run 2 — but the repair is a sibling lane's work,
  receipted (or not) elsewhere; this lane does NOT flip the standing.
  Verdict for the CURRENT kernel: LANDED, target-met on one exit-0
  run, pending the owning lane's receipt — standing UNDECIDED, not
  upgraded.
- Success audit honest form: radix vs nested = 0.97x (pre-repair,
  matches w5) and 4.98x/72.1x (post-repair, this session). Zero
  dynamic heap alloc during the join phase: confirmed via divan
  all-zero alloc columns + two green noalloc courts.
- Pre-existing, non-join failure observed: closure_ladder@65536
  panics `ArenaCapacityExceeded` (chains-bn3.rs:812) — owned by the
  W4-D8 leg, not this lane.
