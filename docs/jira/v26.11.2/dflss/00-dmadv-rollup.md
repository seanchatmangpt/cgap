# 00 DMADV Rollup (DF-6)

Five-phase DMADV ledger for the v26.11.2 starDF campaign. Lane DF-6,
2026-11-02, repo /Users/sac/cgap @ main. DF-1..DF-5 had NOT landed when
this rollup was written (`ls docs/jira/v26.11.2/dflss/` -> directory
absent, exit 1); every phase below is reconstructed from the register
of truth: 14-dflss-charter.md, 15-ctq-tree.md, audit/ (ledger +
release-blockers), and the lane/wave receipts.

## Define

Charter: `docs/jira/v26.11.2/14-dflss-charter.md` (sponsor, SMART goals
G1-G6, SIPOC, tollgates T1-T5, FMEA). VOC->CTQ flowdown:
`docs/jira/v26.11.2/15-ctq-tree.md` (15 CTQ rows over 4 customer
segments S1-S4, with USL/target/LSL per row).

## Measure

Measurement rig of record: `crates/bench/src/bin/cpk_study.rs` with the
G09 stabilization (sliding-window n=30 trials, WINDOW=30, exit contract
Cpk(overall) < 1.33 -> exit 1 fail-closed; CPK_INJECT_FAIL=1 flip
witnessed at exit 1). Owner receipt:
`docs/jira/v26.11.2/lanes/lane-g09-cpk-stabilization.md`
(release build exit 0; healthy run exit 0 with CTQ-06 mean 1.842
GiB/s, sigma 0.0097, lsl 1.0; injected-fail run exit 1, Cpl -84.69,
failed_gates [CTQ-06]). Prior census: n=100 Cpk 0.855 with
sigma_overall 0.26 -> between-sample drift, within-sigma Cpk 2.131
(lane-g09:11-20).

Fresh measured tables this session (audit-ledger.md, SEED-A
2026-11-02, re-read on disk by this lane):

| Item | Measured value | Receipt |
|---|---|---|
| CTQ-06 throughput Cpk | 1.842 GiB/s mean, sigma 0.0097, n=30 | lane-g09:56 |
| FA10 max-consumer .text | llvm-size 102,389 B (22% headroom under 131,072) | ledger G08 row |
| C6 retraction | 248.06M (b05) / 323.84M (git-11) facts/s vs 800M — typed FAIL | ledger rows |
| Commit (G5, prior) | 1.05-1.07 us/tx vs 4 USL — MET | 15-ctq-tree.md:53 |
| Ingress (G4/G5a, prior) | 10.20 ns/frame fastest vs 15 USL — MET | 15-ctq-tree.md:51 |
| Arrow object (G1a) | 28,679 B pre-link OBJECT (87.5% of 32,768) | 15-ctq-tree.md:28 |

Standing law: the parquet 55,711 B figure is UNVERIFIED-BY-RECEIPT
(14-dflss-charter.md:35-36) and is not promoted here.

## Analyze

Root causes actually witnessed this session (each cites its receipt;
none inferred):

1. **C05 used-binding misfire.** C05's warning sweep excised two
   genuinely-used bindings in crates/bench and left an unused_mut:
   chains-parquet.rs:467, chains-seal.rs:492, third leftover at the
   warnings census. Attributed in lane-fix07:48-51 under
   disclose-in-both law; correctness over silence.
2. **rep1-census shared-fixture race.** Concurrent census invocations
   raced shared obj artifacts; witnessed exact panic
   (lane-g10:18); root-cause fix landed as per-process
   `work()` via `OnceLock` keyed on `std::process::id()`
   (branchless_census.rs:55-60; release-blockers R6 CLOSED-VERIFIED).
   A second hook-vs-hook target race was found and fixed in the same
   lane (lane-g10:29,39).

3. **Stale binaries defeating fresh-build evidence.** G05's LSP wire
   rebuild had to re-derive evidence from binaries with fresh mtimes
   (Nov 2 12:53) explicitly newer than the source mtime (rule_pack.rs
   12:27) before the clean EMPTY result was admissible
   (lane-g05:14,42-43). Same class as the mtime-bump restore law
   (a02-finish-receipt.md:170-184): byte-exact restore does not
   invalidate cargo fingerprints.
