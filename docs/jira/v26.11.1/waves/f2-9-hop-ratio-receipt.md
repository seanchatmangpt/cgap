# f2-9-hop-ratio-receipt

Lane F2 item 9 (HOP-RATIO, lane-runner), repo /Users/sac/cgap, 2026-11-01.
Charter: `docs/jira/v26.11.1/phase-change-charter.md` transition 3 (Steam) —
machinery share = 1 − (agent-hops / contract-hops) per phase shape.

## Files written

- `scripts/hop_ratio.py` (NEW, 201 lines) — the instrument; no other file touched.
- `docs/jira/v26.11.1/waves/f2-9-hop-ratio-receipt.md` (this receipt).

## Instrument shape

- Reads a workflow `journal.jsonl`; counts event types launched/started/result/failed.
  The schema has no labels: keys are `v2:<sha>` dispatch-payload content hashes and
  unique keys are distinct agent calls (retries/resumptions share one key).
- `agent_hops` = unique `started` keys. `contract_hops` = `--contract-hops N` or
  `--auto` with a sidecar JSON (lookup order: `wf_*` id, journal basename, then the
  `contract_hops` default key).
- Emits exactly `{"journal", "agent_hops", "contract_hops", "share", "loud_gaps"}`;
  `loud_gaps` = `failed` events with no strictly-later `result` for the same key
  (a result before the failure does not supersede it).
- Exit contract: 0 pass / 1 gate fail (loud_gaps nonempty, or share < 0) /
  2 subject missing (journal absent, malformed line named, zero result events,
  contract_hops <= 0) / 3 loud skip (zero started events, or no contract source).

## Court (all mutations in /tmp only; sandbox /tmp/hop-court-f29)

Baseline sha256 pinned pre-court, verified post-court byte-exact (fail-closed
restore discipline): `scripts/hop_ratio.py` 7c9c58a4dcb9..., wf_399befd9 journal
fe994e96eab5..., wf_f175cd34 journal 887e7ee11662... — `shasum -a 256 -c` OK, exit 0.

| leg | subject | exit | evidence |
|---|---|---|---|
| a1 | real wf_399befd9-aa4 | 1 | JSON: agent_hops=320, contract_hops=320, share=0.0, loud_gaps=73 |
| a2 | real wf_f175cd34-71f | 1 | JSON: agent_hops=58, contract_hops=58, share=0.0, loud_gaps=3 |
| b | vacuity flip (result lines removed, 705->443) | 2 | `SUBJECT MISSING: zero result events` |
| c1 | fixture A, `--contract-hops 8` | 0 | share=0.75 exact, loud_gaps=[] (failed-then-result) |
| c2 | fixture C, line 4 = `NOT JSON AT ALL` | 2 | `malformed line 4: ...` (line named) |
| c3 | fixture B, `--contract-hops 4` | 1 | share=0.5, loud_gaps=[v2:cc @ line 5] |
| d | /tmp script copy, `1.0-(x/c)` -> `(x/c)` | 0* | share=0.25 vs expected 0.75 — flip caught |

Court commands: legs a1/a2 ran `python3 scripts/hop_ratio.py <journal> --auto
--sidecar contracts.json` (fixtures ran the same with `--contract-hops N`).
Leg b's mutation: `grep -v '"type":"result"'` over a /tmp copy of the real
journal. Leg d's shim: /tmp copy of the script, anchor occurrence count
asserted == 1 before replace; restore proven by the sha256 check below.

Leg d is the anti-vacuity mutation of the instrument itself: a script that printed
the expected number regardless of input would fail legs a/c; the formula mutation
moves the fixture's share 0.75 -> 0.25, so the expected-share assertion flips.

## The two real numbers + the steam baseline

- wf_399befd9-aa4 (v26.11.1 waves 1-10 campaign, 705 lines): started=360,
  result=262, failed=82, unique keys=320 -> agent_hops=320, loud_gaps=73
  (9 failed events were later superseded by a result — resumeFromRunId retries
  that succeeded; one key failed twice with both failures unclosed).
- wf_f175cd34-71f (current session, 116 lines): started=58, result=54,
  failed=3, unique keys=58 -> agent_hops=58, loud_gaps=3, zero closed later.

Steam baseline, honestly: `contract_hops` for these two historical runs is
RECONSTRUCTED from the journals themselves (the run's own unique dispatch
payload hashes — the phase shape as dispatched), pinned in the court sidecar.
Machinery share is therefore 0.0 for both runs by construction. That is the
honest reading: these were LLM-heavy orchestration runs with zero externalized
machinery in the loop, and 0.0 is the floor every same-shape replay exists to
beat (next campaign: same sidecar contract, fewer agent() calls, share rises).
The reconstruction is disclosed, not hidden: until a campaign pins its DESIGNED
agent count in a sidecar before the run, contract-hops stays UNVERIFIED and the
instrument can only report the floor. The falsifier stands: flat 0.0 across
campaigns = a sprint ran, not a phase change.

Commands (exits re-read this session): see court table; legs a1/a2 exit 1 =
fail-closed gate on the unclosed failures they genuinely contain (attrition
witnessed, numbers still emitted for the receipt).

## Divergences

1. Dispatch said the a2 journal is at "8f90c4f4... session" — verified present
   under `/Users/sac/.claude/projects/-Users-sac-cgap/8f90c4f4-ac69-479f-8519-
   09a1c68e48f2/subagents/workflows/wf_f175cd34-71f/journal.jsonl`; no conflict.
2. Dispatch spec allowed exit semantics beyond its three named rules to be
   assigned; the repo 0/1/2/3 gate law was mapped as documented above (1 =
   loud_gaps nonempty or share < 0). The a1/a2 legs therefore exit 1, not 0 —
   measured-but-failing, JSON still emitted.
3. Court leg c3 first ran with a zero-result fixture and hit the exit-2 rule
   before the gap rule (correct per spec); fixture B was rebuilt with one
   closed call plus one unclosed failure to witness the exit-1 gap branch.
4. First a1/a2 run reported EXIT=0 through a `tee` pipe — the zsh `$?-after-
   pipe` trap (repo CLAUDE.md); re-run with redirection, true exits 1/1
   recorded. No number in this receipt comes from a piped `$?`.
5. Lease debris: `/tmp/hop-court-f29/` (court sandbox, ~30 KB) left for replay;
   coordinator may sweep at integration.
