# audit-ledger — v26.11.2 session receipts (SEED-A, 2026-11-02)

One row per landed receipt. Standing is AS THE RECEIT CLAIMS (re-read from
disk this lane; not re-derived, not re-run). Absent receipt = PENDING.
Aggregate for the pending EXP-01..10 DoD lanes; they hang per-crate
matrices off this ledger. Facts cite command+exit from the receipt body.

## Sweeper fleet (G-lanes)

| Lane | Standing | Load-bearing verified fact (command + exit) | OPEN item |
|---|---|---|---|
| G01 scan branchless | ALIVE | verify_disassembly_invariants.py --skip-binmut AFTER exit 0 (BEFORE exit 1); cargo test -p cgap-branchless exit 0 | full court arm E = exit 3 loud [SKIP] (binary mutant, pre-existing/environmental) |
| G02 transpose branchless | ALIVE | transpose_courts 7/7 exit 0; symbol witness exit 1 BEFORE -> AFTER flip | AE4 still exit 1 AFTER (transpose symbol branchy); owned by fence, not closed |
| G03 disasm gate | ALIVE | exit-code ABI pinned (CLEAN/BRANCH/SUBJECT_MISSING/LOUD_SKIP), every arm has run witness; riscv llvm-objdump bytes fix landed | (none stated) |
| G04 LSPR attributes | ALIVE | cargo test -p branchless-lsp exit 0 (72 lib + integration, 0 FAILED); positional form fail-closed typed-refused; mutants flip | (none stated) |
| G05 LSP wire rebuild | ALIVE | cargo test -p branchless-lsp --lib exit 0, 72 passed/0 failed; riscv32imc court exit 0 | code-action witness UNVERIFIED (not failed) |
| G06 did_save court | ALIVE | 90 passed / 0 failed exit 0 (72 lib + 4 did_save + 5 + 6 + 3) | (none stated) |
| G07 conformance seal | ALIVE | conformance.rs suite exit 0 (lib 72 + integration 5/6/3) | one transient disasm_court flake observed; shutdown call-site wiring is a separate lane |
| G08 defect register close | ALIVE | FA10 max-consumer riscv32imc build exit 0 + llvm-size 102,389 B (22% headroom under 131,072) | G2-PARETO rollup OPEN; FA1 wording, FA2/Cpk n=1, RV1-PERLANE OPEN |
| G09 cpk stabilization | ALIVE | exit contract pinned Cpk<1.33 -> exit 1 fail-closed; CPK_INJECT_FAIL=1 flip witnessed | charter 1.67 target kept informational; repair-13 honest exit 1 stands |
| G10 flip-tests reconcile | ALIVE | compile-abort-vs-failure census law (empty capture -> [FAIL] passed='0'); flock serialization landed | concurrent foreign census load still false-[FAIL] noted in receipt |
| G11 csct court fix | ALIVE | 6 witnesses: cargo test exit 0 (37 lib + 7 runner + 7 court, 0 failed); stage-1-empty mutation -> exit 1 guard trip | (none stated) |
| G12 csct warmer verify | ALIVE | warmer tests 2/2 exit 0; full csct surface 37+7+7 exit 0 both runs | (none stated) |
| G13 csct emotional clean | ALIVE | cargo test emotional exit 0 (3 passed); warnings census grep exit 1 (0 warnings) | scratch needed stub workspace root (disclosed) |
| G14 csct solution blake3 | ALIVE | debug timing 76 ns/frame -> typed LoudSkip runner exit 3; release passes -> exit 0 | (none stated) |
| G15 csct cli validation | ALIVE | release build exit 0; mode matrix 0/1/2/3 witnessed; 14/14 debug tests | stage 6 LoudSkip timing assertion outstanding (release-gated) |
| G16 boundary gate --deps | ALIVE | --deps run exit 0 | poison-arm flip is OPEN finding (dispatch-limited; mutation prescribed in receipt, not run) |
| G17 zero-string gate | ALIVE | clean exit 0 (22 crates, 88 artifacts); /tmp poison fixture exit 1 (fibo:FndUtlCur:USD hits) | (none stated) |
| G18 G7-PATENT gate | ALIVE | G7-PATENT PASS 4/4 DISTINCT-MECHANISM exit 0 + behavioral court exit 0; budget-flip mutant exit 101 | (none stated) |
| G19 C-header parity | ALIVE | c99+c11 -ffreestanding -Werror header compiles exit 0; 16-bit-int target refuses (fail-closed) | (none stated) |
| G20 make gate0 replay | ALIVE | make gate0 exit 0 (twice, incl. after rm ggen.lock); release workspace build exit 0 | verify_disassembly_invariants.py standalone exit 2 (emit-asm build 101) — attributed, open |
| G08 (see above) | — | — | G2-PARETO rollup OPEN (HANDOFF.md:128) |

