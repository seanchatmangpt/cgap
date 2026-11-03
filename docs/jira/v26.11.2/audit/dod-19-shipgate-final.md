# DOD-19 — final ship-gate table spec (P1-P10 + PUB)

Honesty header — status vocab: COVERED / COVERED-LOCAL / DRAFT-UNRUN /
OPEN / PARTIAL / UNVERIFIED never rounds up. As-of 2026-11-02. Inputs
of record re-read fresh this session: doctrine-ship-plan.md (§D table
+ landed P9 row), tc-10-open-recensus.md, tps-08-shipgate-plan.md
(P8/P9/P10 cell criteria), pub-02-package-dryrun.md (GO/NO-GO matrix),
lanes/lane-ax-00-gate.md, lanes/lane-ax-08-seal.md (LANED since TPS-8
was written), lanes/lane-cgap-prd-10-landing.md (LANED),
lanes/lane-gap-08-e2e-court.md (LANED), lanes/lane-gap-07-g2-pareto.md,
lanes/lane-gap-02-blake3-tamper.md, lanes/lane-gap-04-zero-tax.md,
lanes/lane-gap-05-receipt-lint.md, lanes/lane-gap-06-coverage.md,
lanes/lane-tc-14-ax-rewitness.md, audit/doctrine-commercial-proofs.md,
audit/workstream-receipt-map.md, waves/publish-halt.md.
NOT on disk at spec time: dod-12-g2-decision.md (DOD-12 UNLANDED);
lanes/lane-dod-05*.md, lanes/lane-dod-07*.md (UNLANDED).
This file is a SPEC: the exact cell texts below are what the
coordinator's final table-update commit writes into
doctrine-ship-plan.md §D. Nothing here is measured by DOD-19 itself.

## 0. Table shape

§D becomes 10 pillar rows (P1-P10) plus the PUB block. TPS-8 §3's
"7-pillar plus P8/P9/P10" is the shape of record; P9 already sits in
the table (PRD-10 landed it append-only after P7). DOD-19's commit
inserts P8 after P9 and P10 after P8, append-only, zero pre-existing
lines modified (PRD-10's insert-only relocation is the precedent).

## 1. Per-row spec (status / exact cell text / residue)

### P1 anti-vacuity courts

- Landed evidence: GAP-02 chain-mutation court ALIVE
  (crates/cgap-blake3/tests/chain_mutation_court.rs, 200 lines;
  lib 3 passed + court 4/4, flips witnessed, lane-gap-02:23-30)
  closes ship-plan A5. Zero-tax flip DETERMINISTIC ALIVE
  (lane-gap-04:77-79); doctrine-commercial-proofs.md:87 records
  "R10 flip court FLIPS; R10 is RESOLVED as-of-today" — but
  tc-10 §2 records the MV-5 append (release-blockers:248-255)
  narrowing it (C1/C2/C4-runner/C5/C7/C8 ignore zero-tax). The
  conflict is coordinator-owned, not re-litigated here.
- Status: COVERED (residual: R10 append conflict).
- Exact cell text (status column):
  `COVERED — GAP-02 court 4/4 + flips (lane-gap-02:23-30);
  zero-tax flip deterministic (lane-gap-04:77-79)`
- Residual ASPIRATIONAL-UNVERIFIED cell:
  `MV-5 vs GAP-04 append conflict (tc-10 §2); unblock:
  coordinator conflict-resolution append — one register row
  of record, later timestamp wins (MV-5 precedent)`
- Note: A2's "mechanical env-set both directions" is witnessed;
  tc-10's narrowed reading (some children ignore zero-tax) keeps
  the residue loud until the coordinator append lands.

### P2 measured perf under mutation

- Landed evidence: GAP-07 rollup (lane-gap-07:156-166): L1 MET but
  VACUOUS per gate law (8 flip attempts, transcripted); L2 NOT MET
  23.13 ms vs <10 ms; L3 CLOSED (repair-06, cited); L4 PARTIAL
  (MET@256 vacuous-per-law / NOT MET@4096 9.35 us/tx); L5 NOT MET
  3.055 us/row, flip witnessed +8.3%.
- Status: OPEN.
- Exact cell text (status column):
  `OPEN — L1 VACUOUS, L2/L5 NOT MET, L4 PARTIAL (GAP-07 rollup,
  lane-gap-07:156-166)`
- Residual cell:
  `whole pillar until DOD-12 (R4 decision) lands; unblock:
  DOD-12 rules repair-to-threshold with flips OR operator
  gate-change receipts + divan2receipt successor (chains-*
  parse exit 1) + L1 flip court. Do NOT round up.`
- DOD-12 dependency: dod-12-g2-decision.md is ABSENT on disk.
  P2 cannot flip to COVERED before DOD-12 lands either verdict.
  This is the plan's hard perf-honesty gate (ship-plan §D verdict
  "critical path A10/A11" resolves here).

