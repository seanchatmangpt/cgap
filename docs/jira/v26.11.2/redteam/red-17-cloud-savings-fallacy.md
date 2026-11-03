# RED-17 — Cloud Egress & Compaction Fallacy (adversarial red-team)

Lane: RED-17 · 2026-11-02 · READ-ONLY over /Users/sac/cgap (single receipt file written).
Dispatch named `11-measured-claims.md`, `00-ard-prd.md`, `02-pareto-levers.md` as primary
targets. All were read in full this session.

## W1 — Where the numbers live

Searched the entire repo for "90% cloud-cost reduction" and "99% data-reduction":

- **No on-disk referent exists for either figure.** Repo-wide grep over `docs/` +
  `README.md` for `90%` / `99%` / `cloud cost` / `data reduction` returns zero rows
  carrying either claim. The 90% hits on disk are a churn-workload mix parameter
  (`00-ard-prd.md:65`, `01-release-gates.md:128,133,135` — 90% of facts
  retracted+reinserted), a coverage floor
  (`lanes/lane-c06-scripts-gate-coverage.md:51`), and the bn3 unify stage-share
  (81.2%→99.9%, `README.md:100`) — none is a cost or storage-reduction claim.
- The actual economic claims live in `docs/REVENUE_GENERATION_COMPENDIUM.md` §6.3
  (lines 306-311), not in the campaign's gated docs.

## W2 — Grading the economic claims (REVENUE_GENERATION_COMPENDIUM.md §6.3)

### 6.3a ">75% core-hour reduction" (:308) — BREACH-CONFIRMED (claim hygiene)

Extrapolated from a 4.06x bitslice range-scan microbenchmark (:281) + 26.7% RLE
decode speedup (:284). No method-of-record row exists in
`docs/jira/v26.11.2/11-measured-claims.md`; that file's header (:3-8) states
"No number may appear in the paper that is absent from this table" with method +
receipt cite required. The compendium carries neither: it is TARGET-class
economics written as settled fact. Per the repo standing vocabulary
(`CLAUDE.md` Conventions; `11-measured-claims.md:15-18`), perf/econ numbers are
TARGETs until measured with a stated method.

### 6.3b "$4.5M–$5.2M/year net hard-dollar cloud savings" + "4.5x–5.2x ROI" (:311) — BREACH-CONFIRMED (claim hygiene AND economically false end-to-end)

Claim-hygiene arm: same absence from the measured-claims table; a dollar figure
is stated with zero method, zero receipt, zero run.

Economic arm — the 90%-style byte/speedup→dollar linear transfer fails on
pricing mechanics even if every microbenchmark is granted:

1. **Quantized compute sizing.** The model's own base is 500x c6i.8xlarge
   (:307). EC2 families step in coarse granularity; a >75% core-hour cut in one
   stage does not retire 375 of 500 nodes — work consolidation, fleet
   floor-size, and HA headroom bound realized savings well below the linear
   figure.
2. **Minimum-billing warehouses (Snowflake).** Snowflake bills per-credit with
   a per-warehouse minimum size and 60-second minimum billing per query; a
   90% smaller scan on an X-Small warehouse that already fit in the minimum
   saves ~0 dollars. Byte reduction converts to dollar reduction only above
   the warehouse-size cutover.
3. **Minimum cluster sizing (Databricks).** SQL warehouses bill serverless
   with minimum billing intervals and clusters have minimum-size floors;
   same truncation of the linear transfer.
4. **Ingestion-based licensing (Datadog-class).** Priced per host/GB ingested,
   entirely insensitive to compute latency or heap allocation; the "0-byte
   heap → eliminate 30-40% RAM headroom" lever (:309) moves its bill by zero.
5. **Egress is untouched.** The claimed levers are query/decode speed and heap
   elimination. Cross-AZ / cross-region egress is a function of bytes moved
   and topology, not of in-process decode speed; no lever in §6.1 reduces
   bytes on the wire, so egress-dominated bills are unmoved.
6. **Benchmark-to-bill category gap.** The rows feeding the ROI model are
   ns/µs single-core microbenchmarks (divan, 1k-10M element workloads,
   :281-294). 06-bench-protocol.md's own unit law
   (`11-measured-claims.md:52-58`) rules frame/byte-counter rows
   CATEGORY-ERROR if compared to unlike counters; a cluster-year dollar
   figure is a further category jump with no bridging measurement.
7. **Precedent on-disk: the wrong-unit fallacy already fired.**
   `adversarial/THE-SMOKING-GUN-REPORT.md:31` cracked "943M facts/s
   retraction" as bitmask-scan speed sold as a retraction-engine figure. §6.3
   repeats that shape at the dollar layer: benchmark speed sold as fleet
   spend.

### 6.3c "0-byte heap eliminates 30-40% RAM headroom" (:309) — BREACH-CONFIRMED (mechanism mismatch)

The 30-40% JVM/Go GC headroom is a property of the JVM/Go runtime, not of
Rust-side alloc counts; replacing the runtime eliminates the headroom by
substitution, not by the 0-alloc benchmark result. The 0-alloc rows are real
(MEASURED in v26.11.1 receipts) but do not entail the headroom claim.

## W3 — What survives

- `02-pareto-levers.md:9-10` — explicitly labels every lever target a TARGET
  until re-measured. Survives.
- `11-measured-claims.md` §1 MEASURED rows — carry method + receipt cites;
  survive as scoped. Its §2 unit law (:52-64) is the correct fence the
  compendium violates.
- `00-ard-prd.md:65` 90% — churn mix parameter, not a cost claim. Survives.
- Compendium §6.1 microbenchmark rows themselves — graded MEASURED in
  v26.11.1 receipts; this lane does not re-audit those (scope: economics),
  though :286's "82.24 GB/s reasoning" was already flagged category-error-adjacent
  by `11-measured-claims.md:56`.

## W4 — Verdict summary

| Claim | Location | Grade |
|---|---|---|
| "90% cloud-cost reduction" | dispatch target | UNSUPPORTED — no on-disk referent |
| "99% data-reduction" | dispatch target | UNSUPPORTED — no on-disk referent |
| ">75% core-hour reduction" | REVENUE_GENERATION_COMPENDIUM.md:308 | BREACH-CONFIRMED (TARGET-only economics stated as fact; absent from measured-claims table) |
| "$4.5M–$5.2M/yr savings, 4.5x–5.2x ROI" | REVENUE_GENERATION_COMPENDIUM.md:311 | BREACH-CONFIRMED (unmeasured dollar claim + pricing-mechanics fallacy: min-billing truncation, ingestion licensing, egress untouched, category gap) |
| "30-40% RAM headroom eliminated" | REVENUE_GENERATION_COMPENDIUM.md:309 | BREACH-CONFIRMED (mechanism mismatch) |
| Lever targets as TARGETs | 02-pareto-levers.md:9-10 | SURVIVES |

Standing: **ALIVE** — all reads and greps executed this session against
/Users/sac/cgap; no benches run (lane is read-only economics audit; no
mutation court applies to prose docs, so none was attempted — a skipped court
does not exist here because the subject is documentation, stated per loud-skip
law).
