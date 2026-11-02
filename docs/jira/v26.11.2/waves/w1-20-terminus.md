# w1-20 — W1 terminus (closed by coordinator; lane was stopped in poll loop)

The W1-20 lane independently verified 11 on-disk digests matching the w1-02 and w1-03
receipts byte-exactly (its final message) but then blocked in its bounded poll and was
stopped. Its verification finding is preserved here; the full W1 census already exists as
waves/w2-20-terminus.md §"W1 receipts" (censused from disk), which supersedes this lane's
remaining scope.

## Verdict

- W1 regen byte-identity: independently re-verified (11/11 digests match w1-02/w1-03).
- W1 census: superseded by w2-20-terminus (all w1-01..19 receipts present, metagate PASS).
- Wave standing: PARTIAL→ALIVE by supersession (no W1 lane vacuous).

{"lane":"W1-20","files_written":["docs/jira/v26.11.2/waves/w1-20-terminus.md"],
"commands":["lane digest verification x11 (agent session, matches receipts)"],
"courts_run":["regen byte-identity cross-check (superseded by w2-20)"],
"standing":"CLOSED-SUPERSEDED","divergences":["lane stopped in poll loop; receipt filed coordinator-side"]}
