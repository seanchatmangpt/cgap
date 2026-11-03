# pub-10-claims-preflight

PUB-10 claims preflight (2026-11-02, read-only): every
crates/*/Cargo.toml description + README perf sentence vs the
landed receipts. Honesty gates: 10 ns/frame carries
"kernel-layer, batch-64 amortized, release-only"; >4M seals/s
carries "uncontended-only, machine-dependent"; closure = DECIDED
budgets (C3 60 s / C6 150M f/s); 5.70 ns penstock REFUTED
(tps-01), unqualified use forbidden.

## Headline: public-surface exposure

| phrase | in crates/* public surface | verdict |
|---|---|---|
| 5.70 ns/frame | NOWHERE | PASS |
| 10 ns/frame unqualified | NOWHERE | PASS |
| >4M / 4.35M seals/s | NOWHERE | PASS |
| C3 60 s / C6 150M f/s | NOWHERE | PASS |

Residual offender (non-registry): REVENUE_GENERATION_COMPENDIUM
:59/:237/:288 still carry bare 5.70/5.79 figures — errata 2
records the retraction but the body rows are uncorrected. Rewrite
to 7.71 receipted / 7.35 fresh, or strike (joins CULL-08's list).

## Per-crate description table

22/25 pass receipt-backed. Gaps: cgap-court, cgap-cli, csct ship
NO description (add pre-publish — PUB-01/PUB-02 surface).
Partial: cgap-blake3 "8-way AVX2" — NEON witnessed but AVX2 path
opt-gated with 26.11.3 republish owed (dod-cgap-blake3:91); add
the qualifier or drop until re-witnessed.

## README perf-sentence table

bench README: receipt-cited; append freshness qualifier to the
102.7 ms row ("receipted 10.1x; fresh 9.6x, tps-01:24" — drifted).
cgap-parquet README rows receipt-cited, keep. cgap-gate README's
0-alloc sentence is as-of-date (lib-test compile-aborts 22 errors
at 26.11.2 — banner already discloses; qualifier recommended).
All others: no perf sentences or receipt-cited, keep.

## Verdict

No unqualified 5.70 ns, bare seals/s, or bare 10 ns/frame on any
public crate surface. The registry-facing text is clean; the
offenders are all in docs/ prose (CULL-08 owns).

## See Also

audit/tps-01-reality-ledger.md · audit/tps-08-shipgate-plan.md ·
audit/cull-08-doc-retirement.md
