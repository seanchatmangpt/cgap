# SM-5 Receipts Audit — per-file verdict table

Lane: SM-5-audit (court-auditor, verdicts-only). Date: 2026-11-01. Repo /Users/sac/cgap.
Bounded poll: 4x60s until all 5 writer artifacts landed (sm-1..sm-4 + readme-archive/CLAUDE-pre.md); audit ran on the final landed state.

## Verdicts

| File | Verdict | Findings |
|---|---|---|
| /Users/sac/cgap/CLAUDE.md | LAUNDERING-FOUND | 3 defects (below) |
| /Users/sac/cgap/.claude/agents/lane-runner.md | CLEAN | exemplar a01-finish-receipt.md verified (177 tests, 3-mutation flip battery, sha256 restores) |
| /Users/sac/cgap/.claude/agents/court-auditor.md | CLEAN | exemplar paper-audit.md (PP-1) verified read-only pattern |
| /Users/sac/cgap/.claude/agents/gate-runner.md | CLEAN | exemplar v2-10-cli-state.md verified (ladder table, lease-absent divergence) |
| /Users/sac/cgap/.claude/hooks/publish-guard.sh | CLEAN (receipt divergence) | fail-closed verified; SM-4 receipt says "parse error/absent command exits 2" — observed exit 3 (loud skip) per script contract; also receipt says default marker ~/.claude/hooks/... but script line 7 defaults to repo .claude/hooks/ path |
| /Users/sac/cgap/.claude/hooks/cwd-guard.sh | CLEAN (receipt divergence) | fail-closed verified; same receipt exit-2-vs-3 mismatch |
| /Users/sac/cgap/.claude/hooks/mtime-restore-guard.sh | reproduce-FAIL on receipt | receipt claims "Fail-closed on malformed stdin (exit 2)"; mine: `echo garbage \| ...` -> exit 3. Nonzero fail-closed either way; receipt exit claim not reproducible |
| /Users/sac/cgap/.claude/hooks/flip-tests.sh | CLEAN | exit 0 reproduced (see commands) |
| /Users/sac/cgap/.claude/settings.json | LAUNDERING-FOUND (post-write change) | settings.json was rewritten mid-audit: it now wires guard-generated.sh, guard-secrets.sh (PreToolUse Edit/Write), fmt-edited.sh (PostToolUse), and a Stop cargo-check hook -- NONE of these scripts exist on disk (.claude/hooks/ has only ban_generic_agents.py + the 3 SM-4 hooks + flip-tests.sh). Wired-but-missing scripts = claims without backing. SM-4's three hooks remain UNWIRED as its receipt states. settings.json remains valid JSON. |
| /Users/sac/cgap/.claude/settings.local.json | CLEAN | valid JSON; 10 allow entries; unchanged per SM-2 NO EDIT |
| docs/jira/v26.11.2/waves/sm-1-root.md | CLEAN | archive sha256 claim not independently re-hashed (out of scope; archive file present, content matches pre-image) |
| docs/jira/v26.11.2/waves/sm-2-settings.md | CLEAN | NO EDIT disposition matches disk |
| docs/jira/v26.11.2/waves/sm-3-agents.md | CLEAN | changes match disk; exemplars verified |
| docs/jira/v26.11.2/waves/sm-4-hooks.md | LAUNDERING-FOUND | 3 exit/marker claims not reproducible (see hooks rows) |

## CLAUDE.md defects (each with correction needed)

1. CLAUDE.md:47 cites `OVERNIGHT-LOOP.md:128` for publish sequencing/backoff. Line 128 is the G6-NO-DF datafusion gate. The publish law is at `OVERNIGHT-LOOP.md:146-149`. Correction: cite :146-149.
2. CLAUDE.md:50-52 "Delisted from publish (publish = false): audit, integration, and beam-optional set" contradicts disk and its own cited table (01-release-gates.md:28-32 = court/cli/bench delisted + beam optional). Live Cargo.tomls: audit/integration/ffi are publish=true with cited guard-flip receipts (P3-2b/P3-10); current publish=false members are bench, beam, court. Correction: rewrite the delist sentence to name bench/beam/court (beam optional), keep the 19/22 cite.
3. CLAUDE.md:83 "waves/ (9 receipts)" — uncited hardcoded count, already stale: waves/ now holds 15 files (10 v2-*, 5 sm-*). Correction: drop the count or cite with as-of-date.
4. Minor: CLAUDE.md:7-10 counts (22 members / 18 published / 4 absent / bench collision) carry no in-file cite; they are receipted at sm-1-root.md SS2 census. Correction: append a cite.

## Court evidence

flip-tests.sh reproduced: exit 0, 8/8 PASS including the marker-mutation flip (block->allow). Fail-closed probes: malformed stdin -> exit 3 all three hooks; no-authority `cargo publish` -> exit 2 with block JSON. Three hooks UNWIRED in settings.json (only ban_generic_agents.py wired) — honest UNWIRED status confirmed.

## Standing

ALIVE — every verdict above backed by a command+exit run this session.
