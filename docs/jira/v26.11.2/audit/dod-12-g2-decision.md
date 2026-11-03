# dod-12 — G2-PARETO R4 operator decision package
# (the honest-fail decision package)

Lane DOD-12, 2026-11-02. Repo /Users/sac/cgap @ main. Sole write: this
file. Read fresh at use time: lanes/lane-gap-07-g2-pareto.md (measured
table + flip transcripts), 01-release-gates.md G2 block,
audit/tps-01-reality-ledger.md (fresh penstock 7.35-8.13 ns/frame,
ingress 10.70-11.80 ns/frame), audit/cull-01-gate-overlap-matrix.md
(read; no G2-bearing rows — its one MERGE, gate3_cdata ->
gate3_schema, is not a G2 level; noted here only to record the read).
Pattern of record for gate changes: the MV-5 decided-budget re-scope
(lanes/lane-mv-05-sla-reconcile.md: a DECIDED_* const set on the
measured distribution, an ASPIRATIONAL_* const retained and printed
loudly as a shortfall every run, plus an append-only gate-change
receipt).

## 0. The law this package operates under

01-release-gates.md G2 BLOCKED block:
- Bench target missing -> exit 2, `[SKIP] <level> bench-absent`.
- Threshold met without a flip witness -> receipt line
  `VACUOUS: L<n> threshold met with no flip court`, exit 3.
GAP-07 measured rollup (five levels):
- L1: MET (11.04 ns/frame fastest @1M — QUIET-HOST scope per
  `redteam/red-10-timing-jitter-exploit.md`; R-15 re-witnessed quiet
  legs 10.26-10.65 ns/f, redteam/fix-18 W2) — VACUOUS per gate law
  (8 named-class flip attempts, none flipped; transcripted).
- L2: NOT MET (23.13 ms @1M vs <10 ms; 2.31x over) — chain_ram
  lane figure; consolidated row 9 = 42.58 ms governs; 23.13 vs
  42.58 same-surface 1.84x divergence registered, unreconciled —
  UNVERIFIED-FOR-PAPER (`11-measured-claims.md` row 9;
  `../dflss/02-voc-ctq-matrix.md:252-265`).
- L3: CLOSED by repair-06 (78.3x @1M, <75 ms MET) — out of scope.
- L4: PARTIAL — MET @256 (0.786 us/tx, vacuous-per-law),
  NOT MET @4096 (9.35 us/tx).
- L5: NOT MET (3.055 us/row vs <1.45; flip witnessed +8.3%).
The law does not let a NOT MET be rounded up, and it does not let a
VACUOUS met count as measured. Both states are honest-fail states that
force exactly one of three decisions per level: repair, gate-change,
or retire. That is this package.

## 1. The three lawful options, stated once

(a) REPAIR-TO-THRESHOLD — kernel work that actually moves the measured
    number under the existing gate. Lawful when the gap is plausibly
    closeable and the hot path is correctly attributed.
(b) GATE-CHANGE RECEIPT — MV-5 pattern: DECIDED_* set on the measured
    distribution, ASPIRATIONAL_* retained loud, receipt written. Lawful
    when the original target is aspirational-class or the gap is
    structural on this host. Never lawful when it launders a fixable
    gap or is set on a confounded distribution.
(c) RETIRE-THE-LEVEL — delete the level (or an arm), naming the
    subsumer that inherits the invariant. Lawful only when another
    measured subject already owns the invariant.

## 2. Per-level package

### L1 — ingress frames, <15 ns/frame

Measured: ingress_chain @1M fastest 11.04 ns/f / median 11.76 (GAP-07;
QUIET-HOST scope — under combined bw+cpu contention the slowest column
crosses 124.06 ns/f, red-10 row 28; R-15 re-witnessed the quiet legs at
10.26-10.65 ns/f fastest, redteam/fix-18 W2, all guards green);
fresh independent run tps-01: penstock admission 7.35 ns/f fastest /
8.13 median, full chain 10.70 fastest / 11.80 median. Verdict on disk:
MET but VACUOUS — 8 attempts, all <15 ns/f (best mutant 12.44 ns/f),
~30% headroom never breached.
- (a) Repair: the number needs no repair (two independent distributions
  sit under the gate); the repairable defect is the VACUITY. Fix =
  section-3 semantics court. Est. scope: bench-local, ~30 lines in
  chains-ingress.rs + one golden-digest check; no kernel edit; <1 hr.
