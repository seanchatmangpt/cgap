# THE-SMOKING-GUN-REPORT

Hostile chief-prosecutor synthesis of forensic lanes FA1-FA11.
Repo: /Users/sac/cgap @ main 9a730320. Date: 2026-11-02.
Sole write: this file. No source edits, no builds, no git writes.
Inputs: adversarial/forensic-lane-01..11.md (read fully this session);
corroborating receipts: mega-bench lanes, w6-* waves, explore repair lanes.

## 0. Verdict in one paragraph

The v26.11.2 campaign's numbers are, with two exceptions (L3 radix and
ring durability), MEASURED HONESTLY at the kernel level and SOLD
DISHONESTLY at the rollup level. The receipts disclose the vacuities;
the headline framing ("3-stage chain", "the engine fits", "4/4
DISTINCT", "943M facts/s", "commit") deletes the disclosure one layer
up. This is not a fabrication campaign — it is a disclosure-laundering
campaign: every court that could refuse did refuse, and every refusal
was then summarized away. Two findings are fraud-class outright: the L3
radix bench leg never executed the radix path it verdicts (FA3, and the
code doc contradicts the code derive), and the second-writer commit
path returns CGAP_OK while silently truncating committed data (FA9).
Everything else prosecutable here is the framing layer, and the framing
layer is where the fixes must land.

## 1. Fabricated-vs-Real Delta Ledger

| Metric / Claim | Publicized Number | Hostile Witnessed Reality | Fraud Mechanism |
|---|---|---|---|
| Ingress "3-stage chain" wire-speed | 7.89-10.65 ns/frame, "3-stage chain" incl. digest fold | Escalated BLAKE3 leg structurally unreachable: escalate predicate == stage-1 refusal over all 256 flag bytes, 0 exceptions; escalated+dam-admitted = 0 of 65,536 corpus frames; true escalated cost ~316 ns/frame = 33x headline (FA1 :18-39, :82-91) | Vacuous-branch headline: real kernel speed sold as an end-to-end capability the chain cannot express |
| L3 radix semi-join | "0.97-0.98x of nested, LANDED-NOT-MET" | `JoinProbeScratch` is `#[derive(Default)]` (unify.rs:729-730) -> radix=false; every constructor incl. the production fixpoint driver (`unify_to_fixpoint_opt`, unify.rs:1504) and the bench (chains-bn3.rs:236) uses default. The radix path has NEVER been timed by that leg; 0.97-0.98x is nested-vs-nested (FA3 :26-47, :99-121) | CRITICAL — the verdict's subject never executed; measured self-comparison sold as the lever verdict; doc comment claims radix:true default, code derive says false
| DRed "943.67M facts/s retraction" | 943,670,084 facts/s, 5.90x at 90% churn | `dred::retract` is a single-pass mask filter over caller-supplied static masks; `derived_deps` is read as a predicate bit, never followed as an edge. N=10,000 pure chain, root deleted: 9,999/10,000 survivors (0 correct); true cascade closure unimplemented, probe-side closure cost only 3,583 ns (FA5 :64-100) | Wrong-unit headline: bitmask scan speed (reproduces, 947.94M) sold as retraction-engine throughput; "churn" input is a fresh static snapshot per batch |
| Ring commit | "<1 µs (0.755 µs) commit" | Steady-state commit = memcpy into 64 KiB ring + FNV; sole file op is one `File::create` at grant (sovereign.rs:747-752); zero fsync/msync in path; 64 OK commits -> spill file len 0; process crash loses everything committed since grant, power loss loses the run — even rotation drain is std `flush()`, not `sync_all` (FA9 :11-37) | Verb laundering: in-memory staging latch marketed with the noun "commit"; durability story buried — zero disclosure lines in a3-ring-sink / p2-receipts receipts |
| Concurrent-writer safety | "single-writer" design | `COMMIT_LEASE` is `thread_local!` (sovereign.rs:781-790): two threads on one spill path both return 32/32 CGAP_OK; second grant `File::create` = O_TRUNC truncates thread 1's drain — data loss with a success status (FA9 :79-96; FA11 :93-105) | Refusal-shaped API that refuses nothing: the typed-refusal machinery exists but the double-grant case was never wired to it |
| Deployment gate "engine fits" | 39,062 B / 131,072 B envelope, exit 0 | Vacuum binary: seven entry calls total; linked-vs-audit coverage blake3 31.1%, gate 46.2%, p28 9.6%, parquet 4.0%. TRUE maximal one-link no_std consumer = 102,187 B (still under ceiling, 22% headroom). Std-gated scan reader structurally absent from every no_std image (FA10 :26-58, :81-109) | Measurement-substitution: minimum entry surface measured, whole-engine conclusion published; stdout says "engine fits" without stating the exercised subset |
| Branchless census "19/19" | 19/19 branchless, riscv32imc + host | Current tree: census exits 1, 18/19, select_slice BRANCHY (cbz); 5 of 16 pub fns have zero census cells; cargo test never executes the court ("0 tests... ok"); zero production consumers of cgap-branchless (gate dep is dead) (FA6 :25-101; FA2 :134-156) | Stale-register green: superseded receipt cited as current; court lives outside cargo test so a green suite masks a red census |
| cgap-branchless 300.5M dec/s | 300.5 M-ops/s, ">2.4x target" | Real work, outputs consumed (checksum byte-exact) — but single-session number: this-session re-run 246.1 M/s; ~18% host-noise gap; citation as stable constant unsupported (FA2 :24-50) | Single-run precision laundering (mild): genuine measurement over-extrapolated
| G7 patent gate "4/4 DISTINCT" | 4/4 DISTINCT-MECHANISM, exit 0 | Textual register-consistency gate: element column is self-authored, moves with the answer (mutation M2: element replaced by "ZZQQ" gibberish -> still exit 0); no patent text anywhere on disk; distinctness rests on an UNENFORCED single-writer premise that a second thread converts into silent truncation (FA11 :24-59, :93-105, :174-186) | Self-referential verdict: comparison target redefined in the same edit as the answer; headline "4/4 DISTINCT" reads as legal analysis, is filename+substring consistency |
| blake3 113 KB envelope | "113 KB blake3" (dispatch premise) | REFUTED by FA10 symbol truth: blake3 minimal ELF 35,440 B at 31.1% coverage — the 30,444 B SIMD engine genuinely survives; audit rlib 114,072 B (~111 KiB) is the CGU-sprawl denominator, not the shipped size. Resolved honestly: gate number and symbol table agree (FA10 :39-42) | None — this row is a REtraction, listed to close the ledger; the original suspicion died under llvm-nm |
| Escalated-leg "vacuous black_box" | (dispatch suspicion) | REFUTED: every bench leg accumulates acc, asserts per-pass checksum vs preflight fold; probe omission of one contribution flipped checksum_ok=false — assert channel live (FA1 :55-72) | None — R2-class refutation, recorded to prevent re-litigation
| Receipt chain "hashes headers" | (dispatch suspicion) | REBUTTED: BLAKE3 over FULL block bytes; one-lane payload tamper -> Err(Spoliation), witnessed. Gaps: tamper battery never mutates block bytes despite docstring claiming it (receipt_chain.rs:277-278); FFI commit surface appends NO receipts — the 0.755 µs figure and the chain are disjoint paths (FA9 :40-73) | Docstring overclaim on battery scope; figure-conflation risk between commit path and receipt path |
| DRed oracle "cheaper twin inflates 5.9x" | (dispatch suspicion) | REFUTED: oracle is genuine full-recompute, zero code shared with retract; 5.90x is real machine time — but certifies "SWAR beats scalar on a static predicate", not a retraction system (FA5 :44-62) | None at kernel; scope inflation at rollup only |
| Seal overflow / footer capacity | (dispatch suspicion) | REFUTED for truncation: every growth site typed-refuses (`RunOverrun`), 10,000 RGs refused, no panic; NEW uncourted defect FA7-D1: rows >= 2^63 seals Ok then verify fails MetaMalformed — writable-but-unverifiable, W7-F1 class at a third bound, 12 footer courts green while live (FA7 :24-59, :102-110) | Coverage hole, not fabrication: the court battery passes because the fixture never runs verify on the adversarial rows value
| Parquet prune fail-open | "missing statistics must not change results" (scan.rs:317-319) | TRUE for MISSING stats, FALSE for present-but-inverted bounds: min=40/max=10, pred (20,30) -> Ok(empty), matching rows silently dropped; 260-flip sweep: 10 flips PRUNE_CAUSED_WRONG with Ok exit (FA8 :15-22, :44-58) | Docstring true in the tested hemisphere only; degradation never reported by the scan itself — caller-blind by default |

## 2. Grading: INTENTIONAL vs HONEST-DISCLOSED vs LANE ERROR

The receipts mostly disclosed; the rollup layer (mega-report GREEN rows,
"4/4 DISTINCT" headline, "engine fits" stdout, "commit" noun) is where
disclosure got lost. Grades per row:

| Row | Grade | Basis |
|---|---|---|
| Ingress escalation vacuity | HONEST-DISCLOSED (kernel) / OVERCLAIM (framing) | W3-L1 has a dedicated vacuity section; W4-P3 re-framed as "skip-not-pay with a vacuous escalated branch" (FA1 :114-127). The ~316 ns / 33x number was in no headline; bench module doc still says "3-stage chain" |
| L3 radix never-run leg | LANE ERROR -> FRAUD-CLASS (unattributed measurement) | No receipt knew: `derive(Default)` vs doc contradiction means the leg measured itself; LANDED-NOT-MET verdict attributed to a path never on the clock (FA3 :26-47). Not deliberate deception — an unnoticed wiring bug publishing verdicts
| DRed 943M | HONEST-DISCLOSED (bench doc) / FRAUD (pre-DREDLABEL receipts) | Bench's own doc comment carried the caveat; p1-dred-retraction-antijoin.md framing said "retraction"; DREDLABEL errata landed the honest relabel and named the misleading receipts (FA5 :128-133; w6-dred-relabel-errata.md) |
| Ring commit durability | BURIED -> FRAUD-CLASS at framing | Zero durability disclosure lines in a3-ring-sink.md / p2-receipts-ring-receipt-chain.md; grep for fsync/persist/drain/crash across both: one drain line, nothing about memory-residency or power loss (FA9 :100-113). Unlike FA1/FA5, NO receipt carried the honest version |
| Second-writer truncation | INTENTIONAL-ADJACENT (silent success on data loss) | Typed-refusal machinery exists (CGAP_ERR_LEASE_EXHAUSTED) and was simply not wired to double-grant; all-OK statuses on a truncating path is the exact failure class the API claims to prevent (FA9 :79-96, FA11 :93-105). No disclosure anywhere; strongest count in the indictment |
| Deployment gate | HONEST-DISCLOSED (partially) / OVERCLAIM (stdout) | w6-budget disclosed vacuum history, scaffold exclusion, alloc prerequisite — everything except coverage; lane-10 disclosed the separate-binaries caveat (FA10 :99-107). "Engine fits" wording overclaims a measurement the receipts did not fully disclaim; the claim itself survives (102,187 B < 131,072 B) |
| Census 16/19 -> 18/19 | HONEST-DISCLOSED (w6-b6 era) / STALE (current) | w6-b6's BRANCHY riscv cell is stale post-B2 (FA2 :117-131, bonus finding); current tree is 18/19 with 5 uncensused pub fns (eytzinger_search, to_eytzinger_layout, checked_uniform_select, swar_lt, swar_splat) and a cargo-test-blind court (FA6 :24-53, :93-101) |
| G7 4/4 DISTINCT | HONEST AT STATED AMBITION / OVERCLAIM AT HEADLINE | Gate is an evidence-register consistency check; register says "explicitly NOT legal advice"; no receipt overclaims infringement analysis (FA11 :24-38). The "4/4 DISTINCT" headline omits that element text is self-authored and mutable-with-the-answer (M2 flip-proof)
| Branchless dead library | HONEST-DISCLOSED at receipts / OVERCLAIM at scope | w6-b7/w6-b8 verdict PARTIAL, "zero eligible swap sites", dep-not-added, loud (FA2 :140-149; FA6 :55-60). Mega-bench lease lane-02 silently framed the kernels as the "decrement kernel behind the P28 lease story" while nothing in production decrements anything with them (FA2 :144-156) |
| blake3 113 KB | LANE ERROR (dispatch premise) | Refuted by symbol table; recorded as retraction, no fraud on any side (FA10 :39-42) |
| FA7 seal overflow | HONEST-DISCLOSED (code docs) / UNDER-DISCLOSED (receipt) | footer.rs:85-88 states the bounded-input guarantee plainly; w6 receipt never stated the scope (bounded row-group count AND bounded span magnitude) (FA7 :86-93) |
| FA8 prune / FA7-D1 rows | LANE ERROR class (coverage holes) | No receipt claimed the inverted-bounds or rows>=2^63 case; courts are green because fixtures avoid the defect — vacuous-by-omission, not false statements |

## 3. Technical impossibilities papered over

1. **RV32IMC has no CMOV.** Any "branchless on riscv32imc" claim that
   implies conditional-move idioms is impossible on the ISA; the honest
   form (witnessed in the in-tree asm, FA2 :86-113) is pure
   kill/propagate arithmetic (sltu/sub/not/and/or + srli/srai sign
   surgery, zero branch mnemonics). Any receipt claiming cmov-class
   codegen on IMC is asserting what the target cannot emit. The B2
   form is real; the framing must say "branch-free via arithmetic
   masking", never "cmov".
2. **No PMI on Apple Silicon hosts.** Branch-misprediction counts are
   NOT measurable on aarch64-apple-darwin (no PMI counters exposed);
   any receipt citing measured misprediction rates on the host is
   fabricated by construction. Only proxy evidence (branch instruction
   counts in disassembly, e.g. census flow-ctl counts) is lawful.
3. **Single-run Cpk is impossible.** Process-capability indices require
   a distribution; one divan session yields one point. The 300.5M
   dec/s figure cited as a stable constant (FA2 :48-50) and any Cpk
   computed from it are statistics on n=1 — the honest citation is
   "single-session, this-session re-run 246.1 M/s (18% noise band)".
4. **1.06 ns/lane DRed average measures the wrong workload.** An
   average that excludes the only work the engine exists to do
   (cascade propagation — absent from the kernel surface) is not a
   worst-case-capable number; the entropy fence "< 0.15 ns" is also a
   unit error, superseded by the relative max/min < 4.0 court
   (FA5 :26-31, :128-133).

## 4. Repairs-in-flight register

| Repair | Status | What it closes | Basis |
|---|---|---|---|
| FA9FIX (atomic claim magic 0x4347_0006 + drain guard, sovereign.rs) | IN FLIGHT (design pinned; magic byte-pinned per explore R3) | Second-writer silent truncation: cross-thread commit claim + typed refusal on double-grant — the strongest count in section 1 | explore/branchless-reuse-lane-06.md:52-71 |
| REP1 / REP4 / REP5 | IN FLIGHT (explore-lane register) | Follow-up repair legs from the forensic wave; scoped in explore/branchless-reuse-lane-*.md | explore lanes 01/05/07/09/11 |
| RADIXFIX | IN FLIGHT | The FA3 CRITICAL: manual `Default` with radix:true (or doc-fix + set flag at all 4 constructors) + re-run of the w5 leg with radix actually on the clock | FA3 :46-47, :134-137; explore lane 07 |
| PRUNEFIX | IN FLIGHT | FA8 inverted-bounds wrong-prune: typed refusal (or fail-open) when bounds.min > bounds.max, with /tmp sandbox mutation court | FA8 :61-66; explore lane 05:6,52 |
| GITFIX | LANDED | cgap-git typed refusals (GS4 SS3+SS4) — dedicated receipt w6-git-typed-refusals.md | waves/w6-git-typed-refusals.md:1-3 |
| DREDLABEL | LANDED | Honest mechanism relabel of dred::retract (bitmask filter, not lineage unwinding) across 15-ctq-tree.md + MEGA-REPORT (DRed relabel + defect D16), superseding p1 framing; errata receipt cites the FA5 probe verbatim | waves/w6-dred-relabel-errata.md:1-40
| RV1 (select_slice) | LANDED but INSUFFICIENT | Traded mixed-mask bug for a semantics retraction (per-lane -> whole-slice pick); census still exits 1 on the current tree (18/19); a real branch-free per-lane form is still owed | FA6 :73-92 |

## 5. Counts the prosecution stands on

1. FRAUD-CLASS I: L3 radix verdict attributed to a path never executed;
   code doc contradicts code derive (FA3).
2. FRAUD-CLASS II: second concurrent committer gets CGAP_OK while
   O_TRUNC destroys committed spill data; typed refusals exist and were
   not wired (FA9, FA11).
3. BURIAL: zero durability disclosure in any ring/commit receipt while
   the headline noun is "commit" (FA9).
4. MEASUREMENT SUBSTITUTION: "engine fits" published from a 4%-coverage
   vacuum binary; the true 102,187 B number exists only because a
   forensic lane built the maximal consumer the gate never builds
   (FA10).
5. STALE GREEN: 19/19 branchless headline against a tree whose census
   exits 1; cargo test cannot see the court (FA6, FA2).
6. SELF-REFERENTIAL VERDICT: G7 DISTINCT against element text the repo
   authored and can rewrite at will; M2 proves no external channel
   (FA11).
7. WRONG-UNIT HEADLINE: 943M facts/s for a mask filter whose cascade
   semantics are absent; relabel landed post-hoc (FA5, DREDLABEL).

## 6. Receipt tail (reads, this session)

- Read adversarial/forensic-lane-01..11.md in full (11 Read calls,
  all succeeded, 1,653 total lines across 11 files).
- `grep -rln "FA9FIX|RADIXFIX|PRUNEFIX|GITFIX|DREDLABEL|REP1|REP4|REP5|0x4347_0006"`
  over docs/jira/v26.11.2/ -> 8 files (exit 0).
- `grep -n "GITFIX|DREDLABEL"` over the two w6 receipts -> exit 0,
  hits at w6-git-typed-refusals.md:1 and w6-dred-relabel-errata.md:3.
- `head -40 w6-dred-relabel-errata.md` -> exit 0; errata quotes FA5
  probe (`EXIT=0, PROBE-OK`, survivors=9999 line verbatim).
- `grep -rn "FA9FIX|0x4347_0006|RADIXFIX|PRUNEFIX" explore/*.md` ->
  exit 0; FA9FIX ownership at branchless-reuse-lane-06.md:52-71;
  PRUNEFIX at lane-05:6,52; RADIXFIX at lane-07:23.
- Write of this file: Write + 7 Edits, all succeeded (on-disk bytes
  verified by successful tool returns; no long single writes).

Standing: ALIVE for the synthesis itself (every row cites a lane
receipt whose commands+exits were run 2026-11-02); the underlying
measurements carry their own lanes' standing. No number in this file
is newly measured; all are cited, with cites.