## B-lanes (cscb ladder)

| Lane | Standing | Load-bearing verified fact | OPEN item |
|---|---|---|---|
| B01 cscb C1+C2 | ALIVE | clean release exit 0; CSCB_MUTATE=branchy exit 1 (C2 p99 flip) | (none stated) |
| B02 | PENDING | receipt file absent | entire lane |
| B03 cscb C4 alloc | ALIVE | clean exit 0, window_counts()==(0,0); CSCB_MUTATE=alloc exit 1 (allocs=1 breach) | one dead_code warning (cgap-core reset_breach, foreign) |
| B04 cscb C5 | ALIVE | exit ABI 0/1/2/3; earlier non-flip mutants -> VACUOUS GATE exit 1 (correct refusal) | riscv32imc witness treated as divergence (disclosed) |
| B05 cscb C6 retraction | ALIVE (courts) | mutant flip exit 1 witnessed; retraction correct (10 phantoms detected) | HONEST C6 FAIL: 248M facts/s vs 800M target — typed FAIL stands |
| B06 cscb C7 sealing | ALIVE | clean exit 0 x3; rate-floor flip exit 1; digest-mutation court flips | debug rate < floor (expected, unoptimized) |
| B07 cscb C8 | ALIVE | 41 passed / 0 failed exit 0; over-cap verdict fails via same evaluate() | sibling court file compile-abort (foreign, census law) |
| B08 cscb runner | PARTIAL | runner ALIVE; all-bins-missing -> exit 0 (8x [SKIP] exit 2 typed SubjectMissing) | ladder aggregate exit 1 (C3, C6 honest fails); CSCB_MUTATE=zero-tax court CANNOT FLIP — receipt claims FAIL per law |

## C-lanes

| Lane | Standing | Load-bearing verified fact | OPEN item |
|---|---|---|---|
| C01 | PENDING | receipt file absent | entire lane |
| C02 csct coverage lift | ALIVE | cargo test -p csct exit 0 (51 baseline; new AndonRefusal + FAIL-render tests) | (none stated) |
| C03 bn3 edge courts | ALIVE | edge_closure_courts 5/5 exit 0; M1/M2/M3 sandbox mutants all exit 1 (flips) | (none stated) |
| C04 branchless boundary cov | ALIVE | cargo test -p cgap-branchless exit 0 (59 passed incl. 7 new C04 courts); AE4 re-run exit 0 | (none stated) |
| C05 dead-code excision | ALIVE | --self-check MUT-A/MUT-B fired exit 1 then 0 overall; delta -18 excisions, build exit 0 | parquet with DEFAULT features exits 101 (18 compile errors, FA9/AE1 surface) — OPEN defect |
| C06 scripts gate coverage | ALIVE (tests) | pytest tests/gates/test_gate_exits.py 17 passed exit 0 (194.77 s) | 90% coverage floor HONEST RED: enforce_coverage_floor.py --floor 90 exit 1, four red scripts |

## git-lanes (cgap-git)