4. **429 lane deaths.** GIT-07 is a retry lane; its receipt names the
   prior instance dying on API 429 (lane-git-07-egress-ring.md:3).
   repair-05 is a coordinator-assembled receipt because the lane
   429-terminated (waves/w7-errata.md:18). GIT-03's receipt carries a
   separate `lane-git-03-ingress-ring-receipt.md` retry artifact, but
   its on-disk text does not name 429; the 429 witness for GIT-03 is
   UNVERIFIED on disk (dispatch asserted GIT-03/07; only GIT-07 and
   repair-05 are receipted).
5. **Long-write corruption incidents.** Witnessed and disclosed: a
   csct-03 heredoc truncation self-caught at lane close
   (audit-ledger.md csct-03 row) and the write-integrity scaffold
   gate verifying no-truncation on landing
   (lane-dx-07:10). Repo law already encodes the mitigation: <50-line
   chunks or python-assemble, verify on-disk bytes (CLAUDE.md
   Agents section; _LANES-TEMPLATE.md contract).

FMEA top RPNs stand as chartered (14-dflss-charter.md:96-105): stale
mtime vacuous pass (RPN 120), pre-link object asserted as ELF (90),
1 blr regression (84) — all three map one-to-one to witnessed root
causes 3, the subject-class rule, and the still-open blr census item
(p2-asm-0blr-census.md:37, one blr in unify_to_fixpoint_opt).

## Design

