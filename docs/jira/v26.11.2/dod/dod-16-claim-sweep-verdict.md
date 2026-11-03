# DOD-16 / FS-13 — zero-bare-claim sweep verdict

Date: 2026-11-02. Lane FS-13 (DOD-16). Read-only sweep; this file is the
only write. Method: repo-wide `grep -rn --include='*.md' --include='*.rs'
--include='*.toml' --include='*.h'` over /Users/sac/cgap (target/ and
.git excluded); 335 figure hits + 206 retired-term hits classified by
reading surrounding lines; FS-10 census
(`lanes/lane-fs10-constant-time-sweep.md`) is the prior for the
constant-time class.

## Verdict: FAIL — 9 violations

Classification rule: a figure is QUALIFIED if it carries a quiet-host /
subject scope or a row/receipt cite within proximity; a table passes if
it carries a row-cite column. Frozen receipts with in-row scoping pass.

### Typed violation list (findings only — no fixes applied)

1. BARE-FIGURE — `docs/REVENUE_GENERATION_COMPENDIUM.md:260` — "82.24
   GB/s" (BN3 scan, SKU Tier-3 table) with no quiet-host scope and no
   row/receipt cite in the row. Fix needed: add "(quiet-host divan,
   claims-table row cite)" or a row-cite column entry.
2. BARE-FIGURE — `docs/REVENUE_GENERATION_COMPENDIUM.md:313` — "82.24
   GB/s (64k) / 78.94 GB/s (1M)" (F2-13 dossier bench table) with no
   quiet-host scope and no row-cite column. Fix needed: scope marker or
   row-cite column entry. Note: sibling rows in the same table DO carry
   cites (e.g. the F2-11 row cites "register row 7, MEASURED"), so the
   table is inconsistent, not wholesale unscoped.
3. BARE-RETIRED-TERM — `docs/CHALLENGER_SALES_REPORT.md:21` —
   "deterministic, constant-time execution" with no row-29 pointer.
   Fix needed: row-29 UNSUPPORTED pointer or reword to mechanism
   descriptor ("branch-free").
4. BARE-RETIRED-TERM — `docs/CHALLENGER_SALES_REPORT.md:23` —
   "Constant-time ladder searches" (Wire-Speed bullet), plus adjacent
   unscoped figure ">10 GB/s" (outside the named sweep list; noted).
   Fix needed: row-29 pointer or mechanism reword; scope the GB/s.
5. BARE-RETIRED-TERM — `docs/CHALLENGER_SALES_REPORT.md:48` — "side-
   channel immune" (Arrow Quad Filtering row) with no row-29 pointer.
   Fix needed: pointer or reword; the sibling BN3 row in the same table
   IS scoped, so the row is inconsistent with its own table.
6. BARE-RETIRED-TERM — `docs/REVENUE_GENERATION_COMPENDIUM.md:240` —
   "`oblivious` constant-time gather" (cgap-parquet SKU row).
7. BARE-RETIRED-TERM — `docs/REVENUE_GENERATION_COMPENDIUM.md:241` —
   "constant-time `ladder_search` over 20B OIDs" (cgap-git SKU row).
8. BARE-RETIRED-TERM — `docs/REVENUE_GENERATION_COMPENDIUM.md:321` —
   "100% Constant-Time Side-Channel Immunity" (challenger_git row).
9. BARE-RETIRED-TERM — `docs/REVENUE_GENERATION_COMPENDIUM.md:486` —
   "309.2 ns constant-time OID ladder search" (replay step).
   Fixes for 6-9: row-29 pointer (the compendium §1 wedge, line 33,
   already carries the correct row-29 demotion wording to copy) or
   mechanism-descriptor rewording.

### Pass classes (no action)

- All named figures in `docs/jira/v26.11.2/` surfaces (00/02/03/04/05/14/
  15, dflss/, audit/, bench-validate/, lanes/, redteam/, dod/) are
  QUALIFIED: row cites or scope markers present (spot-read ~60 lines).
- `85 ns` / `3,529,029` / `124.06` / `36,048` / `6,520,984` hits outside
  the campaign docs: code comments cite RED-12/gate/ipc-03 receipts; all
  QUALIFIED. `crates/cgap-ocel2/src/lib.rs:49` and
  `crates/bench/README.md:34` carry "receipted" scope; QUALIFIED.
- False positives (not violations): `836,048 B` (wasm-footprint NOTES,
  substring of 36,048); `10.20 GiB/s physical` (AVEC-110, different
  metric); `RESOLUTIONS.md:23` (line number).
- Mechanism descriptors per FS-10 prior (pass with note): `.rs` doc
  comments in cgap-git/cgap-gate/cgap-bn3/cgap-p28/cgap-branchless/bench
  describing branch-free code shape; `docs/diataxis/` references;
  `docs/PHASE_CHANGE_RECEIPTS.md` + `docs/DATA_INDEPENDENCE_AUDIT.md`
  are NEGATING (audit says NOT constant-time); compendium §1 wedge
  (line 33) carries the row-29 demotion in full.
- `$50M` hits: all are the safe-harbor ceiling ABI pin (5_000_000_000
  cents, A17) or floor-vs-ceiling disambiguation (compendium:458,
  CHANGELOG:82); no un-scoped $50M revenue claim found. `zero-headcount`
  appears only inside RED-16's UNSUPPORTED finding (fix-14:200,
  hyperscaler:262) — negating context; 0 live claims.

### Relative-link spot-verify (part 3)

- Census of markdown relative links in the named living surfaces
  (`dod/`, `dflss/`, `audit/`): exactly 1 relative link exists —
  `audit/pub-13-docsrs-plan.md:93` `(LICENSE.md)` — and it is TEMPLATE
  content inside a fenced README snippet ("CRATE" placeholders); the
  target exists at the render location (repo-root LICENSE.md, verified
  on disk). 0 broken. [NOTE] The named surfaces contain fewer than the
  15 requested link samples; census-verified the entire population (1)
  instead, and widened the census: `docs/*.md` top-level + `docs/
  diataxis/` hold 13 relative markdown links, all verified OK (full
  population, not a sample; seed 13). Total verified this session: 14
  links, 0 broken.

### Fences

Must-not-regress surfaces named and checked: FS-10's constant-time
census (its NEGATING + MECHANISM classes re-derived, not contradicted —
this sweep's 6 new constant-time violations are OUTSIDE FS-10's scope
(docs/jira/v26.11.2/ + README), in docs/ top-level sales surfaces);
row 29 (11-measured-claims.md:53, re-read on disk, UNSUPPORTED standing
intact); row 38 (11-measured-claims.md:166, subject-scoped form intact);
FS-11's CHANGELOG number-absence claim (CHANGELOG.md carries no bare
v26.11.2 figures — confirmed: its only figure-adjacent line is the
scoped SS2.4 floor note at :82). No frozen receipt bytes edited; no git
commands run; single-file write discipline held.