- (b) Gate change: NOT INDICATED. Re-scoping a gate that two
  independent fresh distributions meet comfortably is gate-laundering.
- (c) Retire: NO SUBSUMER — L1 is the only per-frame ingress cost
  measurement, and only its stage2 leg runs through the kernel
  (dam::ingress); retiring it orphans the only kernel-attached leg.
  The stage1/stage3 mirror problem (GAP-07 finding b) is an
  attribution defect to be diagnosed, not a reason to delete the
  level (the section-3 court diagnoses it).
RECOMMENDATION: (a) repair the vacuity with the section-3 semantics
court; the 15 ns gate STANDS unchanged. Rationale: the number is real,
healthy, and reproduced across two independent runs on two days; the
only defect is that the flip class (perf-degradation black_box) is
wrong for a memory-latency-bound pipeline with 30% headroom — 8
attempts proved the class, not the gate, wrong. Cheapest lawful fix is
a new mutation class, not a new gate.

### L2 — 1M-row parquet scan, <10 ms @1M

Measured: chain_ram @1M fastest 23.13 ms / median 23.32 (2.31x over);
@65,536 1.426 ms. Sandbox == shared tree within noise. QUALIFIER:
chain_ram lane figure; consolidated row 9 = 42.58 ms governs; 23.13
vs 42.58 same-surface 1.84x divergence registered, unreconciled —
UNVERIFIED-FOR-PAPER (`11-measured-claims.md` row 9).
- (a) Repair: a 2.31x gap on a zero-alloc scan kernel that shows NO
  movement under two per-byte black_box mutants (23.19 / 23.23 ms vs
  23.13 baseline — itself UNVERIFIED-FOR-PAPER vs row 9's 42.58 ms
  governing figure) says the kernel is at its shape limit, not its
  tuning limit. Closing 13 ms needs a new kernel shape (different
  blocking / SIMD layout), not tuning. Additionally the board's hot-fn
  attribution has DRIFTED: fnv1a64_seeded is off the chain_ram hot
  path post-P2-PARETO (fused SealDigests::absorb owns seal FNV), so
  repair planning would target the wrong function. Est. scope: LARGE,
  unbounded — new shape + attribution re-derivation.
- (b) Gate change (MV-5): DECIDED_BUDGET_MS = 25.0 (covers fastest
  23.13 [lane figure, UNVERIFIED-FOR-PAPER; row 9 = 42.58 governs] +
  noise margin, min-of-N law like MV-5), ASPIRATIONAL_MS =
  10.0 retained loudly (2.3x shortfall printed every run). Receipt
  cites the GAP-07 distribution + MV-5 precedent. Est. scope: one
  const pair + receipt; small.
- (c) Retire: NO SUBSUMER. L2 is the only absolute 1M-row scan cost
  measurement. L3's closed 78.3x is a RELATIVE ratio (radix vs
  scalar, same run) and does not own absolute scan cost. Cull-01
  overlap matrix has no row touching L2 either.
RECOMMENDATION: (b) MV-5 gate-change receipt — DECIDED 25 ms,
ASPIRATIONAL 10 ms loud. Rationale: two independent named-class
mutations cannot move the kernel AT ALL (identical numbers under
mutants), which is direct measured evidence the current shape is at
its limit on this host; the 2.31x gap is structural, and the hot-path
attribution drift means repair scope is unbounded. The aspirational 10
ms stays printed loud every run so the 2.3x shortfall is never hidden.
If a future shape change lands, the re-open trigger moves the gate
back toward the aspirational number.

### L4 — stage_commit_parquet, <4 us/tx

