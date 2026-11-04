# lane-p17-token-churn

Lane A receipt, P-17 token-churn baseline, 2026-11-03. Subject:
`docs/jira/v26.11.2/bench-validate/p17-token-churn.md` (only repo
file written; harness + results live in /tmp scratch).

## Subject and diff

- Written:
  `docs/jira/v26.11.2/bench-validate/p17-token-churn.md` (155 lines,
  0 lines >99 chars).
- Not written: everything else in the repo. No git writes.

## Commands and exits (all this session, 2026-11-03)

| Command | Exit |
|---|---|
  `python3 -c "import tiktoken"` | 0 (tiktoken 0.12.0 -> token figures
  MEASURED, not ABSENT) |
| `sysctl vm.loadavg` (leg start 06:45:33Z / end 06:47:25Z) | 0 |
| `rustc --version` | 0 (1.101.0-nightly 12433c307; recorded, not exercised) |
| `python3 /tmp/p17-harness/harness.py > results.txt` | 0 |
| `awk length>99` over report (per chunk, x4) | 0, 0 offenders each run |
| number cross-check (25 figures vs results.txt) | 0, all present |

## Verification ladder

1. Harness parse-check after each write chunk (`ast.parse`): 4/4 OK.
2. Harness run exit 0: `ROUNDTRIP_FAILURES 0` over 2000 envelopes
   (decode field-equality + re-encode byte-equality + JSON re-dump
   byte-equality per envelope) - Chicago final-state assert, no mocks,
   no sleeps.
3. Real tokenizer: tiktoken o200k_base, not bytes/4; binary-frame
   token method (latin1 byte-per-char rendering) stated in report §5.
4. All 25 headline figures in the report cross-checked mechanically
   against `/tmp/p17-harness/results.txt` (script, exit 0, "missing:
   none").
5. On-disk byte verification: `wc -l` 155, `awk` line-length 0
   offenders, receipt-tag grep present - report re-read from disk, not
   session memory.

## Standing

ALIVE - measured this session with commands + exits above, against the
exact subject file on disk. Headline (MEASURED): binary frames 14.06%
fewer wire bytes but 27.90% MORE o200k_base tokens than JSON-RPC over
stdio on the same 1000-exchange MCP-style corpus; zero-copy wire
savings do not transfer to the token layer for this corpus.
