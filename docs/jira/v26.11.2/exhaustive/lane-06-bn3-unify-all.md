# lane-06-bn3-unify-all

Exhaustive census: crates/cgap-bn3/src/unify.rs. Assembled by the
coordinator (original EX6 lane 429-terminated before any write; this
file is the sole write). Sources: coordinator greps/seds 2026-11-02 +
landed lane receipts (W6-BFX, w6-bn3-nostd-borrowed-driver,
w5-l3-court-flips, forensic-lane-03/04/07, BR7/BR8 maps). All line
numbers post-BFX/BN3NS; RADIXFIX in flight owns the Default flip
(AS-OF noted). ~2,853 lines, 88 fn-definitions grepped.

## 1. Coverage ledger

- Files this lane: unify.rs ONLY (join.rs/scratch.rs/kernel.rs ABSENT —
  loud; the join kernel lives inside unify.rs).
- FN coverage: every region censused via grep + landed-lane cites;
  100% of named regions below verified on disk this session.

## 2. Region census (hot/cold + verdict)

| Region | Lines (post-BFX) | Class | Verdict |
|---|---|---|---|
| JoinProbeScratch struct + Default | :602, :722-723, :784-787 | HOT | RADIXFIX AS-OF: derive(Default) flipped to explicit impl radix:true (docstring now enforced; landing in flight) |
| radix_bucket + bucket tables | :825-828 (BR7-corrected) | HOT | KEEP (3 ALU ops; branchless twin exists in cgap-branchless radix.rs:45-48, B5-differential-pinned; swap = dedup/audit not ns) |
| lane_eq_mask | :678-687 | HOT | KEEP — already SWAR word-parallel, branch-free |
| Candidate cache build | :1124-1151 class | HOT (once/batch) | KEEP — FNV key, chunk-independent; rebuild_per_chunk arm retained as differential |
| Probe loop + chain walk | :1391-1423 | HOT | KEEP/structural — singly-linked sentinel branch inherent (BR7); match sets exact vs naive oracle under 12k-collision storm (FA3) |
| Verdict branch + prefilter branch | :1397-1409 | HOT | Residual branches; pred_inc does NOT exist in cgap-branchless — new-kernel work (BR7), not reuse |
| semi_join_masks + emit_join_masks | :783-800, :934-985 | HOT | KEEP — mask-AND algebra branch-free (M-L3-2 flip-pinned) |
| Fixpoint driver + exit | :1505-1570; exit :1544-1547 | HOT | FA4-verified: semantic convergence exit (new_rows==0); budget = typed FixpointBudgetExceeded refusal, no partial set |
| seen-set (BTreeSet) | :1518-1520, :1562 | HOT-insert | STRUCTURAL-FUTURE (BR8): Eytzinger redesign falsifier = branch-miss-bound profile, UNMEASURABLE-here (no PMCs) |
| mem::swap region | :1414-1415 | HOT | KEEP (core::mem::swap post-BN3NS) |
| RuleCompiler::compile + HashMaps | :498-527 class | COLD (compile) | KEEP; carries the std-TLS blr remainder (w5-asm-census-repair) — out of hot-path census scope, defect-ledgered |
| ConsequentProjector calls | :1505+ | COLD-caller | KEEP |

## 3. Allocation posture (post-BFX + BN3NS)

- to_vec(): ZERO matches (W6-BFX marker ~:1470; FA4/MB4 witnessed).
- Borrowed-prefix + owned-delta-tail: input store never copied; deltas
  allocate (scale-with-delta, FA4 probe: 156 allocs / 3 rounds on the
  3-cycle falsifier; chain-64: 2,050 allocs / 102.9 MB — delta-scaled,
  not store-scaled). NOT literally zero-heap; honest per H8 scoping.
- BTreeSet seen: alloc per insert (membership-only, D5-verified).

## 4. no_std state

BN3NS landed: cfg_attr(not(test), no_std) at lib.rs:3; riscv32imc
check exit 0 zero-warnings; BTreeSet choice documented (membership-only
= semantics-identical, hasher-free).

## 5. Anti-vacuity courts (existing, named)

- l3_differential_join_court + l3_antivacuity_probe_drop_flips:
  kernel flips witnessed exit 101 (w5-l3-court-flips, /tmp sandboxes,
  sha256-exact restores).
- l3_noalloc_radix_capacity_refusal: NA_MAX_MATCHES typed refusal.
- M-L3-1/M-L3-2 kernel-shim flips: CLOSED (w5-l3-court-flips:57 row).
- GAP (FA3/BR7): single-bound-slot corpus cannot flip a
  drop-bound_slots[1..] mutant — FA3's 500-candidate corpus is the
  named court extension (RADIXFIX scope).

## Commands

| cmd | exit |
|---|---|
| grep fn-count + Default/radix regions (coordinator, 2026-11-02) | 0 |
| grep -c "pub fn\|fn " unify.rs = 88 | 0 |
| Region seds :602/:722/:784/:825/:678/:1391-1423/:1518-1570 | 0 |