Measured: @256 fastest 201.3 us = 0.786 us/tx, MET but
vacuous-per-law; @4096 fastest 38.31 ms = 9.35 us/tx, NOT MET (2.34x
over). CRITICAL CAVEAT (GAP-07): A3 measured 2.472 ms @4096 on
2026-11-01; today's 38.31 ms is 15.5x that, fastest-vs-fastest, under
ambient 16-core load (load avg 14.46 disclosed). The @4096 FAIL is
load-confounded — the distribution is not stable enough to act on.
- (a) Repair: NOT INDICATED YET. Any repair plan scoped off the 9.35
  us/tx number would be scoped off a confounded distribution (15.5x
  fastest-vs-fastest divergence vs A3 at the same const on an adjacent
  day). A lawful repair plan cannot be grounded in this number.
- (b) Gate change: NOT INDICATED YET, same ground — an MV-5 decided
  budget set on a load-confounded run bakes ambient load into the gate
  permanently. MV-5 itself was set on N=7 min/median/max distributions.
- (c) Retire: NO SUBSUMER. The per-tx commit-cost invariant (ring
  memcpy + footer small-block seal) is measured only here.
RECOMMENDATION: re-measure first, then decide. Order of operations:
(1) quiet-host re-measure of stage_commit_parquet @4096, min-of-7,
ambient load disclosed per 06-bench-protocol §3; (2) if the quiet
distribution lands near A3's 2.472 ms (0.604 us/tx), the <4 us/tx
gate STANDS and the FAIL retracts as a load artifact; (3) if it lands
near 9.35 us/tx quiet-host, apply MV-5: DECIDED 10.0 us/tx @4096,
ASPIRATIONAL 4.0 loud, receipt citing both runs. Separately, the
@256 MET arm's vacuity is repaired with the section-3 court (~10
lines). No gate may be decided on the confounded distribution.

### L5 — governance chain, <1.45 us/row

Measured: governance_chain_fused_per_row @65,536 fastest 200.2 ms =
3.055 us/row (2.1x over; R-15 re-witnessed the L5-family on 2026-11-02
at 3.086 fastest / 3.123 median us/row, +1.5% fastest / +1.3% median —
redteam/fix-18 W5; no contention guard exists on chains-seal, disclosed
there); control governance_chain_per_row = 155.2 ms
= 2.368 us/row. Fused is a 29% REGRESSION vs its own control on this
host (corroborates W3-L5). Flip WITNESSED: injected redundant per-row
crypto in fused_row (asc450.rs:968) moved the number +8.3% (216.0 vs
199.4 ms @65,536; +8.3% @1024; baseline spread <1%) — anti-vacuity is
ALIVE on this level.
- (a) Repair: two sub-repairs. (i) The fused arm is a failed
  experiment (29% regression); "repairing" fusion is unjustified
  ledger work on a documented dead end. (ii) Even the control fails
  (2.368 vs 1.45, 1.63x over); closing that needs new per-row crypto
  work of unbounded scope, and the flip transcripts give no evidence
  a bounded path exists. Est. scope to threshold: unbounded.
- (b) Gate change (MV-5): gate the CONTROL arm at
  DECIDED_BUDGET_US_PER_ROW = 3.2 (covers control fastest 2.368 and
  the fused 3.055 with margin, min-of-N), ASPIRATIONAL 1.45 loud
  (2.1x shortfall printed every run). The distribution is stable
  (155-202 ms across two legs, two consts, <1% spread) — this is a
  lawful MV-5 substrate, unlike L4 @4096.
- (c) Retire: PARTIAL SUBSUMER. The control arm already measures the
  honest per-row governance cost, so the FUSED ARM can be retired with
  the control as subsumer — but only after the 29% regression is
  receipted, because the fused arm is the only standing witness of
  that regression. The level itself cannot be retired: nothing else
  owns per-row governance cost.
