# W2-19 RECEIPT REPAIRS (addenda over W1-17 defects)

Lane: W2-19-receipt-repairs, repo /Users/sac/cgap, 2026-11-02.
Sole writes: 5 addenda + this file. No git writes. History preserved —
every repair is an appended ADDENDUM, never a rewrite.

## Addenda landed (each cites the W1-17 audit receipt)

| defect | receipt | addendum content |
|---|---|---|
| D2 | waves/bp-10-audit.md | G2 `Bash(cargo publish:*)` line-7 claim UNVERIFIED-at-write |
|    (grep exit 1 on current disk)                              |
| D3 | waves/v2-12-stop-list.md | p3-pub-* marker count corrected 12 -> 11 (census this session) |
| D4 | waves/v2-13-commit-manifest.md | bench count corrected 14 -> 15 (v26_10_31.rs unaccounted) |
| D5 | waves/w1-02-cli-regen-replay.md | regen re-run x2, cmp exit 0, digests embedded |
| D7 | waves/sm-4-hooks.md | exit-code/marker claims recorded OPEN-defect per sm-5-audit.md |

## Evidence source

The dispatched "W1-17 audit receipt" is not present on disk under
docs/jira/v26.11.2/waves/ (no w1-17 file; census `ls | grep -i
"17\|audit"` = bp-10-audit.md, sm-5-audit.md, w1-01-ontology-audit.md
only). Defect texts D2/D3/D4/D5/D7 match bp-10-audit.md's G-gap rows
and sm-5-audit.md's hook rows; those files are treated as the
effective evidence source. Recorded as a divergence, not a blocker.

## Commands (this session, all exits witnessed)

- `grep -n "cargo publish" .claude/settings.local.json` -> exit 1
  (0 hits) — D2 basis.
- `ls v26.11.1/waves | grep -c p3-pub-` -> 11 — D3 basis.
- `ls crates/bench/benches/ | wc -l` -> 15 — D4 basis.
- Regen x2 in crates/cli: `rm Cargo.toml && ggen sync run` exit 0 x2;
  `python3 manifest_patch.py` exit 0 x2; `cmp` run1-vs-run2 digest
  sets exit 0; digests equal to pre-run baseline (all three states
  identical). Shared tree byte-unchanged (sha256 across the 5
  surfaces before == after).
- `shasum -a 256` over 5 surfaces x3 (baseline/run1/run2) — exit 0.

## Anti-vacuity

D5 is the flip: had the regen drifted, run-1 vs run-2 digests (or
baseline) would diverge and cmp would exit nonzero — the addendum
would then record a regression, not an identical table. The other
addenda are count/existence censuses with the raw command+exit cited
inline; flipping them would require the on-disk artifact set itself
to change.

## Standing

ALIVE — every claim above is backed by a command+exit run this
session against the exact subjects.

## JSON

```json
{"lane":"W2-19-receipt-repairs","files_written":["docs/jira/v26.11.2/waves/bp-10-audit.md","docs/jira/v26.11.2/waves/v2-12-stop-list.md","docs/jira/v26.11.2/waves/v2-13-commit-manifest.md","docs/jira/v26.11.2/waves/w1-02-cli-regen-replay.md","docs/jira/v26.11.2/waves/sm-4-hooks.md","docs/jira/v26.11.2/waves/w2-19-receipt-repairs.md"],"commands":["grep cargo-publish settings.local.json (exit 1)","ls|grep -c p3-pub- (=11)","ls benches (15)","ggen sync run x2 (exit 0 x2)","manifest_patch.py x2 (exit 0 x2)","cmp digest sets (exit 0)","shasum x3 (exit 0)"],"courts_run":["D5 regen-replay digest court (delete-manifest mutation x2, byte-identical)"],"standing":"ALIVE","divergences":["W1-17 audit receipt file absent on disk; bp-10-audit.md + sm-5-audit.md used as effective evidence source","doc- marker count in v2-12 is 7 on disk vs 'doc-1..6' text (observed, out-of-scope for this lane, not repaired)"]}
```
