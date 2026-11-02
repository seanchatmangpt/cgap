# TPS-6 — Kanban ledger (re-derived from lanes/ receipts, 2026-11-2)

Census basis: `ls docs/jira/v26.11.2/lanes/` run this session. DONE =
receipt landed with ALIVE standing; WIP = lane in flight, no receipt;
RE-SCOPED/ROADMAP = aspirational rows. Every row cites receipt file:line
re-read from disk this session. The dispatched "TPS manual kanban draft"
is NOT on disk anywhere in the repo (repo-wide grep for "kanban" = 0
hits) — doc yields to reality; this file IS the re-derived kanban.

## A. Stale-row corrections (the dispatched corrections)

| Item | Old draft row | Corrected | Receipt cite |
|---|---|---|---|
| GAP-10 | (draft implied drained) | **WIP** — no `lane-gap-10*.md` on disk; gap-01..09,11,12 present | census in lanes/lane-tps-06-kanban.md §3 |
| LSPA-3 | (stale) | **DONE (PARTIAL standing)** — court landed; arm (ii)/(iii) + 4 prior courts ALIVE; arm (i) REFUTED-by-witness, root cause rule_pack.rs:133-166/:125 handed to owning lane | lanes/lane-lspa-03-attr-witness.md:103-112 |
| AX-8 | "ABSENT on disk" | **DONE — ALIVE** in-container sealer SL-001..004, all 6 runs through seal.sh; sandbox_class declared-by-caller, recorded divergence | lanes/lane-ax-08-seal.md:70-76 |
| AX-9 | (stale) | **DONE (refusal witness)** — end-to-end UNVERIFIED-by-design, stated plainly; suspend_probe.sh landed (bash -n 0, run exit 3 REFUSED[PREREQ-ABSENT]); ax CLI BLOCKED (install NOT APPROVED) | lanes/lane-ax-09-suspend.md:58-65 |
| BLAKE3 Sealer 4.35M | "4.35M seals/s" (bare) | **DONE with load caveat** — 4.35M/s is the UNCONTENDED release-build run median on one box; contended run exit 1 (typed FAIL); debug 204k/s (< floor by design); gate is the 1M/s floor | lanes/lane-b06-cscb-c7.md:49-53; audit/doctrine-sram-and-proofs.md:50-51,69; audit/audit-ledger.md:43 |

## B. Roadmap / re-scoped rows (aspirational, never gate)

| Item | Status | Evidence state | Cite |
|---|---|---|---|
| 2 ms fixpoint closure | ROADMAP | ASPIRATIONAL-UNVERIFIED; no kernel path to 2 ms on the C3 corpus (min measured ~20,782x target); PRD does not pin corpus/size/shape; reported loudly as PRD_ASPIRATIONAL_MS, never gates | audit/cgapprd-07-nfr02-gate.md:70-79 |
| C-ABI batch ingress | ROADMAP | closest witness POSH-2 batch court PARTIAL: 23.35 ns median vs 10.0 ns amortized limit = honest GATE FAIL; per-fact number real, limit unmet | lanes/lane-posh-02-batch.md:107-112 |
| AX cluster | ROADMAP | AX-0 family gate CLUSTER NO-GO (conn-refused, exit 1); ax CLI 127; install NOT APPROVED; LOCAL docker GO | lanes/lane-ax-10-index.md:41 (AX-0 row) |
| Sigma_PQC purged | RE-SCOPED | "mod Sigma_PQC" is invented lore — no in-tree referent; purge = doc rewrite/tombstone plan, NOT executed | audit/cull-08-doc-retirement.md:9-12,76-80; audit/cgapprd-09-prd-rewrite-plan.md:54 |

## C. Ledger cross-check (no double registration)

audit/audit-ledger.md has ZERO rows for the GAP-10, LSPA-3, AX-8, AX-9
families (grep LSPA|AX-|GAP- over the ledger = 0 hits this session).
So this kanban is the sole register for those families; B06 is the one
shared subject and it points at the ledger row (audit-ledger.md:43)
rather than duplicating a row. No double registration.

## D. Divergence recorded (errata against a prior receipt, ERRATA law)

cull-07-ipc-legacy.md:8-9 states "AX-8 receipt: ABSENT on disk (no
lane-ax-08*)". That was true at its read time and is stale now:
`lanes/lane-ax-08-seal.md` is on disk (95 lines) with ALIVE standing.
Per ERRATA-never-rewrite law, cull-07 is NOT edited; this section is
the errata note citing cull-07:8-9.
