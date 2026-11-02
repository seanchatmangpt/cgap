# bp-10-audit receipt

Lane: BP-10 BEST-PRACTICE-AUDIT (court-auditor, read-only except marker),
/Users/sac/cgap, 2026-11-01. All commands re-run by the auditor; exits cited
in the final report (receipt-in-final-message).

## Per-artifact verdicts

| # | Artifact | Verdict | Evidence (auditor exit) |
|---|---|---|---|
| 1 | CLAUDE.md | PASS | 113 lines; 0 IMPORTANT; 0 old-model directives; datafusion refs only removal pointer; refs resolve |
| 2 | settings.json | PASS | json.load exit 0; Stop hook standalone exit 0; all 4 wired hook scripts exist; allow has no publish/push |
| 3 | .claude/hooks | FAIL | guard-generated.sh fires exit 2 only on RELATIVE paths; Claude Code passes ABSOLUTE file_path -> exit 0 (vacuous on real input). Flip: rel=2 abs=0 |
| 4 | settings.local.json | FAIL | "Bash(cargo publish:*)" PRESENT in allow (line 7); bp-8 receipt claims it was removed — receipt vs disk divergence |
| 5 | skills (3) | PASS | frontmatter parses; release disable-model-invocation: true; 62/66/53 lines match bp-5 |
| 6 | reviewer.md | PASS | tools: Read, Grep, Glob exactly |
| 7 | stardf CLAUDE.md | PASS | 112 lines; deference chain at :15 |
| 8 | bp-5..bp-8 receipts | PASS | claims match disk (except bp-8 publish removal claim) |

## Gaps for the overnight loop

- G1 guard-generated.sh: add absolute-path handling (or normalize to
  basename/tree-relative) — currently blocks nothing in production input shape.
- G2 settings.local.json: remove "Bash(cargo publish:*)" (bp-8 claims done;
  disk says otherwise — re-check for post-receipt rewrite, mtime newest in tree).
- G3 CLAUDE.md:31 cites OVERNIGHT-LOOP.md:128 for the publish law; the law
  lives at :146-149 (:128 is G6-NO-DF). Sm-5 flagged this; unrepaired.
- G4 absent at poll deadline: bp-7, bp-9 receipts; overnight-readiness.md.

## ADDENDUM (W2-19, 2026-11-02) — D2

D2 (from the W1-17 audit receipt): the G2
`Bash(cargo publish:*)` claim (line 7 of settings.local.json,
"PRESENT in allow") is NOT reproducible on current disk —
`grep -n "cargo publish" /Users/sac/cgap/.claude/settings.local.json`
returns exit 1 (0 hits) this session. The line-7-presence claim is
therefore marked UNVERIFIED-at-write; row 4 of the verdict table above
inherits that standing. Divergence noted with mtime ordering recorded
by the audit (settings.local.json was newest-mtime in tree at audit
time). Evidence source: W1-17 audit receipt (bp-10-audit.md G2 row).
