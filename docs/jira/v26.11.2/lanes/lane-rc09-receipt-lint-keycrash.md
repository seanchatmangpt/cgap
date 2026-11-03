# lane-rc09-receipt-lint-keycrash receipt

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Lane RC-09.
Owned file: `scripts/receipt_lint.py` (only file written).

## 1. Defect + repro (pre-fix, this session)

DF-05 recorded `receipt_lint.py --strict` OUT-OF-CONTROL with an open
`KeyError('rule')` --strict crash. GAP-05 (receipt
`lane-gap-05-receipt-lint.md:29`) fixed the claim-finding path, but the
strict arm kept unguarded `f["rule"]`/`f["reason"]` dict access in
`main()` — any scan emitting a finding without the key re-opens the
crash. Reproduced this session via a /tmp-mutated copy of the real
script (DOC-RECEIPT-002 finding with the `rule` key dropped):

    File ".../mutated.py", line 880, in main
        struct.sort(key=lambda f: (f["rule"], f["file"]))
    KeyError: 'rule'

(exit 1, stack trace — fail-open-by-crash confirmed against HEAD).

## 2. Diff (scripts/receipt_lint.py only)

- `normalize_finding_keys()`: any finding lacking "rule"/"file"/"reason"
  becomes a typed `RECEIPT-LINT-MISSING-KEY` row naming the file + the
  missing key(s), appended to struct.
- `main()`: guard wired in; all downstream dict access typed via
  `.get` (sort keys, struct print, strict rule-id cite, struct_counts,
  claim print). Strict exit-1 path can no longer KeyError.
- `key_crash_court()` wired into `--self-check`: (arm 3) complete
  fixture -> strict exit 0; (arm 1) /tmp-mutated script copy emitting a
  key-less finding -> typed row, exit 1, NO traceback, default mode =
  loud row only; (arm 2, anti-vacuity) guard-stripped /tmp copy -> no
  typed row -> court REFUSES.
- Docstring documents the typed-outcome law.

## 3. Commands + exits (this session)

| Command | Exit |
|---|---|
| mutated HEAD copy --strict (repro) | 1 with KeyError 'rule' traceback |
| python3 scripts/receipt_lint.py --strict (real tree) | 1 typed STRICT-FAIL, 0 tracebacks |
| python3 scripts/receipt_lint.py --self-check (pre-fix) | 0 |
| python3 scripts/receipt_lint.py --self-check (post-fix) | 0 (all courts) |
| final --self-check confirmation | 0 |

## 4. Courts run

KEY-CRASH PASS (typed row, named file+'rule', no traceback; default
mode loud row; guard-stripped copy refused), STRICT PASS, DOC-FLIP
PASS, LAYER-FLIP PASS, GAP-FLIP PASS (A-D), SELF-CHECK PASS.

## 5. Standing

ALIVE — every claim above ran this session with the cited exit.

## 6. Fences

Must-not-regress: the four pre-existing anti-vacuity courts
(strict/doc/layer/gap-flip) — all re-run exit 0 post-fix; exit-law
0/1/2/3 unchanged; default-mode behavior unchanged (no strict-only
refusal leaks into exit 0 runs). Other scripts/ files modified in the
tree (verify_cscb_sram.py, zero_string_gate.py, journal_mutation_matrix.py)
are other lanes' in-flight work — untouched by this lane.

## 7. Divergences

1. DF-05's control row ("KeyError('rule') --strict crash = open
   OUT-OF-CONTROL") was stale relative to the GAP-05 fix but the crash
   class was still reachable via non-claim scans; RC-09 closes the
   residual (typed guard) — DF-05's row may now be marked IN-CONTROL
   citing this receipt.
2. Contract said "missing rule under strict → exit 1 naming receipt +
   missing rule key" — implemented as RECEIPT-LINT-MISSING-KEY; default
   mode emits the loud row only (no exit-3; exit 3 stays bound to the
   existing no-lintable-dirs convention).
