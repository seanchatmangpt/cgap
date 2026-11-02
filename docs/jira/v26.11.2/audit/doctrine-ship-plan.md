# doctrine-ship-plan

DOCTRINE-5 ranked gap-closure ship plan (2026-11-02). PLAN lane —
every item is a dispatched-lane contract, not an edit. Inputs of record
re-read this session: release-blockers.md R1-R11, audit-ledger.md,
dod-cgap-{blake3,git,lsp-max-wire}.md, lspu1-consume-plan.md,
lanes FIX-04/06, DX-04, LSPA-01. NOT consumed (absent at plan time):
DOCTRINE-1..4 outputs. "GIT-13 abstract-slot" — zero disk hits under
any spelling: carried UNVERIFIED, not assumed.

## A. Ranked gap-closure register

### A1 (P0) cgap-parquet default-features compile abort (R2)
Standing: CLOSED post-session — FIX-02 landed: per-item
#[cfg(all(test, feature="std"))] on the 6 FA9 court items; default exit 0,
--features std 195 passed. Coordinator note appended at land time.

### A2 (P0) CSCB_MUTATE=zero-tax court cannot flip (R10)
Lane: csct runner lane owning cscb_runner + one refusing child; child
reads CSCB_MUTATE=zero-tax -> typed ANDON exit 1, clean -> 0; flip is
mechanical env-set -> different outcome, both directions witnessed.
Scope M. (B08's FAIL stands until this lands.)

### A3 (P0) p99 mislabel in cscb_c6_retraction (:87-100)
FIX-04's named sibling defect, unowned. Lane: FIX-04-clone owning
exactly that file; min/median/max/N rename + no-p99 note. Flip:
forbidden-label grep court (reintroducing "p99" -> exit 1). Scope S.

### A4 (P0) object_store WIRE-PENDING (FIX-06 follow-through)
lib.rs carries WIRE-PENDING(lane-git-05); object_store hard-codes
cgap_blake3::hash (:97,:354) with cgap-blake3 dev-dep-only. Ruling:
digest-fn injection (tree_synth precedent) — kernel law forbids the
real dep. Lane owns object_store.rs + lib.rs mount; flip = constant
digest fn -> store rejects. Scope M.

### A5 (P1) in-crate cgap-blake3 mutation courts (EXP-04 T3)
cgap-blake3 tests/ has NO bitflip-rejection or chain-mutation test; the
court lives only in the csct bin. Lane: tests/chain_mutation_court.rs
(new, zero deps); three flips: bitflip in child digest -> exit 1;
identical digest substitution -> typed FAIL; re-derivation mismatch ->
exit 1. Scope M.

### A6 (P1) aarch64 pin coverage (AE4 is riscv-only)
run_tier2 supports aarch64; nothing pins branchless kernels branchless
on aarch64. Lane: lsp-max wire lane, rule_pack.rs aarch64 arm +
aarch64_pin_court.rs; clean -> n_refused==0, AE4 arm-C mutant text ->
>=1, hand-branchy aarch64 source -> >=1. AE4 script NOT edited
(additive). Scope M.

### A7 (P1) lsp-max client masking + double-silence (LSPA-2/4)
UNVERIFIED at plan time — no receipts on disk (in flight). Plan
requires each lands with a flip witness + filled commands block;
LSPA-01 receipt backfill owed. Scope S (backfill).

### A8 (P1) receipt_lint 445 -> 0 + fail-closed floor (R5)
DX-4 strict rules landed; 445 advisory. Lane: W8-successor, doc edits +
DOC-RECEIPT rule class; flip = strip a receipt ref from a /tmp fixture
-> lint exit 1; floor mode --strict --floor 0 wired into make gate0.
Scope L.

### A9 (P1) scripts coverage floor red (R3)
24.4/64.9/52.6/40.8% vs 90 — honest red preserved. Lane: tests/gates
coverage lifts; poison-fixture asserts per path; no floor change
without operator exception receipts. Scope L.

### A10 (P1) G2-PARETO L1/L2/L4/L5 shell courts (R4)
Rows are exit-0 control shells, not threshold measurements. Lane: bench
receipts lane; per level: run bench then /tmp-mutate the kernel
(remove #[inline] / add black_box) -> threshold must flip FAIL;
sha-exact restore; no flip = [SKIP] loud. L3 stands CLOSED (repair-06).
Scope L.

### A11 (P2) retraction throughput vs 800M gate (R1)
b05 min 248M; GIT-11 323.8M MEASURED-UNGATED; dred alt-path 839-944M
ineligible (tuning = vacuous by its own court law). Honest FAIL stands
until repair or operator gate-change receipt. Do NOT round up.
Scope L / S.

### A12 (P2) F5/F10/WG2/WG3 receipts (R7)
Errata lane mapping each workstream to covering receipts; UNVERIFIED
does not round up. Scope S.

### A13 (P2) honesty headers + ungrounded refs
LSPA-01 receipt "TO BE FILLED" (cannot be cited ALIVE); abstract-slot
ungrounded. Lane: doc-honesty; receipt_lint RECEIPT-STRUCT-002 firing
on the unfilled receipt IS the flip. Scope S.

### A14 (P3) hygiene bundle
R8 debris (libae8_dbg.rmeta, wrapper_riscv32imc.o) delete-never-commit;
R9 CLOSED, R11 refreshed by MV-8; /tmp/exp04-target owed. Scope S.

## B. Wire -> substrate E2E court (new lane E2E-1)

Component courts all exist; the SEAM has no court. Spec:
tests/e2e_wire_substrate_court.rs (no src edits; needs A4 first).
Path: mcp_driver Driver::dispatch/parse_frame (Latin-1 grammar) ->
admission_gate RefUpdate -> octopus_merge on >=3 heads (typed refusals
on poisoned head) -> commit_synth LeaseCommitSpec::render + seal ->
blake3 digest + 80-byte conformance seal shape -> ALL inside one
CountingAlloc window, window_counts()==(0,0).
Four mandatory flips: F1 frame byte corrupt -> ProtocolError; F2 gate
violation -> admission refuses, downstream never runs (step-count
probe 0); F3 poisoned head -> MergeError ANDON; F4 trailer/digest
mutate -> commit validation refuses + re-derivation mismatch.
Scope L (S-M if A4 lands first).

## C. Doc-gate rule class (extends DX-4)

DOC-RECEIPT-001: quantitative claim without a receipt ref in the
proximity window (non-mechanical parse+ref check). DOC-RECEIPT-002:
cited receipt path does not resolve (mechanical). DOC-RECEIPT-003:
missing honesty-header block (status vocab + as-of date + inputs).
DOC-RECEIPT-004: ship-gate row claims covered with no ALIVE receipt.
Flip: /tmp fixture "3.4x faster" with no ref -> 001 fires exit 1;
add ref -> exit 0. Scope: shipped narratives (audit/waves), lanes keep
RECEIPT-STRUCT-*.

## D. The plan's falsifier — ship-gate table

Observable: every pillar maps to >=1 ALIVE receipt and the
ASPIRATIONAL-UNVERIFIED column is empty. Gate rule: the narrative
ships only when all seven rows read COVERED with dated command+exit
receipts. The 7-pillar decomposition is PROPOSED (no prior pillar
canon found on disk).

| pillar | status | aspirational residue |
|---|---|---|
| P1 anti-vacuity courts | MOSTLY COVERED | A2, A5 |
| P2 measured perf under mutation | OPEN — L1/L2/L4/L5 shells | whole pillar until A10 |
| P3 fail-closed gates | MOSTLY COVERED | A9 |
| P4 purity (no_std/zero-alloc) | MOSTLY COVERED | A4 |
| P5 wire/interop seams | PARTIAL — E2E seam court ABSENT | B/E2E |
| P6 packaging/provenance | PARTIAL | A12, A13 |
| P7 doc honesty | OPEN — 445 advisory, C6 p99 | A3, A8 |
| P9 PRD-as-spec substrate | COVERED — receipt-true PRD v2 landed (audit/cgapprd-prd-v2.md), PRD Verify exit 0 this session (every quantitative claim carries a receipt file:line; zero claims use a referent that fails an in-tree grep); Σ_PQC equation, <2 ms closure gate, and sub-ms checkpointing deleted per cgapprd-09 plan | cgapprd-10 footer: none residual — aspirational rows registered in cgapprd-prd-v2.md §8 with unblock prerequisites |

Verdict: doctrine NOT shippable today. Critical path: A3, A2, A4, B;
perf honesty path A10/A11; rest parallelize.

## See Also

audit/release-blockers.md · audit/audit-ledger.md ·
audit/doctrine-friction-census.md