| Lane | Standing | Load-bearing verified fact | OPEN item |
|---|---|---|---|
| git-01 | PENDING | receipt file absent | entire lane |
| git-02 pkt-line | ALIVE | pkt_line_court 10/10 exit 0; zero-grep exit 1 on encode/decode (zero-alloc) | other-lane compile-aborts (census law) |
| git-03 ingress ring | ALIVE | IngressError andon exit class 101 pinned by court; overflow flip twin witnessed | see also waves/lane-git-03-ingress-ring-receipt.md |
| git-04 delta resolver | ALIVE | delta_resolver_court 8/8 exit 0; lib 61/61 exit 0 | full cgap-git aggregate compile-aborts in foreign in-flight files |
| git-05 object store | ALIVE | object_store_court 6/6 exit 0 (Eytzinger < vs <= bug found+fixed via court flip) | (none stated) |
| git-06 tree synth | ALIVE | 6/6 courts exit 0, typed flip witnessed | foreign compile-aborts (census law) |
| git-07 | PENDING | receipt file absent | entire lane |
| git-08 | PENDING | receipt file absent | entire lane |
| git-09 admission gate | ALIVE | admission_gate_court 6/6 exit 0; mutant -> courts 2/3/4 FAIL (3/3 split, non-vacuous) | foreign court compile-aborts (census law) |
| git-10 patent court git | ALIVE (scoped) | patent_court_git 4/4 exit 0 baseline; SHIM-COURT budget mutant FLIPPED (3/1) | scoped standing; ROW_GREPS gate witnessed upstream |
| git-11 retract | ALIVE | retract_court 8/8 exit 0 debug+release; MUTANT FLIP 920 orphan derivations witnessed | honest throughput FAIL recorded vs dred 839-944M facts/s reference |
| git-12 octopus merge | ALIVE | octopus_merge_court 5/5 exit 0 (exit-101 child witness by design) | whole-crate run exit 101 compile-abort (foreign, census law) |
| git-13 ref store | ALIVE | ref_store courts 5/5 exit 0 post-restore; M1/M2/M3 mutations all FLIPPED | (none stated) |
| git-14 mcp driver | ALIVE | mcp driver court 6/6 exit 0; lib 61/61 exit 0 | foreign compile-aborts (census law) |
| git-15 a2a swarm | ALIVE | a2a_swarm_court 4/4 exit 0; sandbox mutation -> named court FAIL (flips) | aggregate exit 1 (87 foreign errors, retract_court.rs; census law) |
| git-16 | PENDING | receipt file absent | entire lane |

## csct lanes (L01-L14) — wave register standing

At wave-1 all 13 stage lanes were UNVERIFIED:compile-pending
(waves/csct-wave-register.md). Post-drain, G11/G12 witnessed the full
crate green (37 lib + 7 runner + 7 court, exit 0), which supersedes
compile-pending for the crate surface; per-lane receipts keep their
original standing vocabulary below.

| Lane | Standing (as receipted) | Load-bearing verified fact | OPEN item |
|---|---|---|---|
| csct-01 scaffold | UNVERIFIED:compile-pending | (no compile attempted; sequencing) | superseded by G11 full-crate green |
| csct-02 types | ALIVE (sandbox) | standalone rlib build exit 0; 9/9 tests exit 0 | root-workspace compile UNVERIFIED at receipt time |
| csct-03 warmer | ALIVE (sandbox) | warmer tests 7/7 exit 0 | shared-crate build FAILED at lane close (siblings); self-caught heredoc truncation incident disclosed |
| csct-04 reframe | UNVERIFIED:compile-pending | sandbox 9/9 exit 0 (4 reframe + 5 types) | compile into shared tree pending at receipt time |
| csct-05 drowning | ALIVE (sandbox) | sandbox 8/8 exit 0 | full-crate run exit 1 (missing court.rs at close) |
| csct-06 emotional | ALIVE (sandbox) | sandbox 8/8 exit 0 | shared-tree run exit 101 compile-abort (census law) |
| csct-07 new-way | UNVERIFIED:compile-pending | sandbox 8/8 exit 0 incl. 3 courts; both mutants flip | shared-lib compile-abort at receipt time |
| csct-08 solution | UNVERIFIED:compile-pending (ALIVE-in-sandbox) | sandbox release 8/8 exit 0 | shared-tree compile pending at receipt time |
| csct-09 mutation | ALIVE (sandbox) | sandbox mutation tests 4/4 exit 0 | shared-tree --lib mutation exit 1 (compile-abort) |
| csct-10 andon | ALIVE (module) | sandbox 6/6 exit 0; polarity mutant 4/2 FAIL (flips) | full-crate compile BLOCKED at receipt time |
| csct-11 court | UNVERIFIED:compile-pending | (sandbox evidence only) | exit 101 compile-abort at receipt time; superseded by G11 |
| csct-12 runner | ALIVE | --suite=nope exit 2; --receipt exit 3; suite 7/7 exit 0 | (none stated) |
| csct-13 docs | ALIVE | line-length check exit 0 (0 over-length lines); grounding reads exit 0 | (none stated) |
| csct-14 register | ALIVE | register files on disk; wave-1 audit landed | all-lane compile-pending — superseded by G11/G12 green |

