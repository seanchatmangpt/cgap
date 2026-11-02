# Forensic lane FA5 — DRed 943.67M facts/s adversarial audit (v26.11.2)

Lane: FA5 (hostile forensic). Repo: /Users/sac/cgap @ main. Date: 2026-11-02.
Sole write: this file. Repo source untouched (verified via git status at end).

## Verdict summary

| # | Suspicion | Verdict | Key number |
|---|---|---|---|
| 1 | 943.67M facts/s is a static-bitmask scan, not relational retraction | CONFIRMED | retract output 9,999/10,000 chain survivors; expected 0 |
| 2 | Oracle arm is a cheaper twin inflating the 5.9x | REFUTED (oracle is honest full-recompute) — but ratio certifies kernel speed, not a DRed system | 5.90x at 90% churn is real machine time |
| 3 | Worst-case cascade collapses the 1.06 ns/lane average | CONFIRMED — collapse is not a slowdown, it is an ABSENCE: no cascade exists | 1.575 ns/lane measures the wrong thing; true chain closure is unimplemented |
| 4 | Entropy sweep is reshuffled-equivalent input, variance trivially 1.0008 | PARTIALLY REFUTED — the purged sets genuinely differ; the flat ns are real branchless data-independence | purged 0 / 54,916 / 110,238 at 0/50/100% entropy at constant ~276.5 us |

## Interrogation 1 — what does dred_churn actually measure?

Reads: `crates/bench/benches/dred_churn.rs`, `crates/cgap-storage/src/dred.rs`.

`dred_churn.rs` measures wall-clock ns of one call to
`dred::retract(derived_deps, del_mask, alt_support, facts, scratch, base, out)`
(`dred.rs:116-127`) over a splitmix64 (derived, del, alt) corpus of
2^18 = 262,144 lanes, min-of-30 rounds, final-state assert
`set == oracle_surviving(...)` at `dred_churn.rs:137` and `:177`.

- `retract` = `keep_mask` + `compact_lanes`. It READS three static masks and
  writes an index vector. It never writes `derived_deps`, `del_mask`, or
  `alt_support` — the relational state is INPUT, never mutated. The
  characterization suspicion is CONFIRMED: the measured surface is a
  single-pass mask predicate, not a retraction engine that updates derived/
  alt state.
- del_mask per batch: `dred_churn.rs:65-83` `corpus()` — splitmix64
  (multiplier 0xBF58_476D_1CE4_E5B9 inside `dred::splitmix64`, dred.rs:34-40),
  seeded per batch `0xA20C_0FFE_E5A7_E000 ^ (frac<<32) ^ (b+1)`
  (dred_churn.rs:173). Bernoulli p = churn fraction, independent per fact.
  Each batch is a FRESH corpus, not a delta over the previous batch: there is
  no increment, no churn across time, no batch-to-batch lineage update. The
  bench name says "churn"; the input is a static snapshot per batch.
- Would the oracle set-equality assert (dred_churn.rs:177) catch a no-op
  retract? YES for output: a no-op (identity) retract returns all facts while
  the oracle purges derived&deleted&no-alt, so `assert_eq!(set, oset)` fails.
  BUT the assert compares two computations over the SAME static masks — it
  proves the predicate is evaluated correctly and nothing about lineage
  propagation, because there is no propagation to prove.

## Interrogation 2 — the oracle arm: real baseline or cheaper twin?

`oracle_surviving` (dred.rs:197-213) is a genuine full-recompute scalar loop:
per-fact bit extraction, `Vec::push` per survivor, no SWAR, ZERO code shared
with `retract`/`keep_mask`/`compact_lanes`. The 5.90x at 90% churn is real
measured machine time; a cheaper twin would deflate the ratio, not inflate
it. On this suspicion the number is honest.

The inflation vector is different from the one suspected: BOTH arms scan the
same static masks, so the ratio certifies "SWAR beats scalar at computing a
pure predicate over static input" — it does not certify a retraction system.
The ratio's churn dependence (1.55 / 4.01 / 5.90 at 10/50/90%) comes from the
oracle's `Vec::push` per survivor vs the SWAR path's caller-leased flat
buffer — an allocation-profile difference, visible in the code, not a
semantic speedup of retraction.

Why facts/s is churn-independent (~943M at 10/50/90%): keep_mask's word loop
is fixed-bound, the alt_support scan is O(facts) regardless of churn, and
compact_lanes is branchless with data-independent control flow, so survivor
density changes the written bytes, not the time.

## Interrogation 3 — cascade deletion probe (N=10,000 linear chain)

Probe: /tmp/fa5-probe (cargo release, path dep on cgap-storage), driven this
session. Command + exit:

    cd /tmp/fa5-probe && CARGO_TARGET_DIR=/tmp/fa5-target cargo run --release; echo EXIT=$?
    -> EXIT=0