RECOMMENDATION: (b)+(c) combined — (1) receipt the 29% fused-vs-
control regression as a measured finding (cite the GAP-07 flip table),
(2) retire the fused arm with governance_chain_per_row as named
subsumer, (3) gate the control via MV-5: DECIDED 3.2 us/row,
ASPIRATIONAL 1.45 loud. The asc450 +8.3% flip witness is recorded as
the level's standing anti-vacuity witness; if the arm change strands
it, re-seat it with the section-3 court on the control leg. Re-open
trigger: any change to asc450.rs fused_row, or any new fusion
proposal, re-runs both arms before the gate is valid.

## 3. L1/L4-met-arm vacuity fix design — the semantics-flip class

GAP-07 finding: 8 black_box-class attempts could not flip L1 because
the pipeline is memory-latency-bound with ~30% headroom; per-byte
barriers are ~free on a 16-wide machine. The fix changes the MUTATION
CLASS, not the gate.

Class name: SEMANTICS-FLIP (register as G2-M2; supersedes G2-M1 for
met-with-headroom arms where a perf-degradation mutant provably cannot
breach the threshold).

Premise: a met gate with large headroom is proven measured not by
degrading the number past the threshold, but by proving the measured
number is tied to the measured WORK. Do that with an output-visible
mutation: mutate so the pipeline OUTPUT changes, and require the
bench's own output check to flip to FAIL. A surviving mutant = the
gate does not observe the work = vacuous, loud. This is the repo's
already-proven court shape — 09-overnight-mutation-set.md G8-M1
("Remove one of the 11 mappings -> census exits nonzero AND golden
digest breaks") and L5-M1b ("Demote a crossing -> proven == N/2 assert
flips") are output-visible mutations with digest/assert flip
requirements, witnessed on disk.

Design (bench-local, no kernel edit):
1. Golden digest: fold the chain_frame outputs of all lanes into a
   digest printed on every receipt line. Const inputs -> deterministic
   digest. Golden value captured from the unmutated tree on the first
   court run.
2. Mutant G2-M2a (kernel leg): in the stage2 leg (dam::ingress — the
   only leg measured through the kernel), drop one lane's contribution
   to the fold. REQUIRED: digest mismatch -> court flips, exit 1,
   sha256-exact restore.
3. Mutant G2-M2b (diagnostic, mirrors): drop one lane's contribution
   in the stage1/stage3 bench-local mirrors. If the digest DOES NOT
   change, that is a SURVIVOR and it is a FINDING, not a court
   failure: it witnesses GAP-07 finding (b) — the mirror legs do not
   contribute to the measured output and are measurement theater.
   Receipt line loud:
   `G2-M2b SURVIVOR: mirror legs do not contribute to output`.
   Same shape as cull-01's zdf self-check (a surviving mutant is the
   diagnostic). If it flips, the mirrors contribute and the
   attribution gap is bounded to stage2-only.
4. Clean run exit 0 with digest printed; [SKIP] law unchanged for
   perf-class mutants on NOT-MET levels (that class worked on L5).
Cost: ~30 lines in chains-ingress.rs, two rebuilds, two bench runs,
two sha256-exact restores. Est. <1 hr lane. L4 @256 arm variant: same
court on stage_commit_parquet outputs, ~10 lines.

## 4. Rollup of the recommended path

| Level | Path | Work | Scope |
|---|---|---|---|
| L1 | (a) semantics court; 15 ns gate stands | sec-3 digest court | ~30 lines, <1 hr |
| L2 | (b) MV-5 receipt: decided 25 ms / aspirational 10 ms loud | none (const pair) | small |
| L4 | quiet-host re-measure @4096, then MV-5 or retract | none yet | 1 run + receipt |
| L4 @256 arm | (a) semantics court (sec 3 variant) | bench-local digest court | ~10 lines |
| L5 | (b)+(c): receipt 29% regression, retire fused arm, | none (receipt + | small |
|     | MV-5 control 3.2 / aspirational 1.45 loud | arm retire + consts) | |
| L3 | stands CLOSED (repair-06) | none | none |

Dependency order: L4 quiet-host re-measure is machine-state-dependent
and should be laned first on a quiet window; L1/L4-arm courts are
bench-local and independent; L2/L5 const pairs are independent
one-file edits; all five receipts append to the G2 receipt set.

## 5. Pre-drafted operator receipt

