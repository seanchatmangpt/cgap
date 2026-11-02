# BP-6 Reviewer Lane Receipt

Lane: BP-6-reviewer (best-practices reviewer subagent)
Date: 2026-11-01
Lease: `.claude/agents/reviewer.md` (NEW), this receipt. No other file
touched; SM-3 owns lane-runner/court-auditor/gate-runner.

## Files written

- `/Users/sac/cgap/.claude/agents/reviewer.md` — reviewer subagent.
  Frontmatter: name `reviewer`, description (review changed files against
  the spec/receipt they were built from; use after implementing, before
  opening a PR), `tools: Read, Grep, Glob` — the read-only limit is
  enforced by Claude Code tool allowlisting, not requested in prose.
  Body adapted to cgap: compare diff against spec/receipt, grep for
  evidence each requirement is implemented, edge cases must have tests,
  report only correctness-affecting gaps with file:line + spec line,
  skip style, clean review is a valid outcome.

## Cost best-practice note (receipt-only, per dispatch)

Simple collection subagents — file collectors, link checkers, and similar
mechanical fan-out workers — should set `model: haiku` in their
frontmatter per the cost best-practice. Deliberately NOT added to
`reviewer.md` itself: review requires judgment over the spec/diff
relationship; haiku is the guidance for collector-class agents only.

## Verification

- File exists: `ls` shows `reviewer.md` in `.claude/agents/`.
- Frontmatter parse: python re-assert on disk bytes — `name: reviewer`
  and exact `tools: Read, Grep, Glob` line both matched (exit 0).
- Line length: awk `length > 100` scan returned zero lines.
- No git writes performed (coordinator owns transitions).

## Commands

- `ls /Users/sac/cgap/.claude/agents/ ...` — exit 0 (existence + peer check)
- `awk 'length > 100 ...'` + python frontmatter assert — exit 0

## Standing

ALIVE for the file-creation + parse verification above (ran this
session, exit 0). No court required: this is a config/prose artifact
with no runtime behavior to mutate; anti-vacuity mutation of a
frontmatter allowlist is Claude Code harness behavior, not repo code.
