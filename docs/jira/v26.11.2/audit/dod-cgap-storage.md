# DoD audit — cgap-storage (EXP-05, 2026-11-02)

Auditor: EXP-05 read-only lane. Repo /Users/sac/cgap, branch main.
Receipt of record: docs/jira/v26.11.2/audit/dod-cgap-storage.md (this file).
No writes other than this receipt. Note: /tmp/exp05-target could NOT be
deleted — `rm -rf /tmp/exp05-target` was denied by the permission system
twice (sandboxed and unsandboxed). Cleanup owed by operator.

## T1 — Retraction throughput: 248M (B05) vs "800M+" (verdict: 800M gate = ASPIRATIONAL-UNVERIFIED)

Correcting the task premise first: there is NO "800M+ mask-only" comment
at dred.rs:73-76. The actual comment (crates/cgap-storage/src/dred.rs:72-77)
is a DREDP probe note reporting a MEASURED -11% for a SWAR rewrite of the
keep-mask rescue loop: "839M vs 944M facts/s, all legs, same bench
method", with the per-lane loop KEPT on the perf fence (repair-16).
"800M+" exists only as the C6 GATE constant (aspirational threshold),
never as a measured mask-only number anywhere in the tree.

The two numbers, precisely:
- 248,062,015.5 facts/s (min of 7, release) — B05's full
  `dred::cascade_retract` call over 100k facts: BFS lineage cascade to
  fixpoint + keep-mask + branchless compaction, CSR build excluded
  (cscb_c6_retraction.rs:66-104, lane-b05-cscb-c6.md:39-46). Witnessed
  exit 1: typed C6 GATE FAIL (lane-b05-cscb-c6.md:49-53, 58-72).
- 839-944M facts/s — dred.rs's own prior probe of the keep-mask path
  ALONE (suspect-mask + alt-rescue + compaction, no BFS, no cascade),
  different corpus/method (dred.rs:73-77).

Why not comparable (DREDLABEL law — same number, wrong unit/basis): the
839-944M figure times ONLY the word-parallel mask arithmetic; the C6
number times the whole cascade, which is dominated by the BFS worklist
walk over CSR adjacency (dred.rs:378-397) plus per-root seeding
(dred.rs:352-373). GATE_FPS=800,000,000.0
(cscb_c6_retraction.rs:24) is a target, not a measurement; the best
observed min is 248M, 3.2x below the gate (lane-b05-cscb-c6.md:47-53).

VERDICT: the 800M facts/s claim is ASPIRATIONAL-UNVERIFIED. Nothing in
the tree measures 800M facts/s on the full cascade. The mask-only
839-944M probe is real but is a different surface and must not be
conflated with C6 throughput — the lane receipt itself flags this
(lane-b05-cscb-c6.md:54-58). Standing is a typed FAIL, honestly
recorded, not tuned to the number.

## T2 — Anti-phantom: 0 orphans proven (verdict: PROVEN, courts alive)

Court sources (all PASS this audit, see T3):
- B05 binary (crates/csct/src/bin/cscb_c6_retraction.rs:105-166): full
  recompute oracle `dred::cascade_oracle` (rounds-based least fixpoint,
  dred.rs:446) set-equality vs the fast path
  (assert_eq out[..n] == oracle), plus per-survivor binary-search
  membership in the oracle keep set; prints
  "0 phantoms ... 0 orphan derivations". Anti-vacuity: CSCB_MUTATE=
  skip-sweep injects 10 phantoms and MUST flip to exit 1 (witnessed
  flip in lane-b05-cscb-c6.md:74-82).
- tests/dred_cascade.rs (5/5): court_chain_root_delete_no_phantoms
  (:69), court_diamond_alt_rescue (:113), court_cycle_alt_rescue
  (:131), court_oracle_equivalence (:151), court_typed_refusals (:191).
- tests/dred_swar_wire.rs (3/3): court_rescue_matches_oracle (:72),
  court_rescue_term_is_load_bearing (:90), court_mutants_diverge (:110)
  — the alt-rescue term is proven load-bearing by mutant divergence.
