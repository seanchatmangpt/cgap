# Lane DFX-3 — AX-Fleet PFMEA Receipt

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical
checkout). No git commands run.

## W1 Identity

- Lane: DFX-3. Owned files (both new):
  - docs/jira/v26.11.2/dflss/08-ax-pfmea.md
    (sha256 fea2fa055bfd9b1b[...], 92 lines)
  - docs/jira/v26.11.2/lanes/lane-dfx-03-pfmea.md (this receipt)
- Subject: AX/interop-fleet PFMEA (08), superseding the AX-relevant
  rows of 04-pfmea-risk-matrix.md.

## W2 Authority

Dispatch prompt: re-score DET from actual witnessed evidence; keep
draft rows only where the detection witness exists; add rows (a)-(e)
earned this session; score RPN=SEV x OCC x DET, rank, flag top-3 for
DFX-6. Owned files only; read-only everything else.

## W3 Commands + exits (this session)

| command | exit | result |
|---|---|---|
| ls dflss/ + lanes/ | 0 | confirmed no draft 08 existed; 04 is the draft matrix |
| Read 04-pfmea-risk-matrix.md, lane-ax-00-gate.md, lane-g05, lane-gap-05, dod-lsp-max-wire.md, dxd-03-go-ergonomics.md | 0 | evidence table built from on-disk receipts, re-read this session |
| grep VENDOR-CLAIM-UNVERIFIED (docs/jira/v26.11.2) | 0, ZERO hits | A2 DET scored manual/high honestly |
| grep DOC-RECEIPT-00[23] | 0 | landed rules located (lane-gap-05-receipt-lint.md:48-50,62-76) |
| ls lanes | grep -i "ax\|dxd" | 0 | lane-ax-03 absent -> A1/A3 container courts PENDING |
| python3 RPN arithmetic check over 08 | 0 | 10/10 rows OK (A1 252, A2 224, A3 224, A4 200, A5 60, C1 84, C2 70, C3 48, C4 42, C5 32); sha256 fea2fa055bfd9b1b |

## W4 Consequence

08-ax-pfmea.md lands with: evidence-citation table (6 rows, all
path:line grounded); 5 new AX rows (A1-A5) + 5 carried rows from 04
(kept only where the detection witness exists); dropped-rows table
giving the honest disposition of 04 R6/R8/R10; top-3 flagged for
DFX-6 (A1 stale-binary 252; A2/A3 tie 224; A4 hearsay 200); named
anti-vacuity mutation (delete any DET cite -> RPN rises, ranking
reshuffles). Honest-high DETs: A1 container channel DET=7 (court
PENDING), A2 DET=8 (manual marking only), A3 DET=7 (prevention-only
design).

## W5 Courts

No code courts in this lane (documentation artifact). The matrix's
own anti-vacuity is receipt-absence falsifiability, stated in 08
("Anti-vacuity of this matrix" section). One loud skip recorded:
[SKIP] no lane-ax-03 receipt -> container symbol-count court witness
ABSENT, DET scored high, never assumed run.

## W6 Fences

Must-not-regress surfaces: 04-pfmea-risk-matrix.md (not modified —
08 supersedes AX rows only and cross-links; 04 remains authoritative
for R6/R10). Chartered charter FMEA at
docs/jira/v26.11.2/14-dflss-charter.md:94-105 untouched (cross-link
only). No shared compile surface touched; compile-freeze SLA not
engaged.

## W7 Standing + divergences

Standing: ALIVE — both owned files written and byte-verified this
session (hash + arithmetic check, exit 0).

Divergences (doc-yields-to-reality):
1. Dispatch said "AX-10's VENDOR-CLAIM-UNVERIFIED marking" — grep
   found the token NOWHERE on disk; only AX-0's prose UNVERIFIED
   verdict lands (lane-ax-00-gate.md:73-76). Scored as MANUAL,
   DET=8, not aspirational-low.
2. Dispatch said "detection = GAP-05's DOC-RECEIPT-002/003" for
   cluster-apply hearsay; those rules check path-resolution and
   status-token presence, NOT exit-code existence — scored DET=5
   (partial-mechanical), with the residual named for DFX-6.
3. 04-R6's named-missing receipt (lane-dx-06-fixture-races.md) has
   since landed; 08 drops rather than re-scores it (not this lane's
   evidence) — flagged to the 04 owner.
4. lane-dx-06 receipt was previously declared missing by the 04
   receipt; on-disk reality changed under us — recorded, not edited.

No SLA unblocks applied to other lanes' files this session.
