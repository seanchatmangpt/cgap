# Lane A-01: court-count reconciliation (SECURITY.md)

Date: 2026-11-02. Owner files: `SECURITY.md` only;
README.md/CONTRIBUTING.md grep-only.

## Diff

- `SECURITY.md` "Conformance baseline" rewritten: 91 -> **95 courts**
  (85 cargo-test + 5 corpus-doc + 5 gate-mode), citing the
  regeneration witness (exporter byte-identical x2, X-05 audit,
  DOD-12 `--list` 95). Previous ERRATA block ("95 is hearsay") replaced
  with a corrected ERRATA: XC-05's frozen receipt recorded 91 at its
  run time (historical, not rewritten); the +4 delta is wave-added
  courts landing post-XC-05.
- README.md / CONTRIBUTING.md: grep for bare `91` — no bare-91
  court-count claims found (README's only hit is `~91%` BLAKE3 fold
  ratio, unrelated). No edits.

## Commands / exits

- `grep -n "91" SECURITY.md; grep -rn "\b91\b" README CONTRIBUTING` — exit 0
  (hits as above).
- `sed -n 182,200p docs/jira/v26.11.2/07-court-corpus.md` — exit 0;
  witness verified on disk: 91 marked superseded/historical, 95 = live
  regenerated count (85+5+5).
- Post-edit sweep: `grep -rn -E "(^|[^0-9])91([^0-9]|$)" SECURITY.md
  README.md CONTRIBUTING.md | grep -iv 91%` — exit 0; only surviving
  91-mention is the historical ERRATA line itself (intended).
- Repo-wide "91 courts" sweep: remaining hits confined to frozen
  receipts (`lane-xc05-...`, `lane-x-05fix-...`, `lane-rw05-...`,
  `lane-dod-07-...`) and `dod/claude-md-handoff.patch.md` (patch
  artifact, historical) — all within the never-rewrite boundary.

## Standing

ALIVE for the SECURITY.md edit (commands + exits this session, witness
re-read from disk). Court counts are doc-state, not executed courts;
the 95 figure is adopted from X-05's audited regeneration, not
re-executed here.

## Divergences

- Dispatch expected SECURITY.md to carry a stale 91 as current;
  on-disk SECURITY.md actually carried 91 as current PLUS an
  anti-95 errata (DOD-07's divergence, confirmed). Both addressed in
  one edit.
- `docs/jira/v26.11.2/dod/claude-md-handoff.patch.md` still contains
  "91 courts" text — patch artifact outside my owned files; left
  untouched, flagged for coordinator.
