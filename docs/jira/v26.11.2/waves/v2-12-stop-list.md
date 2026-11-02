# V2-12 SUPERSEDED-STOP-LIST (court-auditor, read-only except this marker)

Repo /Users/sac/cgap, branch main. Sole write: this file. Zero git writes,
zero builds, zero publishes. Census = on-disk receipt presence + ledger
cross-read, 2026-11-02 session.

## Checked (all absolute paths under docs/jira/)

- v26.11.1/waves/f3-20-terminus.md — v1 BLOCKED; superseded by v2
  (TERMINAL-PUBLISHED 02:47:51Z per ledger 9.3) + coord-publish-record.md
  (all five crates PUBLISHED 26.11.1, exit 0 x5).
- v26.11.1/waves/coord-publish-record.md — exists, ALIVE.
- v26.11.1/waves/f3-publish-ledger.md — landed; cites "f3-20 v2" and
  "W2-7 divergence-1/2"; ledger 9.7.6 names W2-7 PASS x5 terminal.
- v26.11.1/waves/f3-15-api-verify.md — W2-7 section PASS x5 (02:23:18Z).
- v26.11.1/waves/p3-api-verify-all.md — P3-13 verdict table 03:29Z:
  originals PASS x5 (num_versions==2), 17 new crates 404/FAIL — final
  sweep still owed; NOT superseded.
- v26.11.1/waves/p3-pub-*.md — all 12 publisher markers present and
  concluded (spot-checked p3-pub-guard-flip.md tail: all rows closed).
- v26.11.1/waves/{dia-1..6,doc-1..6}.md — all 12 DIA/DOC markers present
  and concluded.
- vac-bench-bn3-chain.md + vac-bench-dead-surface.md — present; ledger
  row: F2-15 flips FLIPPED 4/4, zero VACUOUS.
- paper-*.md — PP-1/PP-2/PP-5/PP-6 receipts present. No PP-4 or PP-7
  receipt found in v26.11.1/waves or v26.11.2/waves (grep, 0 hits).
- RDF-11: no landing receipt; still an open obligation at
  v26.11.2/01-release-gates.md:226,232 (auditor sign-off owed).
  RDF-12: no receipt found in either milestone's waves/.

## Stop list (coordinator action table)

| subject | last known state | superseded-by | rec |
|---|---|---|---|
| W2-12 old terminus | v1 BLOCKED on disk | f3-20 v2 TERMINAL-PUBLISHED + coord-publish-record.md + f3-publish-ledger.md | STOP |
| P3-13 api-verify | partial re-sweep landed 03:29Z; 17-crate FAIL sweep owed | none — W2-7 covers originals only | KEEP until cascade drains |
| RDF-11 patent-map | queued; no receipt | none; obligation live (01-release-gates.md:226) | KEEP |
| RDF-12 terminus | waiting; no receipt | none | KEEP |
| PP-4 | no receipt on disk (grep 0 hits) | none | KEEP (no superseding evidence) |
| PP-7 | no receipt on disk (grep 0 hits) | none | KEEP (per dispatch guidance) |
| P3-2/5/6/8/A-3 publishers | 12 p3-pub-* markers all concluded | p3-pub-*.md (self-concluding) | STOP all |
| VAC stragglers | vac-bench x2 landed, flips 4/4 | v26111-campaign-ledger.md L85-87 | STOP |
| DIA/DOC stragglers | dia-1..6, doc-1..6 all landed | the 12 markers themselves | STOP |

## ADDENDUM (W2-19, 2026-11-02) — D3

D3 (from the W1-17 audit receipt): the stop-list text says "all 12
p3-pub-* markers"; on-disk count is 11 (`ls
docs/jira/v26.11.1/waves | grep -c p3-pub-` = 11 this session:
aux-final, beam-gpu, bn3-gate, bont-bosym, core-storage,
court-audit, crypto-p28, ffi, fibo-cloud, guard-flip,
integration-wasm). "12" above reads as 11; the STOP rows for
p3-pub-* are unaffected (11/11 concluded).
Evidence source: W1-17 audit receipt (D3).
