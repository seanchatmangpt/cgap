# Lane X-05FIX — court-corpus count fix (91 → 95)

## Subject
`docs/jira/v26.11.2/07-court-corpus.md` §6 count prose only.

## Diff
Replaced XC-05's bare 91-court figure with: 91 marked historical/
superseded; live count **95 (85 cargo-test + 5 corpus-doc + 5 gate-mode)**
citing X-05's regeneration witness (conformance_kit_export byte-identical
x2) and the +4 delta named as the wave's post-XC-05 courts. 15-wave-court
list untouched (verified 15 bullet entries on disk post-edit).

## Commands
- `grep -n "91\|95\|..." 07-court-corpus.md` — exit 0; 91 claim now
  carries "superseded (historical)", no other bare "91 courts" in file.
- `grep -c "^- \`"` → 15 (wave-court list intact).
- `ls lanes/lane-x05-court-corpus-audit.md` — ABSENT at edit time.

## Standing: PARTIAL
Count prose landed per dispatch mission. The cited X-05 audit receipt
(`lanes/lane-x05-court-corpus-audit.md`) was not yet on disk at edit
time; citation is annotated "receipt pending on disk at this edit" in
the file itself. X-05 landing that receipt supersedes; until then the
95/85/5/5 figure is hearsay-cited, not independently re-run here.

## Divergences
- Dispatch said "citing the X-05 audit's regeneration witness"; the
  receipt file did not exist on disk, so the citation is qualified
  rather than silently asserted.

## Fences
§6/§7 prose by RW-05 (merge not clobber — only the count paragraph
changed, entries/receipts preserved). No gates exercise this doc.
