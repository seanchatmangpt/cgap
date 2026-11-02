# TPS-8 — Station 400 ship-gate enclosure plan (2026-11-02)

PLAN lane. Deliverable is this file; no code, no gates, no lane receipts
are edited by TPS-8 itself. Inputs of record re-read this session:
audit/doctrine-ship-plan.md (fresh, ship-gate table + P8 row state),
lanes/lane-ipc-06-sealer.md, courts/interop-chicago/cmd/seal_receipt/main.go
(SL-001..004 present in source), lanes/lane-ax-00-gate.md (AX verdicts),
audit-ledger.md. NOT on disk at plan time: lanes/lane-ax-08-seal.md
(AX-8 NOT landed), lanes/lane-cgap-prd-10-landing.md (PRD-10 NOT landed).
Every dependency below is conditioned on its landing, not assumed.

## 0. Station 400 position

Station 400 is the final enclosure before the v26.11.2 narrative ships.
It consumes three landings and produces one artifact: a ship-gate table
whose every row cites a dated, this-session command+exit receipt, plus
an audit-ledger stitch and a claim-ceiling paragraph that the DOC-RECEIPT
rule class can mechanically police.

## 1. Receipt sealing via seal_receipt (SL-001..004)

- Subject: courts/interop-chicago/cmd/seal_receipt/ (main.go +
  receipt-template.md, sole code file of the sealer, IPC-6 ALIVE).
- Gate set at sealing time:
  - SL-001: every quantitative bullet carries a method disclosure
    (batch-amortized | machine-dependent | load-sensitive).
  - SL-002: every bullet cites a receipt path.
  - SL-003: every asserted number has a this-session command+exit in
    its input; a CTQ name with no row also refuses.
  - SL-004 (AX-8 where-it-ran): the receipt records WHERE it ran —
    image_digest (sha256: + 64 hex), ax_runner_version, sandbox_class
    (gVisor|runc). Receipt-level gate, not per-bullet. STATUS: the
    SL-004 code is LANDED in main.go, but its owning lane receipt
    lanes/lane-ax-08-seal.md is ABSENT on disk. Therefore SL-004 is
    LANDED-CODE / RECEIPT-PENDING. Station 400 may not cite an AX-8
    receipt that does not exist; if AX-8 has not landed by enclosure
    time, SL-004 stays in the seal set with its status marked
    PENDING-AX-8 and its three flags (-ax-image-digest,
    -ax-runner-version, -ax-sandbox-class, or AX_* env) supplied from
    the LOCAL docker run that performed the sealing, sandbox_class
    "runc" with the AX-0 local-exec ruling as basis. If AX-8 lands
    first, cite it and drop the PENDING-AX-8 mark.
- Sealing procedure per lane receipt in the enclosure set:
  1. Build once: go build -o /tmp/tps8-seal_receipt main.go (exit 0
     required; go vet clean per IPC-6).
  2. Per receipt: construct input JSON from the receipt's own
     commands+exits; run sealer; exit 0 = EMITTED (sha256 transcript
     root over trimmed bytes), exit 1 = REFUSED SL-00x lines on
     stderr, no file written. A refusal is a finding, not a failure
     of the enclosure: fix the bullet (add method / receipt ref /
     exit), never the gate.
  3. Anti-vacuity replay: re-run the IPC-6 bad-fixture refusal
     (/tmp/ipc6-sandbox/bad.json -> exit 1, 3 SL lines) in the same
     session as any green emit. A green run without the witnessed
     refusal is VACUOUS by IPC-6's own court table.
  4. Every emitted receipt records sha256 (not BLAKE3) as transcript
     root — the IPC-6 divergence, named in the artifact, never faked.

## 2. Audit-ledger stitch semantics

- Target: audit/audit-ledger.md — append-only. Existing rows are
  never edited or re-derived; the ledger's own header law ("one row
  per landed receipt; standing as the receipt claims") is preserved.
- One new row per LANDED lane in the enclosure set, keyed by lane id
  in the existing column shape (Lane | Standing | load-bearing
  verified fact (command + exit) | OPEN item).
- Append order: AX-0 first (gate), then AX-1..10 numerically, then
  IPC-6, then PRD-10 if landed, then TPS-8 last.
- Row content law: Standing copied verbatim from the lane receipt's
  own Standing section (ALIVE / PARTIAL / PENDING / NO-GO as claimed,
  never rounded up); load-bearing fact = one command + its exit, taken
  from the receipt body; OPEN item = the receipt's stated OPEN item,
  or "(none stated)".
- Lanes NOT landed at stitch time get NO row (absent = PENDING is the
  ledger's own rule; do not fabricate rows for AX-8 or PRD-10 until
  their receipts exist on disk).
