# Lane README-02 — README CLAIM_RE token-collision residual (2026-11-02)

W1 subject: `/Users/sac/cgap/README.md` only (canonical checkout, branch
main). Authority: dispatch + FS-08 residual findings
(`lanes/lane-fs08-root-docs-receipt-lint.md:27-32`).

W2 diff (line-shape only, meaning + cites preserved):
- README:3 "fixed-width": intro paragraph restructured to the lint's
  sanctioned "-"-bullet shape — "Sovereign columnar/deductive compute
  substrate:" lead line, then two bullets. The "fixed-width" token now sits
  on an exempt bullet line; every word and cite (A04 128 B, A06/B-ONT,
  A11, A17 80 B, A19, `avec` rename date) preserved verbatim.
- README:77 "courts-green" filename token: the brace-list
  `{gate-1-workspace,...,dod-five-gates}.md` restructured to a bullet
  list of the six receipt basenames under a "Receipts
  (docs/jira/v26.11.1/waves/):" lead. All six receipts still named; the
  "green" token now sits on exempt "-" lines. NOTE: the brace-group form
  was a shell-glob shorthand; the bullet list spells it out — same
  receipts, expanded shape.

W3 commands/exits (this session, exact subject):
```
python3 scripts/receipt_lint.py --root /Users/sac/cgap --dir . --json
  -> BASELINE exit 0, 23 claim findings total; README = 2
  (line 3 'fixed-width', line 77 'courts-green') — matches FS-08's
  recorded residual exactly
python3 scripts/receipt_lint.py --root /Users/sac/cgap --dir . --json
  -> AFTER exit 0, 21 claim findings total (23-2); README findings = 0,
  no new finding on any file attributable to this diff (the -2 are
  exactly the 2 README rows)
awk 'length > 100' README.md -> 0 hits (line-length law holds)
git diff --stat README.md -> 51 insertions / 48 deletions vs HEAD; of
  these, ONLY the two hunks above are this lane's — the remainder is
  the pre-existing DOD-07 working-tree diff (README was already M at
  lane start per git status snapshot)
```

W4 verification: before/after JSON re-read from /tmp/readme02-before.json
and /tmp/readme02-after.json this session; README findings re-counted
from the parsed JSON (2 -> 0), not recalled. FS-08's residual is now
closed: both were CLAIM_RE false positives inside hyphenated tokens
("fixed-width", "courts-green"), cleared structurally per the FS-08
mechanism note (bullet-exempt shapes from root, since waves/ refs do not
resolve from repo root). No git writes; docs-only lane.

W5 divergences:
1. Dispatch's court asked to "verify the count" was 3 README findings
   baseline (2 residual + 1 DOD-07-fixed). Measured baseline this
   session is 2, not 3 — DOD-07's fix has already landed in the
   working tree, so the counted baseline is the 2 residual rows.
2. The Receipts brace-list expansion (W2 item 2) trades shell-glob
   shorthand for an explicit list — a shape change beyond a bare line
   split, judged meaning-preserving; flagged here for the coordinator.

W6 standing: ALIVE for the README edits + lint court runs this session
(commands + exits above). Total-lint counts (23 -> 21) reflect README
only; CHANGELOG/CLAUDE/LICENSE-P28-SIL/RELEASE_NOTES findings are
pre-existing and out of this lane's ownership.

W7 fences: must-not-regress surfaces — (a) receipt_lint --self-check
(no script touched; not re-run, unchanged surface), (b) SECURITY/
CONTRIBUTING/README content (all cites preserved; the other root files
were not edited), (c) the DOD-07 pre-existing working-tree diff in
README (untouched outside my two hunks), (d) line-length law (0 lines
over 100). Court of record: the README findings count 2 -> 0 with the
other 21 findings byte-identical classes before/after — the court
flips precisely on the two restructured lines.