### P3 fail-closed gates

- Landed evidence: DX-4 strict rules landed; GAP-05 DOC-RECEIPT
  census 001=2324 / 002=81 / 003=145, advisory exit 0
  (lane-gap-05:76); GAP-06 coverage floor PARTIAL, honest red
  preserved, floor NOT lowered (lane-gap-06:3, "PARTIAL").
- Status: MOSTLY COVERED.
- Exact cell text:
  `MOSTLY COVERED — DX-4 strict landed; coverage floor honest
  red preserved (GAP-06 PARTIAL); DOC-RECEIPT census advisory`
- Residual cell:
  `A9 scripts coverage red (24.4/64.9/52.6/40.8% vs 90);
  unblock: per-script >=90% branch lifts or operator exception
  receipts; R5 close = census 0 or triage + fail-closed floor
  in make gate0 (tc-10 §2 R5).`

### P4 purity (no_std / zero-alloc)

- Landed evidence: GAP-01 mounted object_store
  (crates/cgap-git/src/lib.rs:155 is now `pub mod object_store;`;
  lane-gap-01 "ALIVE"); E2E court excludes it (lane-gap-08 W7)
  and requires a header update now that the mount landed.
- Status: MOSTLY COVERED.
- Exact cell text:
  `MOSTLY COVERED — object_store mount landed (lib.rs:155,
  GAP-01 ALIVE); A4 digest-fn injection NOT evidenced on disk`
- Residual cell:
  `A4 WIRE-PENDING(lane-git-05): object_store hard-codes
  cgap_blake3::hash with dev-dep-only; unblock: chartered
  digest-fn injection lane (tree_synth precedent) with
  constant-digest -> store-rejects flip; plus e2e court
  header update for the object_store exclusion (gap-08 W7).`

### P5 wire/interop seams

- Landed evidence: lane-gap-08 ALIVE —
  crates/cgap-git/tests/e2e_wire_substrate_court.rs (668 lines),
  cargo test exit 0 (5 passed) 8/8 stable; four flips F1-F4
  witnessed (ProtocolError / GateError + downstream probe 0 /
  MergeError ANDON / ReceiptTagMismatch); window_counts
  divergence disclosed (byte-balanced, not literal (0,0)).
- Status: COVERED (with named divergences carried).
- Exact cell text:
  `COVERED — E2E wire->substrate court 5 passed 8/8, F1-F4
  flips witnessed (lane-gap-08 W3-W4); window byte-balanced
  divergence named (gap-08 W5)`
- Residual cell:
  `object_store excluded from the court (gap-08 W7); unblock:
  court-header update on the landed mount; NEW-1 TC-04
  confirms/repairs the claimed octopus :366 residual; the two
  PRE-EXISTING cgap-git flakes (gap-08 W6) stay loud, owner
  = coordinator handoff.`

### P6 packaging/provenance

- Landed evidence: PUB-02 matrix (pub-02:24-48): 17 GO, 2 NO-GO
  (cgap exit 101 — cgap-branchless absent from crates.io;
  cgap-gate exit 101 — workspace dep has no version key),
  1 non-member GO (wasm-footprint); FIX-01 license holds —
  zero warnings across all 20 dry-runs (pub-02:117-119).
  DOD-01 landed: cgap-core exit 0 (13 binaries, 79 tests),
  court flips 101 on license-file re-add (lane-dod-01-spdx).
- Status: PARTIAL.
- Exact cell text:
  `PARTIAL — PUB-02 17 GO / 2 NO-GO (cgap, cgap-gate; ONE
  root cause: unpublishable cgap-branchless dep); FIX-01
  license holds 0 warnings; DOD-01 cgap-core green`
- Residual cell (PUB row, see section 3 for the full matrix):
  `cgap + cgap-gate re-dry-run clean after the pub-02 exact
  blocking edits (optional feature-gated dep or drop; fix at
  root [workspace.dependencies]); operator publish-freeze lift
  (waves/publish-halt.md, FROZEN-BY-OPERATOR); F10 receipt or
  coordinator errata retirement (R7).`

### P7 doc honesty

- Landed evidence: R5/NEW-2 open — DOC-RECEIPT-001 precision
  debt 2324 findings recall-first; A3 sibling NOT fixed
  (doctrine-commercial-proofs.md:64, S9 OPEN: c6 max-of-7
  labeled p99, undisclosed; C3 discloses, C6 does not).
