# Lane DOD-06 (changelog) — CHANGELOG + v26.11.2 release notes

Date: 2026-11-02 · Repo: /Users/sac/cgap (main) · Standing: ALIVE
Owned files (ONLY): root `CHANGELOG.md` (append-only insert above
`## [Unreleased]`) and NEW `docs/releases/v26.11.2-release-notes.md`.
No git write commands; read-only `git diff` used for the insert proof.

## W1 Subject + identity

- CHANGELOG.md: pre-wave Oct-31 version preserved verbatim; the
  26.11.2 section is a pure insert — `git diff --stat` = 116
  insertions(+), 0 deletions (this session).
- docs/releases/ did not exist; created with the one new file.

## W2 Diff (mu)

1. CHANGELOG [26.11.2] section: summary, Added (12-symbol FFI surface
   incl. cgap_stats_snapshot; CgasStatsBlock 120 B CRC-sealed; M-CTQ
   stations; breach_regression court; OPT rows pointer), Measured
   performance table (row 5 / row 28 / row 30 / row 31 / row 32 /
   SRAM 36,048 B + bn3 REFUSED), Hardening summary, Changed (publish
   freeze, implcell RUSTFLAGS gate, version unification), Honest
   known bounds (9 bullets, all register-cited).
2. Release notes: migration guide (riscv32imc RUSTFLAGS
   '--cfg implcell_critical_provided' or typed refusal; default-features
   E0463 caveat; no_std seal-counter consumers; header consumers
   POST-WINDOW per P-07 — explicitly not in 26.11.2), publish-state
   section (8 live crates, frozen set, 16/20 dry-run matrix,
   post-lift sequence), OPT-01..05 roadmap strictly TARGET
   (v26.11.3-candidate) citing 11-measured-claims §5 rows 33-37 +
   bench-validate/opt-03-channel-curve.md, Discrepancies section,
   known-open section, replay/falsifier section.

## W3 Commands + exits (this session)

| command | exit |
|---|---|
| reads of 11-measured-claims.md, red-20 synthesis, publish-halt.md, LICENSE-P28-SIL.md rider region, 01-release-gates.md flag rows, pub-01-dry-run-matrix.md, lane-r14, X-05FIX, DEFECT-REGISTER RW-09-O* rows, v26.11.3-forward-backlog P-07 §1, lane-r-adv11fix, stats.rs, cgap.h | 0 (Read tool; no shell exits claimed beyond these) |
| python3 chunk-assembly writes x4 (CHANGELOG) + x3 (release notes) | 0 each (assert-guarded) |
| awk >100-char sweep over both files | 0 (only 5 long lines = markdown TABLE rows, prose lines all <=100) |
| git diff --stat CHANGELOG.md (read-only) | 0, 116 insertions / 0 deletions |

## W4 Courts + anti-vacuity

- Authoring lane: no executable court of its own; anti-vacuity is the
  cite-verification pass — every number in both files was re-read from
  its registry row/receipt this session before writing. The 5-leg
  breach_regression court and the 95-court kit are CITED subjects
  (their flips live in lane-r14 and X-05FIX receipts), not re-run by
  this lane.
- Insert-proof control: `git diff` 0-deletions is the mutation-free
  control for "history preserved".

## W5 Fences this diff could break (and their state)

- Fence: CHANGELOG consumers that parse `## [Unreleased]` as the first
  section — preserved (single `[Unreleased]` anchor remains; grep
  witnessed one hit at :127).
- Fence: 11-measured-claims exclusivity law — no number appears in
  either file outside a row/receipt cite; row 29 "constant-time" is
  carried as UNSUPPORTED prose, never as a figure.
- Fence: publish ONE-attempt law — no publish commands run; freeze
  state recorded as-is (FROZEN-BY-OPERATOR).

## W6 Divergences (disclosed)

- D1: dispatch figure "16 corrosion dossiers" vs disk = 18 ADV-NN
  files (ls redteam/corrosion/ this session). On-disk count used.
- D2: dispatch figure "7 breach families repaired with witnessed
  mutation flips" is not grounded on disk; grounded substitutes used:
  18 red-20 breach rows + 15 REPAIRED-RECEIPTED register rows + the
  5/5 flip-witnessed breach_regression legs (lane-r14 W3).
- D3: dispatch item "Cargo lints table" — no `[lints]` table exists in
  any manifest or receipt this session (grep 0 hits). OMITTED from the
  CHANGELOG entry (never claim unreceipted); recorded in the release
  notes Discrepancies section. If intended, it must land with its own
  receipt.
- D4: "12 exported symbols" grounded, not assumed: witnessed at
  lanes/lane-xc01-ffi-surface-truth.md:26 ("12 exported cgap_* symbols
  == 12 header prototypes") and lane-r05-gate-battery-recount.md:24.
- D5: lane-id collision: a prior `lane-dod-06-pins.md` receipt exists
  from an earlier wave; this receipt uses slug `lane-dod-06-changelog`
  to keep receipts one-per-file.
- D6: prior-dispatch deaths (API 429) disclosed: this session started
  from the pre-wave Oct-31 CHANGELOG exactly as the dispatch predicted.

## W7 Standing

ALIVE for the authoring surface: both owned files on disk, byte lengths
witnessed, insert proof run, every number cite-verified this session.
The cited courts (breach_regression 5/5, 95-court kit) remain the
standing of their own receipts; this lane re-ran none of them.

```json
{"lane":"DOD-06-changelog",
 "files_written":["/Users/sac/cgap/CHANGELOG.md",
  "/Users/sac/cgap/docs/releases/v26.11.2-release-notes.md",
  "/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-dod-06-changelog.md"],
 "commands":["git diff --stat CHANGELOG.md (exit 0: 116 insertions, 0 deletions)",
  "awk length>100 sweep over both files (exit 0; only table rows exceed)",
  "grep single-[Unreleased]-anchor check (exit 0, one hit :127)",
  "python3 chunk-assembly writes x7, assert-guarded (exit 0 each)"],
 "courts_run":["none executable (authoring lane); anti-vacuity = full cite-verification pass over 11-measured-claims rows 1-37, red-20, publish-halt, pub-01, lane-r14, X-05FIX, DEFECT-REGISTER RW-09 rows, v26.11.3-backlog P-07"],
 "standing":"ALIVE",
 "divergences":["D1: 18 corrosion dossiers on disk vs dispatch 16 (disk used)",
  "D2: '7 breach families' ungrounded; 18 breach rows + 15 REPAIRED-RECEIPTED + 5/5 lane-r14 flips used",
  "D3: Cargo lints table absent on disk - omitted, recorded as UNVERIFIED",
  "D4: 12-symbol figure grounded at lane-xc01:26 + lane-r05:24",
  "D5: slug lane-dod-06-changelog to avoid lane-dod-06-pins collision",
  "D6: prior dispatch died on API 429 pre-write; Oct-31 CHANGELOG intact at start"]}
```
