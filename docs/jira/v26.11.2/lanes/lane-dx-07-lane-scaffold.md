# Lane DX-7: lane scaffold (agent invariants + lanes template)

- **Identity**: LANE DX-7, repo `/Users/sac/cgap` (branch main, ONE
  canonical checkout). Subject files:
  `.claude/agents/lane-runner.md`,
  `docs/jira/v26.11.2/_LANES-TEMPLATE.md`.
- **Authority**: dispatch prompt (three owned files only).
- **Commands+exits**:
  - `wc -l` + `awk 'length>200'` byte/line sweep of both written
    files — no oversized lines, no truncation; frontmatter
    `name: lane-runner` intact (witnessed this session).
  - `CLAUDE_PROJECT_DIR=/Users/sac/cgap bash .claude/hooks/flip-tests.sh`
    → exit 0, 9/9 PASS (publish-guard fire/quiet/mutant-flip,
    cwd-guard fire/quiet/quiet, mtime-restore-guard fire/quiet,
    rep1-census-gate rc=0 passed=3). First attempt, no retry needed;
    DX-2 serialization not exercised.
- **Diff summary** (before/after, lane-runner.md 88 → 102 lines):
  - Contract block: added BAKED-invariant framing (dispatch prompts
    carry only lane-specific deltas: owned files, context, courts),
    pointer to `_LANES-TEMPLATE.md`, absolute-paths + disjoint
    ownership line.
  - Standing vocabulary enumerated (ALIVE/PARTIAL/BLOCKED/UNVERIFIED/
    REFUSED, never round up).
  - Receipt law: receipt path convention now
    `lanes/lane-<id>-<slug>.md` + W7 skeleton pointer.
  - Disclosure duties with witnessed citations:
    G04 `pub(crate) mod tests` unblock
    (`lanes/lane-g04-lspr-attributes.md:58-68`) and FIX-07's C05
    misfire attribution (`lanes/lane-fix07-bench-build.md:48-54`).
  - Lease law: "unique per lane" made explicit.
  - Sandbox law, Chicago, anti-vacuity, long-file, compile-abort,
    no-sleep, ERRATA, stale-claims, exemplar: carried unchanged.
  - Frontmatter and tool grants untouched.
- **Courts**: none owned; ran the existing flip-tests fence 9/9.
  Anti-vacuity: the publish-guard mutant-flip arm within flip-tests is
  the witnessed flip for the guard family (rc flip block(2)→allow(0)).
- **Fences (must-not-regress)**: ban_generic_agents allowlist — hook
  whitelists by the string `lane-runner` (`ban_generic_agents.py:14`),
  frontmatter name unchanged, so dispatch recognition is preserved
  (verified by reading the hook, not by a dispatch). Second fence:
  flip-tests.sh 9/9 (run, exit 0). No other gates touched.
- **Standing**: ALIVE for the scaffold scope — both files written and
  byte-verified on disk this session; flip-tests exit 0 against the
  exact subject. No perf claims.
- **Divergences**: none. Not touched (per dispatch): settings.json
  (DX-2), CLAUDE.md (MV-8). Build-root lease `/tmp/dx7-target` was
  never created (no cargo builds this lane); nothing to delete.