- Status: OPEN.
- Exact cell text:
  `OPEN — DOC-RECEIPT census 2324/81/145 advisory exit 0
  (GAP-05:76); C6 p99 mislabel NOT applied (commercial-proofs
  S9 OPEN)`
- Residual cell:
  `A3 unowned sibling; unblock: FIX-04-clone lane renames
  min/median/max/N in cscb_c6_retraction.rs:87-100 +
  forbidden-label grep court; NEW-2 TC-09/TC-12 triage to
  residual-or-0 then fail-closed floor. Close = R5 close.`

### P8 (AX substrate) — TPS-8 cell criteria apply

- Landed evidence: AX-0 verdict table (lane-ax-00 W2-W3):
  CLUSTER PATH NO-GO (kubectl conn-refused, ax exit 127),
  LOCAL PATH GO (docker 28.0.4 exit 0). AX-8 LANED ALIVE
  (lane-ax-08 W6: SL-001..004 emit + refusals in-container,
  sandbox_class declared-not-probed divergence named) —
  SL-004 is no longer RECEIPT-PENDING; TPS-8 §1's PENDING-AX-8
  branch is retired, cite lane-ax-08-seal.md. AX-9 stays
  UNVERIFIED (NO-GO, probe landed, nothing simulated —
  lane-ax-09:8-11 / tc-10 NEW-4). TC-14 legs pending: the
  composed image has an in-image arm defect (row 2: mixed-arch
  -m64 gcc failure; lane-tc-14 W2) — NEW-3 not closed.
- Status: COVERED-LOCAL / DRAFT-UNRUN (never plain COVERED).
- Exact cell text (status column):
  `COVERED-LOCAL / DRAFT-UNRUN — per AX-0 verdict (local GO,
  cluster NO-GO); SL-004 covered by AX-8 (lane-ax-08-seal,
  ALIVE); AX-9 UNVERIFIED; TC-14 in-image legs pending`
- Residual cells (ASPIRATIONAL-UNVERIFIED, loud forever
  unless operator lifts):
  - `AX-9 cluster-bound: unblock = operator unblocks cluster;
    otherwise stays permanently loud-aspirational (tc-10
    NEW-4).`
  - `DRAFT-UNRUN: AX-2 manifests drafted, never applied; any
    apply claim is hearsay per AX-0. unblock: chartered
    apply lane post-freeze-lift only.`
  - `TC-14 legs: unblock = arch-pure image rebuild + in-image
    court re-witness receipt (NEW-3 close).`

### P9 (PRD substrate) — COVERED per PRD-10

- Landed evidence: lane-cgap-prd-10-landing.md —
  cgapprd-prd-v2.md (272 lines) landed; PRD Verify (referent
  grep battery) PRD_VERIFY_FAIL=0 exit 0 this session;
  deletions register SS7 (Sigma_PQC, sub-ms checkpointing,
  <2 ms closure gate) landed; aspirational register SS8 with
  named unblocks landed; doctrine-ship-plan.md P9 row inserted
  append-only after P7.
- Status: COVERED.
- Exact cell text (already in §D, carried verbatim — do not
  rewrite):
  `COVERED — receipt-true PRD v2 landed (audit/cgapprd-prd-v2.md),
  PRD Verify exit 0 this session` [+ the existing cell tail]
