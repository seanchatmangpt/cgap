# Lane X-06 — bench-validate completeness audit (TRANSCRIBED)

## 1. Identity

- Lane: X-06 (archival transcription lane T-X06).
- Subject: X-06's bench-validate completeness audit over the
  `crates/bench` validation reports.
- Receipt status: **TRANSCRIBED** — content dictated by the coordinator
  dispatch; this file records the audit's findings and their subsequent
  closures. Subsequent closures are noted inline, marked "since".

## 2. Authority

Coordinator dispatch (lane T-X06), repo /Users/sac/cgap, branch main.
Owned file: this receipt only. No git commands run. No code edits.

## 3. Inventory at audit time

| Report | Present at audit time | Note |
|---|---|---|
| bench-01 | present | audited, PASS w/ mislabel (see §4) |
| bench-02 | MISSING at audit time | F-06 owed; **since LANDED** |
| bench-03 | MISSING at audit time | F-06 owed; **since LANDED** |
| bench-04 | present | audited, PASS |
| bench-05 | present | audited, PASS w/ mislabel (see §4) |
| opt-03 | landed mid-audit | audited, FORMAT-INCOMPLETE (see §4) |

The bench-02/03 gap was filed as F-06 (owed completeness work); F-06
has since landed, closing both gaps.

## 4. Per-report verdicts (as of audit time)

| Report | Verdict | Defect / note | Closure |
|---|---|---|---|
| bench-01 | PASS | §4:80 "quiet-leg" mislabel | since closed by BENCH-ADD §7 |
| bench-04 | PASS | none | n/a |
| bench-05 | PASS | "quiet ref" mislabel | since closed by A-03 §8 E1 |
| opt-03 | FORMAT-INCOMPLETE | report shape gaps | since corrected by A-04 |

A-04 note: on correction, A-04 found opt-03's §6 and standing fields
were already present; only the roundoff errata was appended. The
original FORMAT-INCOMPLETE verdict overstated the gap.

## 5. Cross-report consistency

- Channel split: agrees across all three comparable reports (x3).
- M-CTQ identity: verified across reports.
- Same-day tail-spread finding: 33.09 vs 92.58 / 117.72 — disclosed
  per-report; retained as a disclosed divergence, not an error.

## 6. dflss ambient-caveat findings (2)

- `interop-cpk.md` — ambient caveat defect — since closed by A-05.
- `11-ax-dmadv-rollup.md` — ambient caveat defect — since closed by
  A-05.

## 7. M-CTQ bench-side re-measurement finding

Finding: M-CTQ lacked bench-side re-measurement. Since closed by the
F-06 / FS-01 / FS-02 drift-checks.

## 8. Commands + exits

- `ls docs/jira/v26.11.2/lanes/` — exit 0 (this session, receipt-land
  check; audit findings themselves are transcribed from X-06, not
  re-executed).
- No builds, tests, or courts run: transcription lane; standing claims
  are as-of-audit-time and marked "since" where later lanes closed them.

## 9. Standing

TRANSCRIBED / PARTIAL — the audit's findings are recorded verbatim
here; per-report verdicts and the "since closed" attributions are
transcribed from the dispatch, not re-executed this session. Standing
of the closures (F-06, BENCH-ADD §7, A-03 §8 E1, A-04, A-05, FS-01/02)
belongs to those lanes' receipts.

## 10. Divergences / disclosures

- None. No shared files touched; no SLA unblocks applied.
