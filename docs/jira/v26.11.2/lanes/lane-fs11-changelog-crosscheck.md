# Lane FS-11 — CHANGELOG cross-check

Date: 2026-11-02. Repo: /Users/sac/cgap (main). No git commands run.

## W1 Subject

`CHANGELOG.md` (26.11.2 section, in-place drift fixes) and
`docs/releases/v26.11.2-release-notes.md` (in-place drift fixes).
Both owned files; both living docs.

## W2 Context

Dispatch: spot-verify 8 numbers against `11-measured-claims.md`
register rows; reconcile the Honest-known-bounds block with the final
rider list; land the omitted "Cargo lints table" item if verifiable.

## W3 Findings + diff

1. **Number spot-check (dispatch premise FALSE — recorded as
   divergence):** none of the 8 dispatch numbers (10.20/10.30,
   124.06, 3,529,029 / 4,135,837, 112.48-125.12, 0.15-0.18, 36,048 B,
   6,520,984 B) appear anywhere in either owned file. Greps for
   10.20 / 124.06 / 3,529,029 / 112.48 / 36,048 / 6,520,984 / 0.15 /
   10.30 / 4,135,837 / 3,846,425 / 114.44 / 1.56 returned rc=1 (0
   hits) on both files this session. So there was NO drift to fix:
   the CHANGELOG 26.11.2 section deliberately cites the registry rows
   rather than restating the figures ("Numeric registry of record:
   11-measured-claims.md (rows 1-37 ...)"). All 8 values DO match
   their register rows verbatim — verified by reading rows 5, 28,
   30, 31, 32, 38 of
   `docs/jira/v26.11.2/11-measured-claims.md` (lines 29, 52, 106,
   107, 109, 166): 10.20/10.30 (row 5), 124.06 slowest (row 28),
   3,529,029-4,135,837 (row 30), 123.81-125.12 / 112.48-114.44
   (row 31 — dispatch's "112.48-125.12" spans the two runs' extremes;
   the register's own two columns are 123.81-125.12 and
   112.48-114.44), 0.15-0.18 med (row 32), 36,048 B and
   6,520,984 B (row 38). Nothing altered; registry stays the single
   restatement surface.
2. **Honest-known-bounds reconciliation:**
   - R1-R10 OPEN: present, cites RW-09-O5 — correct as-is.
   - M-CTQ cross-instrument wire legs UNVERIFIED: present (row
     31's D1 note confirms cgo wire authority stays ipc-03) —
     correct as-is.
   - impl16/implcell OPEN: present, cites RW-09-O3 — correct.
   - commit-leg receipts UNSUPPORTED: cite UPDATED — plan-p07 has
     LANDED on disk
     (`lanes/plan-p07-commit-leg-wiring.md`, read this session; its
     own header says "PLAN ... not yet executed") — CHANGELOG bullet
     now cites the landed plan file and carries the NOT-executed
     status so nobody reads landing-of-plan as execution.
   - URN decision coordinator-open: kept, but dispatch's
     "(option b recommended)" is UNGROUNDED — grep for "option b"
     across docs/jira/v26.11.2/ and LICENSE-P28-SIL.md found zero
     referent (only unrelated hits in 12-ard-prd-cgap.md:208 and
     lane-gap-04:23). NOT added; recorded here per no-overclaiming.
   - cli license-via-graph LANDED: verified
     (lanes/lane-cli01-license-via-graph.md W3-W5: manifest_patch.py
     step 1.5, byte-identical replay, cargo build exit 0) and ADDED
     to both files (CHANGELOG Added; release notes §2).
3. **Cargo lints table:** VERIFIABLE, now landed. The
   `[lints.rust]` `unexpected_cfgs` allowlist is real on disk at
   `crates/cgap-git/Cargo.toml:13-18`; receipt
   `audit/gap-04-final-verification.md:111-120` (lane WARN-01
   ERRATA) witnesses the post-fix build. Added to CHANGELOG Added.
   The release notes' own D3 discrepancy bullet ("no `[lints]` table
   exists in any member manifest") is now falsified by disk — fixed
   in place with an in-file ERRATA to the D3 bullet naming the grep
   miss, citing the WARN-01 receipt. Note: the dispatch attributed
   the receipt to lane-r-adv05fix; the actual receipt backing is
   gap-04's WARN-01 ERRATA (adv05fix's sha256 is for gate_battery.py,
   not the manifest). Recorded as divergence.

## W4 Commands (this session, exits)

- `grep -n "26.11.2" CHANGELOG.md` — section located (line 11); exit 0.
- `grep -rn -E "10\.20|124\.06|3,529,029|112\.48|36,048|6,520,984|0\.15" CHANGELOG.md docs/releases/...` — rc=1, 0 hits (number-absence proof).
- Second absence grep (10.30, 4,135,837, 3,846,425, 114.44, 1.56) — rc=1, 0 hits.
- Register rows read directly (11-measured-claims.md lines 29, 52,
  106, 107, 109, 166) — all 8 values match verbatim.
- `sed -n` reads of lane-xc01:20-30, lane-r05:20-28,
  lane-x-05fix:5-12 — 12-symbol and 95-court cites check out
  (xc01 gate "12 == 12 (1:1)" exit 0 witnessed; x-05fix "95
  (85+5+5)" hearsay-disclosed, carried verbatim in the CHANGELOG as
  it already was).
- `grep lints crates/*/Cargo.toml` — exactly one hit:
  cgap-git:13. `ls lanes/plan-p07-commit-leg-wiring.md` — present.
- `awk length>100` sweep post-edit — my added lines all ≤100;
  the >100 lines at release-notes:90-94 are pre-existing OPT-table
  rows (not this diff).

## W5 Fences

- Fence: CHANGELOG 26.10.29 + Unreleased sections must survive
  verbatim — verified: edits confined to the 26.11.2 section; the
  release-notes §6 "pure insert" claim stays true.
- Fence: the registry-of-record single-source law (CHANGELOG cites
  rows, never restates numbers) — PRESERVED; the number-absence
  finding is a conformance, not a defect.
- Fence: no receipt silently rewritten — DOD-06's receipt (lane
  dod-06:73,107) still says "lints absent"; the correction lives in
  the release notes' ERRATA bullet + this receipt, per
  ERRATA-never-rewrite.

## W6 Standing

ALIVE for the reconciliation work (commands + exits above, this
session). The 8-number spot-check lands UNVERIFIED-to-conformed:
values verified against the register, presence-premise of the
dispatch falsified and recorded.

## W7 Divergences

1. Dispatch said the 8 numbers are "in the 26.11.2 section" — they
   are not; both owned files cite rows instead. No drift existed.
2. Dispatch's "112.48-125.12" for row 31 spans the two runs'
   extremes; register row 31 is 123.81-125.12 / 112.48-114.44.
3. "(option b recommended)" for the URN item: ungrounded on disk —
   not added.
4. Lints receipt attribution: gap-04 WARN-01 ERRATA, not
   lane-r-adv05fix.
5. plan-p07 is LANDED-as-PLAN (not executed); cites now carry that
   status explicitly.