Output (verbatim):

    CHAIN N=10000 retract_ns=15750 ns/lane=1.5750 survivors=9999 (expected 0 if cascade)
    TRUE_CASCADE probe-side propagation: 3583 ns, retracted=10000
    HEADLINE_REPRO facts=262144 best_ns=276541 facts/s=947.94M ns/lane=1.0549
    level total_ns purged keep_ns compact_ns
    ENTROPY 0% keep=87458ns compact=189000ns purged=0 total=276458
    ENTROPY 50% keep=87500ns compact=189000ns purged=54916 total=276500
    ENTROPY 100% keep=87458ns compact=188958ns purged=110238 total=276416
    PROBE-OK

Findings:

1. Facts 1..9999 (all derived, all transitively dead once the root dies)
   SURVIVE the real `dred::retract`. survivors=9999. The pure chain admits no
   alt_support rescue (alt = 0 everywhere), so a correct retraction must
   purge all 10,000. It purges exactly 1 (the root, fact 0).
2. The collapse is not "worst-case latency per lane rises" — it is that the
   cascade SEMANTICS ARE ABSENT from the kernel surface. There is no
   derived_deps-as-dependency-graph traversal anywhere in dred.rs: the bit
   named `derived_deps` is read as a predicate bit, never followed as an
   edge. The 1.06 ns/lane average is thus measured on a workload that
   excludes the only work DRed exists to do.
3. For scale, a trivial probe-side iterative closure of the same chain took
   3,583 ns (0.36 ns/lane) — closure cost is not the obstacle; the obstacle
   is that dred::retract cannot express it. (Probe-side, single pass, not a
   cgap kernel number — directional only.)

Erratum (disclosed): the probe's own CASCADE_HONESTY println used an inverted
ternary and printed "TRANSITIVE (cascade real)" while the data line above it
shows survivors=9999 — the label is wrong, the data is authoritative: NO
cascade. The probe source at /tmp/fa5-probe/src/main.rs carries the bug; the
finding does not depend on the label.

## Interrogation 4 — entropy sweep honesty

Generator: `corpus()` at dred_churn.rs:65-83 (mirrored from
dred.rs:223-244), Bernoulli p over three independent bit classes; entropy
level e sets ONLY the del-bit probability, seeded
`CORPUS_SPREAD ^ e` (dred_churn.rs:134). Two sub-questions split:

- Does the WORK change? YES, measurably: purged counts at 2^18 are 0 /
  54,916 / 110,238 at 0 / 50 / 100% entropy (probe output above; 110,238
  ~ 262,144 * 0.6 * 0.7 as the 60% derived, 30% alt-rescued model predicts).
  The surviving sets genuinely differ per level — the input is NOT
  reshuffled-but-equivalent.
- Does the TIME change? NO, and honestly so: total ~276.4-276.5 us at all
  three levels; keep_mask ~87.5 us and compact_lanes ~189 us at every level.
  keep_mask is fixed-bound and O(facts) in the alt scan regardless of
  entropy; compact_lanes is branchless (run += set with no data-dependent
  branch), so write density changes, not ns. ENTROPY_VARIANCE 1.0008 is
  therefore NOT a trivial artifact of equivalent inputs — it is the real,
  expected signature of a branchless data-independent kernel. The suspicion's
  mechanism (equivalent inputs) is refuted; its surface observation (flat
  variance) is real for the honest reason.

Caveat carried from the bench's own doc comment (dred_churn.rs:22-31,
disclosed, not silent): the gate text's absolute "< 0.15 ns" entropy fence is
a unit error against the measured quantity; the enforced fence is the
relative max/min < 4.0 (dred.rs:386-390, court 3). Recorded, not relitigated.

## Receipt

- Commands run this session (both exit 0):
  1. `cd /tmp/fa5-probe && CARGO_TARGET_DIR=/tmp/fa5-target cargo run --release` -> exit 0, PROBE-OK
  2. probe binary: /tmp/fa5-target/release/fa5-probe
- Reads (on disk this session): dred_churn.rs (222 lines), dred.rs (392
  lines), docs/jira/v26.11.2/mega-bench/lane-05-dred-maintenance.md (70
  lines, source of the 943,670,084 and 5.90 claims).
- Standing: ALIVE for the forensic findings; the audited lane's claims stand
  as measured but their SEMANTIC SCOPE is a mask predicate, not DRed.
  Bench-level numbers reproduce: 947.94M facts/s, 1.0549 ns/lane measured
  vs 943.67M / 1.06 claimed (0.45% delta, within timer noise).
- Anti-vacuity: the chain probe is itself the mutation-in-reverse — it
  constructs an input (pure chain, alt=0) where a correct DRed must purge
  10,000 and the real kernel purges 1. A kernel that cannot fail this input
  cannot be called a retraction engine; the court that would catch it does
  not exist in cgap-storage's test surface (courts 1/2/2b/3 all use
  independent-lane Bernoulli corpora where propagation is vacuous by
  construction).
- Cleanup: /tmp/fa5-target deleted at lane end (lease law); probe source at
  /tmp/fa5-probe retained for replay.
