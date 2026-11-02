# AU13 — Architectural Missteps & Dead-End Rework (Phase 2 model)

Grounded in v26.11.2's own measured evidence: each class below is a dead end an
enterprise team pays for BEFORE reaching this design. Detection lag = stated
ASSUMPTION. Cost classes: S (<1 wk), M (1-4 wk), L (quarter+).

| # | Class | Session evidence (measured) | Enterprise equivalent | Detection lag (ASSUMPTION) | Rework | Preventing mechanism |
|---|---|---|---|---|---|---|
| 1 | GC/owned-collection first draft | H8: 1,117 allocs / ~1.2-1.67 GB churn per closure run @N=1M (lane-08:96, repair-11:4); F9 flat-table rejected on 14x facts/s regression (repair-11:78) — 343k vs 4.70M facts/s | Team builds closures on per-round `Vec` churn + custom open-addressing table; GC pressure blamed on runtime, not design | Weeks-months; found only when a memory profile lands in a P1 | **L** | Census court: alloc-count pins in-lest (`court_f9c` churn envelope < 1.676 GB, repair-11:67); no-alloc divan columns; zero-alloc fixpoint driver |
| 2 | Interpreter-first evaluation | MB4 admission: the fixpoint driver re-evaluates beyond delta — semi-naive landed only late (closure.rs:94-136 round-delta + SeenSet; BFX borrowed-prefix LANDED unmeasured, lane-04:50-96) | Naive fixpoint re-scans the full store per pass; "works at 10k facts", collapses at 1M | One release cycle; visible only as load-test cliff | **L** | Delta-driven `dR x B` via `Adjacency::stream` as the ONLY sanctioned shape; bench protocol pins facts/s at fixpoint (06-bench-protocol:156) |
| 3 | Premature microoptimization | DREDP SWAR swap refused on measured -11% facts/s (repair-16:64,103); scalar per-lane loop autovectorizes to 944M facts/s | Bit-twiddling "fast path" replaces the loop the compiler already vectorizes | Never — until someone benchmarks both arms honestly | **S** (if caught) | Typed refusal + perf fence: bit-identical output still rejected on the -11% fence, receipted (repair-16:103); BR3 "refusals must branch" |
| 4 | Measure-the-wrong-arm | FA3: `JoinProbeScratch` `#[derive(Default)]` -> `radix: false`; the "radix" bench leg ran the RETIRED nested arm; 0.97-0.98x verdict + 22x-over-target LANDED-NOT-MET was attribution-broken (forensic-lane-03:29-44) | Team benchmarks a feature flag that is never actually set; ships a verdict on the wrong code path | Quarters; the flag flip is silent | **M** | Flip witness: probe flip `cache_misses 0->1` proves the flag selects arms (forensic-lane-03:44); doc-vs-derive contradiction surfaced by census |
| 5 | Port-late | D5: `cgap-git` default features pull `std`; riscv32imc build exits 101 with E0463 at lib.rs:13, 410 errors (git-survey-lane-01:10, a5-sram:12,38) | no_std/portability treated as a porting task after the codebase is std-tangled | Months-years; discovered at first embedded bring-up | **L** | no_std probe pair (riscv control + attempt, E0463 typed BLOCKED); no_std census table (current-implementation-baseline:225) makes std-dep a first-class typed fact |
| 6 | Unenforced policy | E1: guard-generated.sh wired in settings.json mid-session but script absent on disk; hooks UNWIRED for the whole session — "claims without backing" (sm-5-audit:18,34) | A policy doc / lint rule nobody runs; the generated-surface is hand-edited for months | Until the first hand-edit corrupts a generated file | **M** | Fail-closed hooks: malformed stdin -> exit 3; wired-but-missing = laundering, caught by audit; flip-tests marker mutation (block->allow) proves wiring |
| 7 | Constant-input bench | FA10: budget gate's `--profile-deployment` vacuum ELF keeps only 7 entry calls; coverage 4.0-46.2% of audit rlib (p28 9.6%, parquet 4.0%); 39,062 B envelope certifies almost nothing (forensic-lane-10) | A size/latency budget measured against a binary that --gc-sections stripped of the real engine | Until a deploy-mutant or real workload breaches | **M** | DEPLOY-MUTANT COURT (66,176 > 49,152 breach flips) + symbol-table truth (llvm-nm coverage ratios) required beside every envelope number |
| 8 | Persistence assumption | FA9: 64 "OK" commits -> spill len 0; zero fsync/msync/sync_all; stage_commit µs figures measured an in-memory staging latch (forensic-lane-09; w6-fa9:42-49) | "Commit succeeded" read as "data is durable"; lost on power failure in production | Until the first crash-loss postmortem | **L** | Receipt law: CGAP_OK means STAGED never DURABLE — disclosure blocks at commit + drain sites; honest relabel of the µs figure; concurrent-writer gate 0x4347_0006 |

## Cross-class pattern

Three of eight (3,4,7) are measurement lies, three (1,2,5) are architecture
committed before the constraint was known, two (6,8) are claims outrunning
enforcement. cgap's answer is uniform: every number travels with a witness
(court, flip, typed refusal, receipt) or is marked as an assumption — an
enterprise team's cheapest rework avoidance is refusing to let an unbacked
claim become a landed verdict.