- dred.rs in-module courts (#[cfg(test)]): court_retraction_matches_or
  acle (:529), court_antivacuity_alt_support_flip (:554),
  court_antivacuity_inverted_bitmask (:585), court_entropy_flat_varian
  ce (:626).

## T3 — Memory: SRAM residency bounds + window handling (verdict: BOUNDED, no mmap in this crate)

There is NO mmap/mmap_writer anywhere in cgap-storage (grep over src/ =
zero hits; crate deps are cgap-blake3 runtime + cgap-git dev-only,
crates/cgap-storage/Cargo.toml:11-22). Fixed bounds in code:
- bitplane_seen.rs: BitPlaneSeenSet<const SLOTS> is a FIXED inline
  footprint of PLANE_COUNT * SLOTS * 8 bytes = 128 * SLOTS * 8 (consts
  KEY_BITS=128 :23, PLANE_COUNT=128 :26); capacity is typed-loud
  (SeenFull, :28-33), never silent drop. The court leases 16,384 slots
  = 16 MiB ON THE HEAP via new_boxed()/alloc_zeroed, never stack-staged
  (bitplane_seen.rs:49-59, court_bitplane_seen.rs:22-24). No growth,
  no window: full residency, hard refusal at capacity.
- closure.rs: seen-set is a std HashSet<u64> PRE-SIZED once from an
  upper bound on pair count (with_capacity, closure.rs:57-66, repair-11
  discipline) — bounds allocation COUNT (<1000 allocs = H8 falsifier),
  not bytes; N^2 bitplane rejected as infeasible at N=1M (4 TiB,
  closure.rs:8-11). Edge/pair columns are caller-leased Vecs; no mmap
  window, everything resident.
- dred.rs: all masks/worklists/CSR are CALLER-LEASED fixed slices with
  typed capacity refusals (CascadeError::WorklistCapacity /
  CountsLease, dred.rs:333-350); the timed path allocates nothing
  (cscb_c6_retraction.rs:60-66).
- atomic_commit.rs: WAL-framed; no mmap (zero grep hits).

Test run (CARGO_TARGET_DIR=/tmp/exp05-target, cargo test -p
cgap-storage --release, exit 0). Whole-run plus targeted re-run counts:
- dred_cascade 5/5 PASS (0.00s, expected)
- dred_swar_wire 3/3 PASS (0.00s, expected)
- court_bitplane_seen 3/3 PASS — 48.73s warm release on this box;
  the ~215s figure from the task brief did NOT reproduce (warm target,
  release profile). Debug/cold builds are the plausible source of 215s.
- closure_court 3/3, closure_alloc_court 3/3, court_crc_identity 3/3,
  court_flash 3/3, court_faults 5/5, court_mutations 6/6, court_sweep
  2/2, unit tests ok, doctests 0. ZERO failures across all suites.

## T4 — Packaging readiness per .claude/rules/publish.md

(verdict: READY per rules, but the GLOBAL PUBLISH FREEZE applies)

- publish = true already flipped, receipt cited in-tree:
  crates/cgap-storage/Cargo.toml:7 ("P3-2b guard flip (2026-11-01)...
  A05 receipt ALIVE"). This satisfies the "receipt-backed court run"
  precondition; the receipt lives at
  docs/jira/v26.11.1/waves/a05-finish-receipt.md.
- House style conforms: one-line capability description (Cargo.toml:6),
  license-file = ../../LICENSE.md (:7), explicit publish flag (:7),
  workspace version/edition/repository inheritance (:2-4).
- BUT publish.md declares publishes FROZEN since 2026-11-01
  (publish.md:21-27; waves/p4-terminus.md, waves/publish-halt.md), with
  a mechanical guard (.claude/hooks/publish-guard.sh). Lift sequence at
  waves/publish-halt.md:31-42 does NOT list cgap-storage republish as
  owed — cgap-storage is not named in the repair list, i.e. it is not
  a known-debtor; any attempt still refuses mechanically until the
  operator lifts the freeze.
- Zero external runtime deps (E=0), matching the substrate law.
- Fences owed before any republish: the T3 court run above (fresh
  exit-0 receipt, this file) and csct's cscb_c6 standing (typed FAIL
  on throughput — perf gap is disclosed, not a packaging blocker).

## Exit lines (this session)

- CARGO_TARGET_DIR=/tmp/exp05-target cargo test -p cgap-storage
  --release — exit 0 (all suites, zero failures)
- targeted re-run of 6 suites — exit 0 (counts in T3)
- rm -rf /tmp/exp05-target — DENIED by permission system (twice);
  /tmp/exp05-target left on disk, cleanup owed by operator