- The stitch itself is sealed with seal_receipt (SL-001..004) like
  any other receipt, and the stitch run's command+exit is itself a
  ledger row (TPS-8, appended last, self-referential row permitted
  as the ledger's closing row).

## 3. Ship-gate table update spec

Amends audit/doctrine-ship-plan.md section D. The table stays 7-pillar
plus the NEW P8/P9/P10 substrate rows as follows (cell criteria, not
prose):

| row | substrate | COVERED cell criteria (short) | basis receipt |
|---|---|---|---|
| P8 (AX substrate) | AX-1..10 local-exec courts | see P8 criteria below | lane-ax-00-gate.md W4 |
| P9 (PRD substrate) | cgap-prd ladder | see P9 criteria below | lane-cgap-prd-10-landing.md |
| P10 (TPS enclosure) | Station 400 closure | see P10 criteria below | this plan + sealed receipts |

P8 cell criteria (COVERED-LOCAL / DRAFT-UNRUN):
- COVERED-LOCAL: every AX lane that ran cites a docker-run
  command+exit under AX-0's local-exec GO ruling.
- DRAFT-UNRUN: AX-2 manifests drafted, never applied; any apply
  claim is hearsay per AX-0.
- UNVERIFIED: AX-9 (cluster-bound; ax exit 127, cluster exit 1).
- Never reads plain COVERED; the local qualifier stays loud.

P9 cell criteria:
- COVERED iff the PRD-10 landing receipt exists on disk with a
  this-session command+exit, and its OPEN items are restated, not
  resolved. Receipt absent -> P9 stays OPEN (state at plan time).

P10 cell criteria (all four must hold simultaneously):
- (a) every sealed receipt cites a this-session command+exit
  (SL-003), (b) zero cluster-apply claims survive AX-0's NO-GO,
  (c) ledger stitch has one row per landed lane, (d) the
  claim-ceiling paragraph is present verbatim.
- Any single failure -> P10 OPEN; narrative does not ship.

- P8 status at plan time: the AX-0 verdict table says CLUSTER PATH =
  NO-GO (kubectl exit 1 conn-refused), LOCAL PATH = GO (docker 28.0.4
  exit 0). P8 therefore reads COVERED-LOCAL/DRAFT-UNRUN per AX-0's
  verdict — never COVERED outright. AX-9 stays UNVERIFIED.
- P9 status at plan time: PRD-10 receipt ABSENT -> P9 stays OPEN;
  it flips to COVERED only on PRD-10's landing with receipts.
- P10 starts DRAFT and flips to COVERED only when all four cell
  criteria hold simultaneously; any single failure -> P10 stays
  OPEN and the narrative does not ship (doctrine section D gate
  rule: ship only when every row reads COVERED — P10's COVERED-LOCAL
  reading satisfies the gate rule via the local-exec qualification,
  P8's DRAFT-UNRUN cells stay loud in the aspirational column).

## 4. Claim-ceiling paragraph (verbatim in the enclosure)

What v26.11.2 MAY assert (each with its receipt of record):

- 351x gross arbitrage — only as gross ratio 351.99 vs floor 350.0,
  drowning_benchmark 4/4 courts + zeroed-spark flip
  (doctrine-commercial-proofs.md). "352x" is not the pinned form.
- 36,048 B SRAM — measured gate PASS (verify_cscb_sram.py exit 0,
  mutant 136,068 B flips). "39,184 -> 36,048 improvement" is a
  category error and stays narrative-only.
- ~3.95-4.35M seals/s — median of clean release runs, machine-
  dependent, load-caveated; the gate is the 1M floor, not the median
  (doctrine-sram-and-proofs.md; B06 medians 4.35M are run medians,
  never "4.0M" or a bare "4.35M").
- 22.99 ms — witness latency, EXP-03, NOT a gate
  (dflss/02-voc-ctq-matrix.md:109-111); must never be cited as a
  gate pass.
- container 1.7-1.8x delta — published per AX-5's delta table
  (~1.8x cgo floor, ~1.7x T_kernel), machine-dependent, published
  as delta not absolute.
- CSCB ladder baseline exit 0; C6 "hundreds of millions" (242-248M
  facts/s full-cascade, gate decided at 150M) with the loud 3.31x
  800M-shortfall line carried wherever 800M appears.

What stays ASPIRATIONAL (no v26.11.2 claim may assert these):

- 2 ms closure (C3 decided budget is 60,000 ms; 25 ms aspiration
  not met, 32x over, loud).
- 800M facts/s retraction (measured 242-324M band; honest C6 FAIL
  vs 800M stands; do not round up).
- Sigma_PQC-purged (no receipt; ungrounded referent).
- sub-ms checkpoint (no receipt; PRD substrate, PRD-10 absent).
- cluster-apply (AX-0 NO-GO: kubectl exit 1, ax exit 127,
  install unapproved; zero apply witnesses may exist).

## 5. Falsifier (verbatim in the enclosure)

the milestone Verify passes iff every sealed receipt cites a
this-session command+exit and zero cluster-apply claims survive
AX-0's NO-GO.

## 6. Sequencing

1. Re-check disk for lanes/lane-ax-08-seal.md and
   lanes/lane-cgap-prd-10-landing.md (fresh, at enclosure time).
2. If AX-8 landed: cite it, SL-004 fully covered. If not: SL-004
   stays PENDING-AX-8 with local-run SL-004 flags from the sealing
   docker run (AX-0 local-exec basis).
3. If PRD-10 landed: P9 -> COVERED per its receipt. If not: P9 OPEN.
4. Seal each enclosure receipt (section 1 procedure).
5. Ledger stitch (section 2), TPS-8 row last.
6. Ship-gate table update (section 3) — P8/P9/P10 per their cell
   criteria.
7. Verify runs the section-5 falsifier; pass -> narrative ships.

## See Also

audit/doctrine-ship-plan.md · lanes/lane-ipc-06-sealer.md ·
lanes/lane-ax-00-gate.md · audit/audit-ledger.md ·
audit/doctrine-commercial-proofs.md ·
audit/doctrine-sram-and-proofs.md ·
dflss/01-project-charter-ax-testbed.md