## Cross-cutting receipts

| Item | Standing | Load-bearing verified fact | OPEN item |
|---|---|---|---|
| w6-g7-patent-gate-flip (G7F) | ALIVE | G7-PATENT 4/4 exit 0 x2 runs; sandbox claims-text mutant FLIP_EXIT=1 | (none stated) |
| countermeasures/lane-06 behavioral patent gate | ALIVE | gate now runs patent_behavioral_court; gate_exit = 0 iff all_distinct AND court_ok; m_key=MASK_ALL mutant exit 101 (flipped) | stubbed-command fail-path witness disclosed (stub flips command, not gate logic) |
| repair-01 branchless census | ALIVE | cargo test exit 0 requires 20 HOT cells 0-branch | (per receipt) |
| repair-04 dred cascade | ALIVE | 5 dred_cascade courts exit 0 | (per receipt) |
| repair-05 production wire | ALIVE | (receipt commands) | Outstanding section present in receipt |
| repair-06 radix default flip | ALIVE | re-measure exit 0 with exact-equality cache counter | (per receipt) |
| repair-07 prune bounds | ALIVE | parquet default-features exit 0; --features std exit 0 (stats_skip_court 4/4) | — |
| repair-08 alloc boundary | ALIVE | new ceiling 2x measured; ceiling flip court exit 0; 14,216,246 B witness | — |
| repair-09 warnings sweep | ALIVE | cargo build --workspace exit 0, ZERO in-code warnings | — |
| repair-10 cgap.h parity | ALIVE | cargo test -p cgap-ffi exit 0 (lib 50, all targets) | — |
| repair-11 closure alloc | ALIVE | storage release tests exit 0 | — |
| repair-12 drain witness | ALIVE | fsck --strict exit 0 on drained spill payload | — |
| repair-13 cpk study | ALIVE | cpk_study exit 1 (honest fail-closed, N>=100, real limits) | feeds G09 |
| repair-14 scripts pyright | ALIVE | pyright 0 errors exit 0 x3 scripts | — |
| repair-15 nanoarrow C1 | ALIVE | baseline 7/7 exit 0; size-check-removed mutant exit 1 (flipped) | — |
| repair-16 dred swar wire | PARTIAL | KEEP on perf fence; dead-library gate PASS exit 0; PURGE_COUNTS identical | divergence recorded in receipt |
| repair-17 bitplane stack | ALIVE | membership court 3/3 exit 0 (contains 241 s) | — |
| repair-18 swarlt topbit | ALIVE | exhaustive correctness 16,384/65,536 wrong BEFORE -> 0 AFTER, exit 0 both | — |
| repair-19 scan module | ALIVE | scan_courts 8/8 + 34/34 exit 0 | — |
| repair-20 xorfilter gate | ALIVE | courts 0 failed exit 0 | — |
| repair-21 transpose | ALIVE | 34 lib + 3 census + 8 scan + 7 transpose courts exit 0 | — |

## Session-level known blockers (aggregate)

1. G2-PARETO rollup OPEN (G08; HANDOFF.md:128) — no post-dating G2 receipt.
2. B05 C6 throughput honest FAIL: 248M vs 800M facts/s target.
3. C06 scripts 90% coverage floor HONEST RED (4 red scripts, exit 1).
4. C05 parquet DEFAULT-features build exits 101 (18 FA9/AE1 compile errors).
5. B08 zero-tax anti-vacuity court cannot flip (receipt itself claims FAIL).
6. Operator publish freeze (waves/publish-halt.md, FROZEN-BY-OPERATOR);
   cgap-ffi BLOCKED on parquet 26.11.3 re-publish; wasm-footprint host-link
   defect undiagnosed; 3 already-live repairs owed on lift (blake3/arrow/
   parquet 26.11.3).
7. HANDOFF gate ledger: G1-PUBLISH, G3-MAT-BENCH, G4-DRED, G7-PATENT
   (since flipped by G18/G7F), G8-STARDF-F1 rows BLOCKED as of receipt time.
8. G20: verify_disassembly_invariants.py standalone exit 2 (emit-asm build
   101) — open, attributed.
9. AE4 full-court arm E loud [SKIP] (binary-mutant arm, no patchable
   candidate in witness archive) — environmental, disclosed per ABI.
