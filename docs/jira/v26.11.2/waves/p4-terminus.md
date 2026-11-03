# p4-terminus — campaign census (lane p4-terminus, gate-runner)

THE RUN TERMINATES HERE. Bounded poll on receipt FILES, 60s cadence,
90-min deadline (started 2026-11-02T00:0xZ). Repo `/Users/sac/cgap`,
branch main, HEAD 6801affa. No git writes; publishes FROZEN by operator
order. Sibling-terminus check: `waves/w1-20-terminus.md` and
`waves/w2-20-terminus.md` cover waves W1/W2 only — this receipt covers
the p1..p4 lane wave; no sibling p4-terminus existed at lane start
(count 0, witnessed by ls at lane start).

## Census at lane start (08 receipts, witnessed by ls)

p1: blake3-optlevel-ub-fix, closure-closure, dred-retraction-antijoin,
lineage-workflow, repair-unify-delim, repair-unify-repair (6)
p2: footprint-flash-ladder, penstock-escalation-law (2)
p3: NONE LANDED (named ABSENT — loudly)
p4: NONE (this receipt is the only p4 artifact)

## Table 1 — measured numbers (unit | method | receipt cite; never
cross-multiplied)

| metric | value | unit | method | cite |
|---|---|---|---|---|
| blake3 hash_many_simd/x2 throughput | 1.85 | GiB/s | cargo test bench_throughput --ignored, 256 MiB input, 5 rounds | p1-blake3-optlevel-ub-fix.md |
| blake3 hash_wide_simd | 0.43 | GiB/s | same | p1-blake3-optlevel-ub-fix.md |
| closure pairs/s @ N=65536 (229376 pairs) | 11280787 | pairs/s | Instant harness=false, min of 4 | p1-closure-closure.md |
| closure pairs/s @ N=1048576 (16252928 pairs) | 4987591 | pairs/s | same | p1-closure-closure.md |
| dred retract speedup @10/50/90% churn | 1.60x / 3.66x / 4.85x | ratio | min-of-30 wall-clock, oracle/retract ratio only | p1-dred-retraction-antijoin.md |
| penstock L1 frame digest | 10.20 | ns/frame | predicate identity 256/256, skip-not-pay; quiet host, divan fastest/median (claims row 5); contended divergence row 28 (124.06 ns/frame slowest) | p2-penstock-escalation-law.md, w4-p3-escalation-corpus.md, 11-measured-claims.md rows 5/28 |
| L1 chain speedup | 10.1x | ratio | same | p2-penstock-escalation-law.md |
| cgap-gate .text (riscv32imc, audit profile) | 940 | B | llvm-size -A, ELF objects | p2-footprint-flash-ladder.md |
| cgap-arrow wasm32 CODE | 85411 | B | llvm-size -A | p2-footprint-flash-ladder.md |
| cgap-parquet wasm32 CODE | 65459 | B | llvm-size -A | p2-footprint-flash-ladder.md |

## Table 2 — defect ledger (found / fixed / open, with coordinates)

