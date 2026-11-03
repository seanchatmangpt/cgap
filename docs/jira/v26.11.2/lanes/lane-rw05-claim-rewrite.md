# Lane RW-05 — claim rewrite (07-court-corpus + 08-registry-sweep)

Date: 2026-11-02. Standing: **ALIVE** for the read/verify legs; the two
edits are doc merges over receipts read in full this session. Owner
files ONLY: `docs/jira/v26.11.2/07-court-corpus.md`,
`docs/jira/v26.11.2/08-registry-sweep.md`. No git commands run.

## Commands + exits (this session)

| command | exit |
|---|---|
| `ls crates/*/tests/*.rs` (court inventory grep) | 0 |
| `ls docs/jira/v26.11.2/audit/` + lanes grep | 0 |
| `grep -n '91' lanes/lane-xc05-conformance-kit-exporter.md` | 0 |
| `grep -rn andon_liveness_court lanes/*.md` | 0 |
| `awk 'length>100'` line-limit check on both files post-edit | 0 (clean) |

## W1 — 07-court-corpus.md §6 added

Wave court reconciliation appended as §6 + §7 (superseding the §5 tail
note). 15 new court files inventoried live from `crates/*/tests/` and
matched to owning receipts (each read this session): breach_regression,
penstock_corpus, quota_alloc_purity, media_loss_recovery,
journal_fault_matrix, capacity_bound, attestation_mac, stats_block,
ref_store_reentrancy, ref_store_width_arms, andon_liveness
(fix04 owner + adv12 expansion, both cited per disclose-in-both law),
pack_demux_poison, ocel2 tamper + roundtrip, fleet-ops courts.
Stale-count correction: §5's 13-row table is the L/G roster only;
XC-05 inventoried **91 courts** (81 cargo-test / 5 corpus-doc / 5
gate-mode) at its run — `lanes/lane-xc05-conformance-kit-exporter.md:30,34`.
§6 cells are receipt-backed but NOT re-run by this lane (UNVERIFIED
here until re-read at use time, stated in the file).

## W2 — 08-registry-sweep.md §1a added

`audit/pub-02-flag-table-rederivation.md` landed and was read in full.
§1a overlay records the PUB-02 supersession: 23 members (not 22);
flags 23/23 explicit, 20 true / 3 false; 13 members at registry
26.11.2, 5 at 26.11.1, cgap-branchless HTTP 404; bench collision
confirmed from response body; wasm-footprint registry state stays
UNVERIFIED (never rounds up). §1's census-era table retained as
provenance, not deleted (merge, never clobber).

## Divergences

- Pre-existing >100-char table rows in 08 §1/§4 left untouched (not
  my diff); all lines I wrote are ≤100 chars (awk-verified).
- PUB-02 names root CLAUDE.md and 01-release-gates.md:32 as
  owner-targets for stale figures; not edited by this lane
  (outside file ownership) — coordinator sequences.
- Receipt law: this file is the lane receipt; summarized reply-back
  delivered in the final agent message.

## JSON

```json
{"lane":"RW-05","files_written":["docs/jira/v26.11.2/07-court-corpus.md","docs/jira/v26.11.2/08-registry-sweep.md","docs/jira/v26.11.2/lanes/lane-rw05-claim-rewrite.md"],"commands":["court inventory ls (0)","lanes/audit greps (0)","awk line-limit checks (0)"],"courts_run":[],"standing":"ALIVE","divergences":["pre-existing long lines in 08 left as-is","CLAUDE.md / 01-release-gates.md stale-flag figures are PUB-02 owner-targets, not edited here","07 §6 cells receipt-backed, not re-run by this lane"]}
```
