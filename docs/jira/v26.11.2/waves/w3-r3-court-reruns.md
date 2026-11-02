# w3-r3 — W2 stale-race court re-runs (court-auditor verdict, filed by coordinator)

The auditor delivered verdicts in-message but wrote no file; this lands the verdict so the
metagate can cite it. All commands/exits are the auditor's own runs.

## Court 1 — zero_datafusion_gate: ALIVE / non-vacuous

Production exit 0 (`ZERO-DATAFUSION: PASS`); self-check PASS. Flips witnessed: plain
`datafusion = "43"` dep → exit 1 naming the full transitive graph; `datafusion-core` path
dep → exit 1; feature-gated dep → exit 2 GATE-REFUSED (three-probe disagreement, loud);
PATH without cargo → exit 2; malformed manifest → exit 2. Self-check operates only on
tempfile copies (:211-227). Minor defect: bare-python3 (no tomllib) environments crash with
exit 1 traceback instead of typed exit 2 — repair-owed.

## Court 2 — registry_sweep: ALIVE / non-vacuous

Baseline fixture exit 0 `PASS 0 missing`. Flips: dead-proxy API → exit 1 `[MISSING] ...`
(loud, not blank); publish-guard drop → exit 1 PUBLISHABLE-UNATTESTED (flip A); unresolvable
court pointer → exit 1; version drift + pointer → exit 1 (flip B); restores green. Grep
confirmed zero `cargo publish` invocations. Defect class noted: the script writes
`registry-sweep-table.json` into the repo tree on every run — in-tree invocations mutate
waves/; route through `--out`/sandbox discipline.

## Court 3 — receipt_lint D1 patch: PARTIAL (at audit time)

Near-empty-line arm flips under the patched bytes; residual WEAK-GATE — the content check
was length-only, an arbitrary substantive in-range line still passed. Coordinator closed
this AFTER the audit: resolve_ref now requires the cited line's ±2-line window to share
content words with the claim sentence (stale/irrelevant receipt refuses); self-check green
and strict run exit 1 (census non-empty as expected). Residual honest limitation: the
overlap check is word-level, not semantic.

## Court 4 — W2-4 probes re-confirmed

Empty dir → exit 2 fail-closed; misspelled `[TARGET-v26.11.3]` → exit 1 flagged; exact-token
match strict. MEASURED.

## Footer

{"lane":"W3-R3","files_written":["docs/jira/v26.11.2/waves/w3-r3-court-reruns.md"],
"commands":["zero-DF probes a/b1/b2/c/d/e","registry-sweep flips A/B + restores",
"receipt-lint probes b/empty-dir/misspelled-TARGET"],
"courts_run":[4],
"standing":"ALIVE (courts 1/2/4); PARTIAL (court 3 residual, closed post-audit)",
"divergences":["receipt filed coordinator-side (auditor wrote no file)",
"zero-DF bare-python3 crash path repair-owed",
"registry_sweep repo-tree output routing repair-owed"]}
