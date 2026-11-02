# Lane DF-4: cpk capability analysis (05-cpk-analysis)

Repo /Users/sac/cgap @ main, 2026-11-02. Write scope honored:
`docs/jira/v26.11.2/dflss/05-cpk-analysis.md` (new; dflss/ dir created)
+ this receipt. No git commands. Build lease /tmp/df4-target deleted at
lane end (shutil.rmtree, below).

## W1 Subject

`crates/bench/src/bin/cpk_study.rs` (landed G09 n=30 rig, read fresh) +
the landed CTQ/receipt corpus. Re-derivation, zero source edits.

## W2 Commands + exits (all run this session)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | CARGO_TARGET_DIR=/tmp/df4-target cargo build --release -p bench | 0 | 14.83 s, 3 pre-existing manifest warnings |
| 2 | /tmp/df4-target/release/cpk_study | 1 | FAIL [CTQ-06, CTQ-04]; full table in 05-cpk-analysis.md Sec 1 |
| 3 | CPK_INJECT_FAIL=1 /tmp/df4-target/release/cpk_study | 1 | FLIPPED: Cpk 0.694 -> -9.6525 at forced LSL 10.0, injection=true |
| 4 | /tmp/df4-target sha / size check + lease delete | 0 | lease removed via shutil.rmtree |

## W3 Courts run

CPK_INJECT_FAIL anti-vacuity flip (witnessed, Sec 2 of the analysis
doc; Cpk sign change 0.694 -> -9.65). Honest caveat recorded: baseline
exit was already 1 under load, so the exit-code channel could not
discriminate today; flip witnessed through the Cpk sign change + JSON
injection flag. A court that cannot flip is FAIL; this one flipped.

## W4 Fresh Cpk table (rig JSON v2, n=30)

CTQ-05 Cpu(overall) 190.949 (mean 8.447 us, USL 35 us) PASS;
CTQ-06 Cpl(overall) 0.694 (mean 1.574 GiB/s, LSL 1.0) FAIL;
CTQ-04 Cpu(overall) 0.408 (mean 16.640 us, CV 12.26%, USL 1.15*mean)
FAIL. Exit 1 fail-closed, honest.

## W5 Key findings

1. ERRATA vs draft-era story: "optimization moved 12.4 ns to 6.00 ns"
   as the cause of the 0.855 is WRONG - 12.4/6.00 ns are the CSCB C2
   p99 ladder numbers (lane-b01-cscb-c1-c2.md:19), a different metric.
   Honest narrative: load-induced between-sample drift (within-sigma
   Cpk 2.131 vs overall 0.855) fixed by the n=30 sampling repair, not
   codegen. Landed as Sec 3 of 05-cpk-analysis.md with cites.
2. Gage R&R: BATCH=64 amortized Instant is the measurement SYSTEM for
   CTQ-02; per-frame timing cannot resolve 10 ns vs ~40 ns clock
   overhead on macOS. The gauge defines the number. Sec 4.
3. CTQ-05 capability (25 ms-class): ASPIRATIONAL-UNVERIFIED. Fresh
   const read: cscb_c3_closure.rs DECIDED_BUDGET_MS=60,000.0 (:40),
   ASPIRATIONAL_MS=25.0 (:42, not a gate), RUNS=7, corpus seed :28,
   SM64 mult :30. No 25 ms gate exists.
4. Today's run FAILs honestly under concurrent wave load (CTQ-06
   sigma 0.276 vs g09 quiet 0.0097) - reported, not rounded away.

## W6 Divergences

1. Dispatch named `dflss/05-cpk-analysis.md` "reconcile with existing
   15-ctq.md" - no dflss/ dir existed in v26.11.2 (created it) and the
   actual tree file is `15-ctq-tree.md` (not 15-ctq.md). Cross-linked.
2. "CPK_INJECT_FAIL flip - witness it if cheap": witnessed, but with
   the exit-code degeneracy caveat above (baseline already exit 1).
3. The dispatch's CTQ-05/25 ms framing: no 25 ms gate exists; the
   dispatch's own hunch confirmed by fresh const read and recorded as
   ASPIRATIONAL-UNVERIFIED.

## W7 Standing + fences

Standing: ALIVE for runs 1-4 above (command+exit this session).
Fences: this lane writes docs only - nearest fences are the receipt
lint (dx-04) and the doc no-overclaiming rules; zero source/Makefile/
manifest surface touched, so no compile or gate fence can regress.
Lease /tmp/df4-target deleted at lane end.
