# DIA-6 receipt — Diataxis index + navigation (index/SUMMARY)

Lane: DIA-6-index · Date: 2026-11-01 · Repo: /Users/sac/cgap

## Commands + exits

- Bounded poll for DIA-1..DIA-4 output: loop over `docs/diataxis` until
  populated, then `ls -R docs/diataxis` — exit 0. All four quadrant dirs
  present at read time (tutorials 4 files, how-to 3, how-to-guides 2,
  reference 2, explanation 3; 14 files total).
- Case-sensitivity probe (`.casecheck` -> `.CASECHECK`): exit 0 — the
  volume is case-INSENSITIVE APFS.
- SUMMARY.md link check (python re-scan of all relative links against
  disk): `MISSING: none`, exit 0.
- `ls -la docs/diataxis/SUMMARY.md`: 1187 bytes on disk.

## Files written

- docs/diataxis/SUMMARY.md (NEW) — mdBook-shaped nav tree, all 14 doc
  links verified on disk.

## Divergences

1. Contract leased `docs/diataxis/index.md` as NEW, but `INDEX.md`
   already exists and the volume is case-insensitive APFS — creating
   `index.md` would byte-clobber the pre-existing entry point outside
   my lease. INDEX.md is left untouched and serves as the entry point;
   SUMMARY.md links to it.
2. Contract named the quadrant dir `how-to-guides/`; the tree has BOTH
   `how-to/` (DIA-2) and `how-to-guides/` (extra). Both are indexed.
3. `docs/jira/v26.11.1/waves/` contains no `dia-*` receipts at write
   time — DIA-1..DIA-4 docs exist on disk but their wave receipts are
   in-flight; SUMMARY grounding cites on-disk file existence, receipts
   to follow from those lanes.

## Cross-link verification

Every file referenced in SUMMARY.md was resolved against the filesystem
at write time (script above): zero dead links. No quadrant file was
invented; all named siblings exist.
