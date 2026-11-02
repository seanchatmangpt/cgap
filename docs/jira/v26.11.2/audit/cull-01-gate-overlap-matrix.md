# cull-01-gate-overlap-matrix

CULL-01 gate-script overlap matrix (2026-11-02, read-only lane).
Scope: scripts/*gate*.py + receipt_lint.py +
tests/gates/enforce_coverage_floor.py (correction: not under scripts/).
13 scripts inventoried; flip witnesses cited per script.

## Subject x script SAME-invariant rows (the only overlaps)

1. Dep graph: boundary_gate --deps (E=0 externals) vs
   zero_datafusion_gate P1/P2/P3 (*datafusion* needle) —
   OVERLAP-PARTIAL: predicate differs; zdf also scans
   [workspace.dependencies] latent opt-ins; boundary is
   covered-crates-scoped. Not VERIFIED-REDUNDANT.
2. Test results: gate_cargo_test.py vs test_census.py —
   OVERLAP-PARTIAL: census classifies COMPILE-ABORT vs FAIL
   per target; gate_cargo_test owns the quota + exit-3
   evasion law. Neither subsumes the other.
3. Receipts: receipt_lint vs floor's probe-1 — liveness of
   the subsumer, not duplicate coverage.

Everything else UNIQUE-COVERAGE: string surface split
(sources vs compiled artifacts), panic symbols, binary size,
throughput, timing, coverage each have exactly one gate.

## zero_datafusion_gate.py: NOT a ghost

The subject is the live whole-graph NEGATIVE invariant
post-removal (removal receipt cited at :16; the needle is the
bare string "datafusion" over cargo metadata/tree/manifests —
it never names cgap-datafusion). This is the regression-sealer
pattern: the gate is what keeps the removal removed.
BUT it is LOUD-BLOCKED: --self-check exits 1 (VACUOUS) on the
pre-existing vendor/nanoarrow symlink loop (lane-c06 ERRATA).
Fix = one line in copy_subject's ignore set (:194) — laned.

## Verdicts (13 scripts)

12 KEEP (each UNIQUE-COVERAGE with a named flip witness;
bench_gate, boundary, zero_string, zero_datafusion KEEP+fix,
symbol_gate, enclave_budget, timing_gate, gate_cargo_test,
gate3_schema, receipt_lint, test_census, enforce_coverage_floor).
1 MERGE: gate3_cdata.py -> gate3_schema.py (VERIFIED-REDUNDANT
at assert level: schema gate re-proves bitwise buffers per
config as its check 4; same cdylib; fold cdata's wire-parity
set as schema config 0, then `git rm scripts/gate3_cdata.py`;
re-open trigger: any C-Data export ABI change or schema gate
dropping check 4).

Net: no VERIFIED-REDUNDANT deletions available. Two laned
fixes: (a) zdf self-check symlink guard; (b) coverage floor
red (follow-on per lane-c06).

## See Also

audit/cull-05-ghost-subjects.md · lanes/lane-c06-scripts-gate-coverage.md
