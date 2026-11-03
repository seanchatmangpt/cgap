# Charter §7 T-4/T-6 gate-run receipt (CE-11)

Date: 2026-11-02. Single write: this file. No git commands.
Read-first lineage: X-05 (`lanes/lane-x05-court-corpus-audit.md`),
X-05FIX (`lanes/lane-x-05fix-court-count.md`, 91→95),
FS-13 (`dod/dod-16-claim-sweep-verdict.md`, 9 violations),
FS-13FIX (`lanes/lane-fs13fix-claim-qualifiers.md`, closed 9/9,
flip court `violations: 0`).

## T-4 — exporter determinism + live court count

- `python3 scripts/conformance_kit_export.py --out /tmp/ce11-m1.json`
  → exit 0. Run 2 (`/tmp/ce11-m2.json`) → exit 0.
- `cmp /tmp/ce11-m1.json /tmp/ce11-m2.json` → exit 0, byte-identical.
- sha256 both runs:
  `c3ccb0bf1f91f5d6df6286bf7d61f019e1c313ccb24c3d85c993271ec383ca64`
  (matches FS-07's re-grounding hash in X-05 §3 — same tree, same bytes).
- Parsed manifest: `inventory_count` **95** = 85 cargo-test + 5
  court-corpus-doc + 5 gate-mode.
- Charter's bare "91 courts" figure is STALE: 91 is XC-05's historical
  count, superseded per X-05FIX; `07-court-corpus.md:186` marks it
  historical and :191 states live 95 (85/5/5). This run independently
  re-derives 95 on today's tree — not hearsay.

## T-4b — gate battery

`python3 scripts/gate_battery.py --with-self-check` → 8/8 PASS,
battery exit 0:

| Gate | Pass | Self-check |
|---|---|---|
| boundary | 0 (deps §6.4) | both arms flipped |
| zero-string | PASS (loud rlib SKIP row) | PASS |
| cscb-sram | PASS (admission REFUSED = typed gate failure, exit 1, rolled up) | bn3-pin mutant refused |
| ffi-surface | 12 symbols == 12 prototypes | byte-exact re-restore |

Anti-vacuity witnessed this run: boundary self-check poison arm exit 1;
cscb-sram bn3-pin diverged-pin mutant refused (exit 1). The battery's
own flip court (`--prove-refusal`) was not re-run here — its standing
is FS-13-lineage (X-05/adv05fix witness receipts), UNVERIFIED-this-session.

## T-6 — bare-figure + retired-term sweep

Method: ±3-line window court (python), same windows as FS-13FIX's
flip court: figures 10.20 / 124.06 / 36,048 / 6,520,984 / 3.53M /
82.24 GB/s / >10 GB/s without a scope qualifier or row pointer;
retired terms constant-time / side-channel immune(ty) without a row-29
pointer or mechanism reword; $50M without ceiling/floor/ABI scope;
zero-headcount outside negating context.

- Both sales surfaces
  (`docs/REVENUE_GENERATION_COMPENDIUM.md`,
  `docs/CHALLENGER_SALES_REPORT.md`): **0 violations** — FS-13's 9/9
  closures independently re-verified; the 2 `$50M` hits are the
  safe-harbor ceiling ABI pin (compendium :144, :466), FS-13's pass
  class, confirmed.
- Spot-check (5 surfaces + 1 loud skip): README.md, CHANGELOG.md,
  docs/PHASE_CHANGE_RECEIPTS.md, docs/DATA_INDEPENDENCE_AUDIT.md,
  docs/jira/v26.11.2/00-ard-prd.md, 11-measured-claims.md → 0
  violations. [SKIP] `docs/diataxis/reference/index.md` — path does
  not exist (loud, exit 0 otherwise).

**T-6 verdict: PASS** (zero violations; FS-13FIX closures hold).

## Fences (must-not-regress)

- FS-13 verdict doc and FS-13FIX per-site fixes: re-read from disk,
  not contradicted; both sales files untouched by this lane.
- X-05FIX's 07-court-corpus §6 prose: re-read (:186, :191), consistent
  with live 95.
- `dod/charter-s3-ctq-verification.md:59-63` (CTQ-3 already verified
  95-not-91): consistent, no contradiction introduced.
- Gate scripts (`scripts/*.py`): run-only, no edits; shared tree
  untouched except this receipt file.
- No Process.sleep; battery used bounded subprocess timeouts
  (1800 s/gate, stated in gate_battery.py:43).

## Divergences

- Dispatch expected "8/8 exit 0" for the battery; observed 8/8 PASS
  exit 0 — matches.
- Spot-check list included a nonexistent diataxis path; replaced with
  11-measured-claims.md (row-29 owner) to keep 5 real surfaces.

Report JSON:
{"lane":"CE-11","files_written":["docs/jira/v26.11.2/dod/charter-t46-gate-run.md"],
"commands":["conformance_kit_export.py x2 (exit 0, exit 0)","cmp (exit 0)",
"manifest parse count 95 (exit 0)","gate_battery --with-self-check 8/8 (exit 0)",
"T-6 window sweep sales surfaces (0 violations, exit 0)",
"T-6 spot-check 6 surfaces (0 violations, 1 loud SKIP, exit 0)"],
"courts_run":["exporter byte-identical determinism leg",
"gate_battery 4 gates x pass+self-check (8 rows, all exit 0)",
"boundary self-check flip arm","cscb-sram bn3-pin mutate court",
"FS-13 window court re-run (0 violations)"],
"standing":"ALIVE","divergences":["diataxis reference index path absent — loud SKIP",
"battery --prove-refusal not re-run this session (lineage standing, UNVERIFIED-here)"]}