Component courts inventory — census run this session:

    ls crates/*/tests/*court*.rs | wc -l   -> 62 court files
    grep -rln "_court|#\[test\]" crates/*/tests/*court*.rs | wc -l -> 62
    ls docs/jira/v26.11.2/lanes | wc -l    -> 68 lane receipts

Named courts with their witnessed mutation flips (all exits re-read
from audit-ledger.md this session):

| Court / suite | Mutation (anti-vacuity) | Flip witness | Exit |
|---|---|---|---|
| cpk_study gates (G09) | CPK_INJECT_FAIL=1 | CTQ-06 FAIL, Cpl -84.69 | 1 |
| transpose_courts (G02) | symbol witness BEFORE | exit 1 -> AFTER exit 0 | 1->0 |

| scan_courts (G01) | disasm invariants BEFORE | exit 1 -> AFTER exit 0 | 1->0 |
| branchless census (G01) | 20 HOT cells 0-branch | cargo test -p cgap-branchless | 0 |
| lspr attributes (G04) | mutant battery | mutants flip, 72 passed | 0 |
| LSP wire (G05) | riscv32imc court | exit 0, fresh-mtime binaries | 0 |
| did_save court (G06) | suite flip arms | 90 passed / 0 failed | 0 |
| conformance seal (G07) | disasm_court arms | exit 0 (one transient flake noted) | 0 |
| csct courts (G11) | stage-1-empty mutation | guard trip | 1 |
| csct warmer (G12) | 2/2 warmer tests | exit 0 | 0 |
| boundary_gate --deps (G16) | poison-arm flip OPEN | exit 0 (flip OPEN finding) | 0 |
| zero_string_gate (G17) | /tmp fibo poison fixture | fibo:FndUtlCur:USD hits | 1 |
| G7-PATENT (G18/G7F) | budget-flip mutant / claims-text mutant | exit 101 / FLIP_EXIT=1 | 101/1 |
| C-header parity (G19) | 16-bit-int target | refuse, fail-closed | non-0 |
| make gate0 (G20) | rm ggen.lock replay x2 | byte-identical | 0 |
| cscb C1/C2 (B01) | CSCB_MUTATE=branchy | C2 p99 flip | 1 |
| cscb C3 alloc (B03) | CSCB_MUTATE=alloc | allocs=1 breach | 1 |
| cscb C5 exit ABI (B04) | non-flip mutants | VACUOUS GATE exit 1 (correct refusal) | 1 |
| cscb C6 (B05) | mutant flip | exit 1 witnessed; throughput honest FAIL | 1 |
| cscb C7 (B06) | rate-floor + digest mutation | exit 1 both | 1 |
| cscb C8 (B07) | over-cap via same evaluate() | 41 passed | 0 |
| cscb runner (B08) | CSCB_MUTATE=zero-tax | CANNOT FLIP -> FAIL per law | FAIL (R10) |
| bn3 edge courts (C03) | M1/M2/M3 sandbox mutants | all exit 1 | 1 |
| dead-code excision (C05) | MUT-A/MUT-B --self-check | exit 1 then 0 overall | 1->0 |
| gate tests (C06) | pytest 17 passed | coverage floor honest RED | 0/1 |
| pkt_line_court (git-02) | zero-grep encode/decode | exit 1 (zero-alloc) | 1 |
| ingress ring (git-03) | overflow flip twin | andon exit class 101 | 101 |
| delta resolver (git-04) | court flip | 8/8 | 0 |
| object store (git-05) | Eytzinger < vs <= | found+fixed via flip | 0 |
| tree synth (git-06) | typed flip | 6/6 | 0 |
| admission gate (git-09) | mutant | courts 2/3/4 FAIL 3/3 split | non-0 |
| patent git (git-10) | SHIM-COURT budget mutant | 3/1 flip | non-0 |
| retract (git-11) | MUTANT FLIP | 920 orphan derivations | non-0 |
| octopus merge (git-12) | exit-101 child by design | 5/5 | 0 |
| ref store (git-13) | M1/M2/M3 | all FLIPPED | non-0 |
| mcp driver (git-14) | court flip | 6/6 | 0 |
| a2a swarm (git-15) | sandbox mutation | named court FAIL | non-0 |
| patent behavioral | m_key=MASK_ALL | exit 101 | 101 |
| repair-15 nanoarrow | size-check-removed | exit 1 | 1 |
| repair-18 swarlt | exhaustive 16,384 wrong -> 0 | exit 0 both | 0 |
| repair-08 alloc | ceiling flip court | exit 0 | 0 |

Courts with NO witnessed flip this campaign (per law: FAIL, not pass):

- CSCB_MUTATE=zero-tax (B08) — receipt itself claims FAIL (R10).
- boundary_gate.py poison arm — flip prescribed but not run
  (lane-g16; OPEN finding, dispatch-limited).
- AE4 full-court arm E — loud [SKIP], binary-mutant arm,
  environmental (audit-ledger G01 row); a loud skip is not a flip.

## Verify

Gate matrix (DF-5 not landed; fresh gate exits re-read from
audit-ledger.md this session):

| Gate | Command class | Exit | Witness |
|---|---|---|---|
| boundary_gate.py --deps | gate | 0 | G16 |
| zero_string_gate.py | clean | 0 | G17 (22 crates, 88 artifacts) |
| zero_string_gate.py | /tmp poison | 1 | G17 flip |
| make gate0 | ggen replay x2 | 0 | G20 (incl. post rm ggen.lock) |
| G7-PATENT | 4/4 DISTINCT-MECHANISM + court | 0 | G18/G7F |
| cpk gates | healthy / injected | 0 / 1 | G09 |
| coverage floor --floor 90 | enforce | 1 honest RED | C06 (4 red scripts) |
| receipt_lint.py | advisory census | 445 findings | R5 (grew 143->212->445) |
| workspace build | cargo build --workspace | 0 | repair-09 |
| parquet default features | cargo test -p cgap-parquet | 101 | R2 (FA9/AE1 un-gated) |
| riscv32imc ceilings | FA10 llvm-size | 102,389 B | G08 (22% headroom) |

### Honest OPEN list (cross-checked against audit/release-blockers.md)

No double registration: each row points at its blocker-register row;
this rollup registers nothing new.

| Rollup item | Register row | Note |
|---|---|---|
| C6 retraction throughput honest FAIL vs 800M | R1 | gate decision or repair owed |
| parquet default-feature test compile abort 101 | R2 | FA9 court un-gated |
| CSCB_MUTATE=zero-tax cannot flip | R10 | court FAIL per law |
| G2-PARETO L1/L2/L4/L5 rollup receipts | R4 | L3 closed |
| F5/F10/WG2/WG3 provenance receipts | R7 | UNVERIFIED, no rounding |
| coverage floor honest RED (4 scripts) | R3 | floor not lowered |
| receipt_lint 445 census (growing) | R5 | advisory, un-gated |
| build debris (rmeta + wrapper .o) | R8 | delete-never-commit |
| commit plan stale (refreshed by MV-8) | R11 | refreshed section landed |
| boundary gate poison-arm flip OPEN | lane-g16 receipt (new) | recorded there, NOT re-registered |
| AE4 arm E loud [SKIP] | (session blocker list #9, audit-ledger.md:146) | environmental |
| verify_disasm standalone exit 2 | (session blocker #8) | attributed, open |
| 1 blr in unify_to_fixpoint_opt | charter A4 (14-dflss-charter.md:114) | T3 exit criterion |

### Phase-gate verdict (per CTQ, from 15-ctq-tree.md rows)

VERIFIED = a this-session command+exit against the exact subject (this
session = SEED-A 2026-11-02 receipts; pre-campaign numbers are cited
as-of-receipt and never re-dated). DEFECT-OPEN = measured, honestly
failing. ASPIRATIONAL-UNVERIFIED = no witness, no decided gate.

| Segment.CTQ | Verdict | Cite |
|---|---|---|
| S1 linked-ELF ceiling | ASPIRATIONAL-UNVERIFIED | 15-ctq.md:36-38; G08 object 102,389 B |
| S1 zero-heap | VERIFIED | ledger G04/repair rows; charter G2 MEASURED |
| S1 spoliation 0x5032_38FF | VERIFIED | 15-ctq-tree.md:32-34 |
| S1 BLAKE3 receipt coverage | VERIFIED | w3-t5-pack-requal 14/14 (as-of-receipt) |
| S2 ingress ns/frame | VERIFIED | 10.20 vs 15 USL (as-of-receipt; charter G4 [TARGET] satisfied) |
| S2 commit us/tx | VERIFIED | 1.05-1.07 vs 4 USL |
| S2 0 blr / 0 brk | DEFECT-OPEN | p2-asm-0blr-census.md:37 (1 blr) |
| S3 throughput >= 1M facts/s | DEFECT-OPEN | R1: 248M/324M vs 800M gate, honest typed FAIL |
| S3 WAL byte-identical replay | ASPIRATIONAL-UNVERIFIED | 15-ctq-tree.md:85-86 [TARGET-v26.11.2] |
| S3 typed refusals | ASPIRATIONAL-UNVERIFIED | 15-ctq-tree.md:85-86 [TARGET-v26.11.2] |
| S3 DRed filter scope | VERIFIED | w6-dred-relabel-errata |
| S4 receipt coverage 100% | DEFECT-OPEN | R5 (445 unreceipted claims) + R7 |
| S4 court non-vacuity | DEFECT-OPEN | R10 (zero-tax cannot flip); boundary poison arm OPEN |
| S4 license URN pin | VERIFIED | frozen byte-pin, CLAUDE.md law |
| S4 seed non-conflation | VERIFIED | w3-t7-hygiene PASS (as-of-receipt) |

Verdict tally: 8 VERIFIED, 5 DEFECT-OPEN, 2 ASPIRATIONAL-UNVERIFIED.

**Falsifier (verbatim, governing):** DMADV Verify phase passes iff
every CTQ row cites a this-session command+exit and zero rows are
aspirational without a decided-gate pointer.

Applied: the falsifier is NOT satisfied this session — 5 rows are
DEFECT-OPEN and 2 rows are ASPIRATIONAL-UNVERIFIED (both WAL and
typed-refusals rows carry the [TARGET-v26.11.2] pointer but no decided
gate receipt). Verify = NOT PASSED; tollgates T2/T3/T5 remain OPEN per
the charter (14-dflss-charter.md:89-93).

## Receipt: commands + exits (this lane, this session)

| Command | Exit | Purpose |
|---|---|---|
| ls docs/jira/v26.11.2/dflss/ ; lanes ; audit | 1 (dflss absent) | DF-1..5 landing census |
| grep census over lane-g09/fix07/g10/g05/git-07 receipts | 0 | Analyze root-cause cites |
| ls crates/*/tests/*court*.rs \| wc -l | 62 | court inventory |
| ls docs/jira/v26.11.2/lanes \| wc -l | 68 | receipt inventory |
| grep -rn "429" waves/*.md HANDOFF.md | 0 (2 hits w7-errata) | 429-death witnesses |