The operator signs or amends. Sign = run the rollup as laned work.
Amend = strike rows, keep the rest. Absence of a response inside the
lane window is NOT a sign.

    OPERATOR RECEIPT — G2-PARETO R4 close (dod-12)

    Decision basis: lane-gap-07-g2-pareto.md measured distributions
    (2026-11-02 @ 9a730320), tps-01 fresh ingress numbers (7.35-8.13
    penstock / 10.70-11.80 ns/frame), MV-5 decided-budget precedent
    (lanes/lane-mv-05-sla-reconcile.md), cull-01 overlap matrix
    (no subsumers found for any G2 level).

    L1: threshold 15 ns/frame STANDS, NOT re-scoped — two independent
    distributions meet it (GAP-07 11.04 ns/f; tps-01 10.70-11.80),
    quiet-host scope (red-10 row 28 registers the contended divergence;
    R-15 re-witnessed the quiet legs 10.26-10.65 ns/f, fix-18 W2).
    Vacuity repaired via the G2-M2 SEMANTICS-FLIP court (dod-12 §3):
    golden digest over chain_frame lane folds, stage2 lane-drop mutant
    must flip, mirror-leg survivor = loud attribution finding.
    Bench-local, ~30 lines, no kernel edit.
    L2: gate re-scoped per MV-5: DECIDED_BUDGET_MS = 25.0 on the
    GAP-07 distribution (fastest 23.13 / median 23.32 — lane figure,
    UNVERIFIED-FOR-PAPER vs row 9's governing 42.58 ms), ASPIRATIONAL
    10 ms retained loudly (2.3x shortfall printed every run). No
    kernel work ordered: two named-class mutants moved the number by
    zero, the shape is at its limit on this host, and the board's
    hot-fn attribution (fnv1a64_seeded) has drifted off the hot path.
    Re-open trigger: any chain_ram kernel shape change re-derives the
    gate against the aspirational number.
    L4: @256 MET (0.786 us/tx) stands; vacuity court per §3 variant.
    @4096: today's 9.35 us/tx FAIL is LOAD-CONFOUNDED (A3 measured
    2.472 ms at the same const the adjacent day; 15.5x
    fastest-vs-fastest; load avg 14.46 disclosed). QUIET-HOST
    RE-MEASURE ORDERED (min-of-7, ambient load disclosed) before any
    gate decision: near 0.604 us/tx -> gate stands, FAIL retracts;
    near 9.35 us/tx -> MV-5 (decided 10.0, aspirational 4.0 loud).
    No gate is decided on a confounded distribution.
    L5: the 29% fused-vs-control regression is RECEIPTED as a measured
    finding (GAP-07 flip table; corroborates W3-L5; R-15 re-witnessed
    3.086/3.123 us/row within +1.5%, fix-18 W5). The fused arm is
    RETIRED with governance_chain_per_row as named subsumer. The
    control is gated via MV-5: DECIDED 3.2 us/row, ASPIRATIONAL
    1.45 us/row retained loudly (2.1x shortfall printed every run).
    The +8.3% asc450.rs:968 flip witness stands as the level's
    anti-vacuity witness. Re-open trigger: any asc450.rs fused_row
    change or new fusion proposal re-runs both arms.
    L3: stands CLOSED by repair-06 (78.3x @1M, <75 ms MET), cited,
    not re-run.

    Net effect: G2 exits the honest-fail state with every level either
    measured-and-passing (L1 and L4 @256 post-court), decided-with-
    loud-aspirational (L2, L5 control), or pending-one-quiet-run
    (L4 @4096). Nothing rounded up. No silent gate changes; every
    gate change carries an MV-5-pattern receipt; every met gate
    carries a flip witness or a semantics court.

## See Also

lanes/lane-gap-07-g2-pareto.md · 01-release-gates.md (G2 block) ·
audit/tps-01-reality-ledger.md · lanes/lane-mv-05-sla-reconcile.md ·
09-overnight-mutation-set.md (G2-M1, G8-M1, L5-M1b) ·
audit/cull-01-gate-overlap-matrix.md
