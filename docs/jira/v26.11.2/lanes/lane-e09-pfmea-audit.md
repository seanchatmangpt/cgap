# Lane E-09 — PFMEA refresh audit (archival transcription)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: TRANSCRIBED (original
audit executed by the E-09 Explore lane; read-only). Post-audit closures:
FM-01 laundering closed by F-02's refresh; the FM-07 header/body RPN
inconsistency it flagged was fixed directly by the coordinator (header
DET 3→2 / RPN 30→20, erratum inline at 04-pfmea-risk-matrix.md:224).

## W1 per-row verdicts (as of audit time)

- FM-01 (:97-107): LAUNDERING / STALE — fix01 receipt real, but the row
  concealed the rename-TOCTOU + case-variant residuals; adv10fix re-status
  pending (since closed by F-02: dual-layer cite, fresh fa9 court exit 0
  7-passed, residuals loud).
- FM-02 (:109-125): CLOSED, CLEAN (fix02 + rc03 receipts; figures match).
- FM-07 (:180-207): CLOSED-RECEIPTED, CLEAN — F2 scoping disclosed in-row;
  correctly cites lane-fix05-pack-demux (the boundary-gate decoy noted at
  :262-264).
- FM-08 (:209-241): REPAIRED-RECEIPTED, CLEAN — adv12fix M1-M4 + F-03
  receipts; observational-only residual retained twice, loud.

## W2 PY rows

- PY-1 STALE cites (canonical_key :898→:931, fa9 :261→:275) — refreshed by
  F-02. PY-2/PY-3/PY-4/PY-5/PY-6 CLEAN; PY-5 keeps the impl16/implcell
  execution-certification residual loud (reserve cite drift :321→:380
  refreshed).

## W3 cross-cutting findings

1. FM-01 was the single laundering surface (row text, not code — the code
   was landed and green).
2. Stale-contradiction bullets: :266 ("FM-08 remains REGISTERED-DESIGN"
   vs :254) — since fixed by F-02; :30/:67 lane-dx-06 absence — lane
   landed, flag actioned.
3. Line-cite drift inventory — refreshed by F-02.

## W4 disposition

All findings closed: F-02 (FM-01 + supplement), coordinator (FM-07 header),
A-03/BENCH-ADD class erratas elsewhere. Register carries SW-2/SW-3 rows.

## W5 standing

TRANSCRIBED, landed by the coordinator. Original audit: E-09 Explore lane
(read-only; could not land its own file — reported BLOCKED).
