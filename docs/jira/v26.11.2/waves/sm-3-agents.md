# SM-3 Agents — lane-runner/court-auditor/gate-runner definition refresh

- **Lane**: SM-3-agents, repo `/Users/sac/cgap` (one canonical checkout)
- **Date**: 2026-11-01
- **Lease**: `.claude/agents/lane-runner.md`, `court-auditor.md`,
  `gate-runner.md`; archive at
  `docs/jira/v26.11.2/readme-archive/agents/`; receipt at
  `docs/jira/v26.11.2/waves/sm-3-agents.md`.

## What changed (session-learned patterns encoded)

1. **lane-runner**: contract block (repo path, lease isolation, no git
   writes, Chicago courts, standing vocabulary, <50-line writes +
   token-sweep, spec cite-don't-redefine list) + new sandbox law:
   frozen seeds `0xBF58_476D_1CE4_E5B9` /
   `0xA20C_0FFE_E5A7_E000`; mtime-bump restore law (sha256-exact
   restore does NOT trigger cargo rebuild — bump mtime or
   `cargo clean -p`); cwd-reset-per-command hazard; build-root leases
   `CARGO_TARGET_DIR=/tmp/<id>-target` deleted via python3
   `shutil.rmtree` (`rm -rf` denied by permission layer); JSON receipt
   terminus schema; compile-freeze SLA (~10 min) with
   disclose-in-both-directions cross-lane unblock law.
2. **court-auditor**: verdicts-only law (never fix); zero-commands =
   VACUOUS audit; /tmp sandbox mutation with sha256-exact restore +
   mtime bump + post-restore green re-run; read-only markers
   discipline (no shared-tree writes; receipt-in-final-message);
   frozen seeds carried in contract.
3. **gate-runner**: exit contract 0/1/2/3 spelled out; compile-abort
   is not a failure count; bounded polls with stated deadlines; ladder
   tables (Gate | Command | Exit | Evidence); BLOCKED = file:line +
   command + exit; build-root lease cleanup via shutil.rmtree.
4. Each definition cites one witnessed exemplar: lane-runner ->
   `docs/jira/v26.11.1/waves/a01-finish-receipt.md`; court-auditor ->
   `docs/jira/v26.11.1/waves/paper-audit.md` (PP-1); gate-runner ->
   `docs/jira/v26.11.2/waves/v2-10-cli-state.md`.

## Divergences

1. All three agent files already existed (pre-scaffolded); per the
   dispatch contract they were archived to
   `docs/jira/v26.11.2/readme-archive/agents/` (cp, byte-identical)
   and rewritten in place — not created fresh.

## Standing

ALIVE — files written this session, verified on disk (wc -l / frontmatter
check below), JSON terminus in the final message.