- Residual cell: none residual — aspirational rows live in
  cgapprd-prd-v2.md §8 with unblock prerequisites (the landed
  P9 row's own aspirational column already says "none residual").
  Carry as-is.

### P10 (TPS enclosure) — per TPS-8's four criteria

- Landed evidence toward the four criteria: (a) SL-003 sealing
  capability ALIVE (AX-8) but enclosure receipts NOT yet sealed
  this session; (b) zero cluster-apply claims — holds today per
  AX-0 NO-GO + DRAFT-UNRUN; (c) ledger stitch NOT yet appended
  (audit-ledger.md tail has no AX-0..10/IPC-6/PRD-10/TPS-8
  enclosure rows in the TPS-8 §2 order); (d) claim-ceiling
  paragraph not yet present verbatim in an enclosure artifact.
- Status: DRAFT (OPEN until all four hold simultaneously).
- Exact cell text (status column):
  `OPEN/DRAFT — enclosure not yet run; criteria (a)-(d) tracked
  in tps-08-shipgate-plan.md §3; flips COVERED only when all
  four hold simultaneously`
- Residual cell:
  `unblock: TPS-8 enclosure execution — seal each enclosure
  receipt per TPS-8 §1 (SL-001..004; cite lane-ax-08, no
  PENDING-AX-8 mark), ledger stitch per §2 (AX-0 first, then
  AX-1..10, IPC-6, PRD-10, TPS-8 last), claim-ceiling
  paragraph §4 verbatim, falsifier §5 pass. Any single
  failure -> P10 OPEN; narrative does not ship.`

## 2. Table verdict cell (§D footer)

Exact text for the verdict line under the table:

    Verdict: doctrine NOT shippable today. COVERED rows: P1, P5,
    P9 (+ P8 COVERED-LOCAL). OPEN: P2 (DOD-12 dependency), P7
    (R5/A3), P10 (enclosure unrun). PARTIAL: P6 (PUB-02 NO-GOs
    + freeze). MOSTLY COVERED: P3, P4. Critical path: DOD-12 ->
    P2, TPS-8 enclosure -> P10, pub-02 edits -> P6, A3 -> P7.

## 3. PUB rows (PUB-02 matrix state, verbatim-mapped)

| crate | verdict | residual / unblock |
|---|---|---|
| cgap | NO-GO exit 101 | unblock: pub-02 NO-GO 1 edit (a) optional |
| cgap |  | feature-gated dep, (b) drop dep (verify lib.rs), or |
| cgap |  | (c) flip branchless publish (not preferred) |
| cgap-gate | NO-GO exit 101 | unblock: fix at ROOT [workspace.dependencies] |
| cgap-gate |  | version key + same (a)/(b) as cgap — member-level |
| cgap-gate |  | override unsupported in stable cargo (pub-02:89-92) |
| 17 crates | GO | no residual |
| wasm-footprint | GO (non-member) | disclose out-of-scope member sequencing |
| license | FIX-01 holds | zero warnings, 20 dry-runs (pub-02:117) |
| freeze | FROZEN-BY-OPERATOR | hard block; lift is operator-only |
| wave | debris flags | crypto wave7 shim; wasm NOTES; readme-archive |
|  |  | disclose, not blocking (pub-02:94-107) |

The PUB block goes under the pillar table as a second table,
introduced by one line: `PUB matrix (pub-02, packaging only,
publishes FROZEN):`. The gate is NOT all-GO until cgap and
cgap-gate re-dry-run clean (pub-02 verdict, verbatim).

## 4. Cross-lane closure dependencies (named, with disk state)

- DOD-12 (R4 decision): UNLANDED — dod-12-g2-decision.md absent.
  Blocks P2 and the §D verdict's critical path. Unblock named
  in the P2 residual cell; either verdict unblocks (repair
  path or gate-change path), neither is assumed.
- DOD-01 (SPDX court): LANDED ALIVE — lane-dod-01-spdx.md;
  cgap-core exit 0, anti-vacuity mutation flips 101.
  P6 carries it; no open dependency.
- DOD-05: UNLANDED — no lanes/lane-dod-05*.md on disk. P6
  provenance residue keeps a named dependency: closure requires
  the DOD-05 receipt or an explicit retirement append.
- DOD-07: UNLANDED — no lanes/lane-dod-07*.md on disk. Same
  law as DOD-05: closure requires the receipt or retirement
  append; absent = PENDING (the ledger's own rule, TPS-8 §2).
- R10 append conflict (GAP-04:81 vs release-blockers:248-255):
  coordinator-owned; later-timestamp-wins per tc-10 §2; P1's
  COVERED stands on the flip witness, the conflict stays loud
  in P1's residual cell.
- NEW-5 (POSH-2 23.35 ns gate decision): honest typed FAIL
  (lane-posh-02:43,111). Not a pillar row of its own; feeds P2
  class. Close = operator gate-change receipt (R1 precedent)
  or repair to <=10 ns with flip court. Do NOT round up.
- Operator publish freeze (waves/publish-halt.md): HARD BLOCK,
  non-code. P6/PUB closure requires operator lift; already-live
  crates are immutable; 3 already-live repairs owed on lift.

## 5. Falsifier

DOD-19's spec is correct iff every exact cell text in section 1
is writable into doctrine-ship-plan.md §D without modifying any
pre-existing line (append/insert-only), every status claim above
cites a landed receipt path (+line where load-bearing), and the
DOD-12 / DOD-05 / DOD-07 / AX-9 / freeze rows are stated as
UNLANDED or blocked with named unblocks rather than rounded up.

## See Also

audit/doctrine-ship-plan.md · audit/tps-08-shipgate-plan.md ·
audit/tc-10-open-recensus.md · audit/pub-02-package-dryrun.md ·
audit/audit-ledger.md
