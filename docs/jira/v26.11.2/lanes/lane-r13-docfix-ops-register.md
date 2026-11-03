# Lane R-13 — ops-requirements + SIPOC doc merge (FIX-14 / 07-ax)

- Repo /Users/sac/cgap, main. No git commands. Owned files only:
  `docs/jira/v26.11.2/redteam/fix-14-ops-requirements.md`,
  `docs/jira/v26.11.2/dflss/07-ax-sipoc-c4.md`.
- Read first: EVAL-04 defect set via dispatch; lane-rc05-stats-block.md,
  lane-rc13-fleet-ops.md, lane-xc01-ffi-surface-truth.md (all ALIVE,
  all claims re-read from the receipt files at use time, not memory).

## Diff (merge, never clobber)

fix-14-ops-requirements.md:
- A1 status MISSING -> PARTIAL, citing lane-rc05-stats-block.md
  (CgasStatsBlock 120 B CRC-sealed, extern "C" cgap_stats_snapshot,
  Cell64 single-writer for rv32imc, 5/5 courts + witnessed mutation
  flip). HONEST RESIDUAL kept: ring-owner integration
  (stats::global().record_* at ring sites) is DOCUMENTED-NOT-WIRED.
- A3 status MISSING -> PARTIAL, citing lane-rc13-fleet-ops.md
  (standalone cgap-fleet-ops, RMA state machine + hash-chained log,
  4 courts green, 2 witnessed sandbox mutation flips). Residuals kept:
  R-A3.1 device identity not minted in cgap-ffi; kernel counter
  minting still a future lane.
- A6 status PARTIAL (forensics only) -> PARTIAL (forensics + host
  poller), citing slo.rs (AlertPredicate, poll_until, SloSnapshot)
  per RC-13; kernel counter minting + SloSource feed still MISSING.
- Census note (file head): the 5-file cgap-ffi listing predates
  stats.rs — as-of qualified, original grep evidence preserved as
  historical (errata, never rewrite).
- Standing section updated to PARTIAL A1/A3/A6 post-merge.

07-ax-sipoc-c4.md:
- :22 supplier row -> "C-ABI surface, 12/12 symbols as of XC-01"
  citing lane-xc01-ffi-surface-truth.md.
- :31 input row qualified (11 then; 12/12 as of XC-01).
- :40 PX-3 row qualified.
- C4 diagram dylib cell -> "nm -gU 12/12 (XC-01)".
- "binds" arrow -> "(C-ABI, 12/12 XC-01)".
- NEW FFI-census note blockquote after Section 2: "11 symbols" is
  historical-as-of-IPC-0, not current truth.

## Cite verification (this session)

- crates/cgap-ffi/src/stats.rs:369 `pub unsafe extern "C" fn
  cgap_stats_snapshot(` — on disk.
- crates/cgap-fleet-ops/src/{lib,fleet,slo}.rs — on disk.
- 07-ax census figures are cited from lane-xc01 receipt (12/12,
  exit 0), not re-measured by this lane (no nm run here — doc lane).

## Commands + exits

- Read/grep/awk sweeps over both owned files + cited crates
  (exits 0). awk length audit of fix-14: all lines <= 100 chars.
- 07-ax has pre-existing wide table rows > 100 chars (md table
  convention; rows 18-32 etc. predate this lane). My added/edited
  rows match that file convention rather than clobber it — recorded
  as a divergence.

## Standing

- ALIVE for the doc edits themselves (files written this session,
  on-disk state verified by grep/awk). The underlying code claims
  (12/12 census, 5/5 courts, mutation flips) stand on the cited
  lane receipts, run in their own sessions — this doc lane re-read
  them, did not re-run their courts.

## Divergences / fences

- Divergence 1: 07-ax table-row width convention kept (> 100 char
  rows pre-existing); fix-14 fully <= 100.
- Divergence 2: dispatch's claimed exact text at 07-ax:22
  ("11 symbols REAL nm -gU = 11") had drifted; actual text was the
  supplier-table row — qualified that plus all other "11-symbol"
  occurrences (lines 31, 40, 76, 93 pre-edit).
- Fences: no code, gates, or shared seams touched; the cited gates
  (ffi_surface_gate, stats_block_court, fleet-ops courts) were run by
  their owning lanes this session per their receipts and are quoted
  as-of-receipt, not re-run here. No fence could regress from a
  two-doc diff.
