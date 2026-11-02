# Lane FA10R — deployment-gate verdict rename (SG-FA10)

2026-11-02. Owner: `scripts/enclave_budget_gate.py` (stdout wording only),
this receipt. Closes FA10 fix (c) + fix (a) disclosure portion of
`docs/jira/v26.11.2/adversarial/forensic-lane-10.md` (fix (b), the binding
maximal-consumer arm, is a separate lane and NOT landed here).

## Change

Success-verdict stdout of `--profile-deployment` renamed from the
overclaiming "BUDGET GATE: all N linked deployment ELF(s) under the
rebalanced tier table and the 131072 B whole envelope" to the honest form:

- `MINIMUM ENTRY SURFACE FITS: all N linked deployment ELF(s) under the
  rebalanced tier table and the 131072 B whole envelope`
- a mandatory `COVERAGE CAVEAT (mandatory, FA10)` line: exercised-subset
  disclaimer, pointer to the per-crate coverage ratios, the FA10
  maximal-consumer 102187 B .text+.rodata witness (under ceiling, ~22%
  headroom, established by the FA10 probe NOT by the gate), the std-gated
  parquet scan absence, and the witness cite
  `docs/jira/v26.11.2/adversarial/forensic-lane-10.md`.
- per-crate coverage ratio `linked/audit-rlib %.1f%%` appended to each
  measured-subject row where computable, using frozen FA10 witness
  denominators `FA10_AUDIT_RLIB_BYTES` (blake3 114072, gate 1048,
  p28 9220, parquet 55799 B — FA10 session-measured, matching
  w6-budget-elf-profile.md) over the LIVE measured linked-ELF numerator.
  Constants are disclosure-only; they enter no exit/breach decision.

NO change: exit codes, budget tables, measurement logic, failure/skip
wording (`BUDGET GATE FAILED`, `DEPLOY-MUTANT COURT`, summary line).

## Before/after wording

BEFORE (success, 4 subjects):
`BUDGET GATE: all 4 linked deployment ELF(s) under the rebalanced tier
table and the 131072 B whole envelope`

AFTER (verbatim, this session's run):
`MINIMUM ENTRY SURFACE FITS: all 4 linked deployment ELF(s) under the
rebalanced tier table and the 131072 B whole envelope`
`COVERAGE CAVEAT (mandatory, FA10): this verdict covers only the exercised
subset of each subject — the minimal no_std entry snippets, not the whole
engine. Per-crate linked/audit-rlib coverage is printed above where
computable. FA10 maximal one-link no_std consumer = 102187 B
.text+.rodata (under the 131072 B envelope, ~22% headroom; established by
the FA10 probe, NOT by this gate; std-gated parquet scan is structurally
absent from every no_std image measured) — witness cite:
docs/jira/v26.11.2/adversarial/forensic-lane-10.md`

## Scenario exits (this session)

`python3 scripts/enclave_budget_gate.py --profile-deployment \
  --subject cgap-blake3=tier0 --subject cgap-gate=tier0 \
  --subject cgap-p28=tier0 --subject cgap-parquet=tier2 --deploy-mutant`
→ exit **0** (unchanged). Witnessed: per-crate coverage 31.1% / 57.8% /
9.6% / 4.0%; `DEPLOY-MUTANT COURT: mutant ELF 66176 B > tier0 budget
49152 B — breach flips (anti-vacuity holds)`; envelope 39184/131072,
verdict=OK. (gate 606 B vs FA10's 484 B is nightly-toolchain drift in the
live numerator; the ratio computes live, so 57.8% vs FA10's 46.2% is
expected and correct.)

`python3 -m py_compile scripts/enclave_budget_gate.py` → exit 0.
`pyright scripts/enclave_budget_gate.py` → 0 errors, 0 warnings.

## Divergences

- FA10's coverage figures (46.2% gate) used that session's linked-ELF
  bytes; this lane computes the ratio from live measurement over frozen
  audit-rlib denominators, so ratios can drift with toolchain. Intentional.
- FA10 fix (b) — a binding maximal-consumer court arm — remains open;
  this lane is wording-only by dispatch.