| # | defect | coords | status |
|---|---|---|---|
| D1 | blake3 x2 NEON UB: `vst1q_one` wrote through shared-ref-derived ptr; opt-level>=1 dropped `^ cv_pre` on out words 8..16 | crates/cgap-blake3/src/multi_chunk.rs | FIXED (receipt p1-blake3; release+dev suites 0; anti-vacuity flips 101). Published 26.11.2 tarball still carries it — repair = blake3 26.11.3 on operator lift |
| D2 | arrow stale `generated_version` 26.10.29 embedded in 26.11.2 tarball | crates/cgap-arrow/src/generated_version.rs | OPEN — repair = arrow 26.11.3 on lift (publish-halt addendum) |
| D3 | bn3 mid-edit kill: unify.rs +548/-100 vs HEAD left by killed lane | crates/cgap-bn3/src/unify.rs | FIXED/verified (p1-repair-unify-delim + -repair: build 0, 119 unit + 7 lever courts pass; forced recompile) |
| D4 | `court_bitplane_seen` stack overflow (SIGABRT) in full cgap-storage test run | crates/cgap-storage (pre-existing, zero diff vs HEAD) | OPEN (logged, not repaired; lib+dred+closure suites green) |
| D5 | riscv32imc build BLOCKED for std-dependent crates | cgap-bn3, cgap-bont, cgap-storage (E0463 `std` not found; 438/372/140 errors) | OPEN — typed BLOCKED, no artifact exists |
| D6 | wasm32 CODE over the 32,768 B Flash ceiling | cgap-arrow 85411 B, cgap-parquet 65459 B [TARGET-v26.11.2], wasm-class not flash-class | OPEN (recorded as TARGET) |
| D7 | naming-law violation in `rust-toolchain.toml` (forbidden ecosystem word) | rust-toolchain.toml | OPEN (observed by p2-footprint; out of that lane's scope) |
| D8 | cgap-ffi publish BLOCKED: registry parquet 26.11.2 predates RingSink | crates.io dependency edge | OPEN — needs parquet re-publish on lift |
| D9 | wasm-footprint publish BLOCKED: host-link failure in tarball verify (build.rs host-cdylib leg vs registry rlibs) | publish lane addendum | OPEN — new defect class, diagnosis owed |
| D10 | L1 escalated leg repair surface (A04 §6.3 reserved-bits wire-law amendment) | crates/cgap-core/src/frame.rs:38,284-286; guard_flip_court.rs:288,565; benches/ingress.rs:198 | OPEN — REFUSED at lane scope, A19 register candidate, UNBUILT |

## Table 3 — registry table (FROZEN state as-is; live-witnessed this
session, crates.io API v1, UA cgap-p4-terminus/26.11.2)

12 crates LIVE at 26.11.2: cgap(3), cgap-arrow(3), cgap-bn3(2),
cgap-bont(2), cgap-bosym(2), cgap-core(2), cgap-crypto(2), cgap-gate(2),
cgap-git(3), cgap-p28(2), cgap-parquet(3), cgap-storage(2)

FROZEN at 26.11.1 (6): cgap-audit, cgap-cloud, cgap-ffi, cgap-fibo,
cgap-gpu, wasm-footprint — publishes resume only on operator order

ABSENT (first publish pending): cgap-integration
Registry name collision (not ours): `bench` (delisted, 1.1.0)
Registry-absent by design (publish=false): cgap-beam, cgap-court,
cgap-verification (structural/[[test]]-only), plus cgap-cli/ggen side-bins

## Table 4 — non-vacuity metagate (receipts with zero witnessed
command+exit named VACUOUS)

| receipt | witnessed commands w/ exits | metagate |
|---|---|---|
| p1-blake3-optlevel-ub-fix | 5 (incl. flip 101 -> 0, sandbox mutation court) | PASS |
| p1-closure-closure | 3 (test 0, bench 0, build 0; court caught real defect pre-fix) | PASS |
| p1-dred-retraction-antijoin | 3 (4/4 lib courts incl. 2 flip mutants; pre-existing failure disclosed) | PASS |
| p1-lineage-workflow | 2 (7 typed tamper courts + digest-movement guard) | PASS |
| p1-repair-unify-delim | 3 (build/test/workspace exits 0) | PASS |
| p1-repair-unify-repair | 3 (forced-recompile exits 0) | PASS (duplicate-lane instance of p1-repair; coverage identical) |
| p2-footprint-flash-ladder | 4 (3 BLOCKED exits 101 named loudly, sizes via llvm-size) | PASS |
| p2-penstock-escalation-law | 4 (sync/gate0/3 court runs, exits 0) | PASS |
| publish-halt | 1 (crates.io API x8) | PASS (measurement-only; no court claim) |

VACUOUS lanes: NONE at census time. Every p1/p2 receipt carries at least
one command+exit witnessed this session. A receipt that lands later with
zero commands will be named in an errata at integration.

## Table 5 — saturation verdict

Wave state at census: 8/expected receipts landed (p1 x6, p2 x2), p3
ABSENT at this write. Upstream lanes confirmed ALIVE: sibling poller
`/tmp/p3-pack-poll2.sh` (started 12:03Z) is itself waiting on p1-bgp +
shacl-mask receipts — those lanes are still in flight. NOT SATURATED:
saturation = a full wave returns zero defects; D2/D4-D10 are open with
named repair surfaces, and the p3 class has not landed. Verdict:
PARTIAL — the run terminates here with the frontier named, not closed.
