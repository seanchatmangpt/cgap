# w1-14-mutation-set

- Lane: W1-14 (mutation-set consolidation)
- Date: 2026-11-01
- Subject: docs/jira/v26.11.2/09-overnight-mutation-set.md (new file)
- Sources consolidated: 01-release-gates.md (FLIP blocks G1-G8),
  02-pareto-levers.md (L1-L5 per-lever mutations). No invented mutations.

## Register content

- 9 G-gate mutation rows (G1-M1 .. G8-M1, incl. G6-M2 self-check) and
  9 L-lever rows (L1-M1, L2-M1, L3-M1, L4-M1a/L4-M1b/L4-M2,
  L5-M1a/L5-M1b/L5-M1c). 18 rows total, all [UNFLIPPED-t26.11.2].
- Overnight-loop mechanics recorded: /tmp sandbox mutation -> required
  flip -> sha256-exact restore + mtime bump -> unflipped = VACUOUS-GATE
  (exit nonzero); loud skips only.
- Same-class v26.11.1 witnesses cited where they exist
  (P3-17 parquet flip, SHIM-DOE4 a12 flips, bench-chains-seal triple,
  gate-4 spoliation sweep, gate-5 silicon size-law flips). G1/G3/G4/G6/
  G7/G8 have no v26.11.1 witness found this session -> UNVERIFIED.

## Commands / exits

- grep over both source docs for FLIP/mutation blocks: exit 0
- grep over docs/jira/v26.11.1/waves/ for same-class flip witnesses: exit 0
- python3 heredoc rewrite of 09-overnight-mutation-set.md after a
  corrupted table-header write: exit 0 (on-disk bytes verified: H1 match,
  18 UNFLIPPED tags, tables intact)

## Standing

ALIVE for the consolidation artifact itself (file written and verified on
disk this session). All 18 flip statuses are [UNFLIPPED-t26.11.2] /
UNVERIFIED as executions: no mutation has been run against the v26.11.2
tree; this lane names courts, it does not run them.

## Divergences

- Dispatch mandates table format; markdown law caps lines at 100 chars.
  Several table rows exceed 100 chars (max 344). Tables kept per the
  dispatch; divergence recorded here.
- First Write call corrupted a table header line; fixed by full python
  rewrite in the same session. Final on-disk bytes verified.

```json
{"lane":"W1-14","files_written":["docs/jira/v26.11.2/09-overnight-mutation-set.md","docs/jira/v26.11.2/waves/w1-14-mutation-set.md"],"commands":["grep FLIP/mutation over 01-release-gates.md + 02-pareto-levers.md (exit 0)","grep same-class witnesses over docs/jira/v26.11.1/waves/ (exit 0)","python3 rewrite + awk/grep verification of 09-overnight-mutation-set.md (exit 0)"],"courts_run":[],"standing":"ALIVE (artifact); all 18 mutations [UNFLIPPED-t26.11.2]","divergences":["table rows exceed 100-char markdown law (dispatch table format kept)","first write corrupted a table header; python rewrite + byte verify"]}
```
