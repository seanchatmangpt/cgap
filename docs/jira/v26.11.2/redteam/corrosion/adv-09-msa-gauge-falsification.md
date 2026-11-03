# ADV-09 — MSA gauge falsification (corrosion probe receipt)

Lane ADV-09, adversarial corrosion wave, 2026-11-02. Repo /Users/sac/cgap
@ main. Contract: READ-ONLY (zero bench runs needed — all probes were
disk reads); only file written is this receipt. No git commands run.
Fail-closed logging: every probe verdict below names the grep/Read that
produced it; a probe with no referent found is stated as such, never
rounded to a pass.

## W1 Probe 1 — batch-amortization honesty (41.0 ns floor vs 10.20 ns/frame)

**Verdict: REGISTERED-BOUND (limit admitted, one residual gap).**

The MSA docs admit the limit explicitly, twice:

- `dflss/03-msa-gauge.md:30-32`: "A ~10-23 ns per-frame quantity is
  smaller than the clock's own granularity: literal per-frame `Instant`
  timing measures the instrument, not the process."
- `dflss/09-ax-cpk-msa.md:15-16`: "every per-frame nanosecond number in
  this campaign is a batch-amortized gauge reading, disclosed as such
  wherever it appears."

The 10.20 ns/frame figure is NOT claimed as individually resolvable
anywhere — it is always introduced as the batch/window quotient
(1,048,576-frame const, divan whole-batch ms, divided; method stated in
11-measured-claims.md row 5 and 06-bench-protocol §1).

**Residual gap (corrosion finding ADV-09-R1):** no doc states that the
per-frame TAIL within a window is equally unresolvable. Row 28's
"slowest 124.06 ns/frame" is the slowest whole 1,048,576-frame SAMPLE
amortized — a within-window frame-level tail could be far worse and is
structurally invisible to the gauge. The docs admit the mean-level
limit but never extend the admission to the tail-level limit. Any
consumer reading 124.06 as a per-frame worst case is over-trusting the
gauge by exactly the batch factor. Not a violation (the amortization
disclosure is the registered law and covers the mechanism); a
disclosure-scope gap.

## W2 Probe 2 — N=7 tail defensibility (max-of-7 as tail SLA)

**Verdict: IMMUNE.**

- The quantile-honesty law already forecloses p99-of-7 labeling:
  `03-msa-gauge.md:66-69` (max-of-7 is not a quantile;
  `lanes/lane-fix04-cscb-p99.md:4-12`).
- No gate keys on max-of-7. The C3 closure gate keys on MIN:
  "PASS min 36,918.401 ms <= 60,000 ms" N=7
  (`lanes/lane-mv-05-sla-reconcile.md:24`); max is reported loudly
  (42,674.974 ms, :37), never gated.
- Every quiet-host tail-adjacent claim carries the row-28 divergence
  cite: rows 5/8 (`11-measured-claims.md` rows 5, 8, 28), CTQ
  M-CTQ-01 (`dflss/02-voc-ctq-matrix.md:198-207`: "contention row 28
  ... is a REGISTERED DIVERGENCE, never quoted unqualified").
- Searched for a tail SLA drawn from max-of-7 without the divergence
  cite: ZERO hits. New N=7 rows 30/31/31a/32 report max loudly and
  gate/pass on min or median; none labels max a percentile and none
  draws an SLA from it.

Fallback floor: even if a future row drew an SLA from max-of-7, the
row-28 registration is attached at the CTQ layer (M-CTQ-01), so the
fence exists upstream of any such claim.

## W3 Probe 3 — anchor-fix verification (cscb_c3_closure.rs)

**Verdict: IMMUNE.**

Fresh Read this session of `crates/csct/src/bin/cscb_c3_closure.rs`:

- `RUNS: usize = 7` — line 52. Confirmed on disk.
- `DECIDED_BUDGET_MS: f64 = 60_000.0` — line 40.
- `ASPIRATIONAL_MS: f64 = 25.0` — line 42.
- `PRD_ASPIRATIONAL_MS: f64 = 2.0` — line 48 (kept strictly separate
  from ASPIRATIONAL_MS, per its own doc comment :45-47).

The R-09 errata block (`dflss/03-msa-gauge.md:185-209`) discloses the
prior wrong anchor (:40-42 carried the budget consts, not RUNS) and the
corrected cite. The correction is verified true on disk today: the
anchor fix is real, disclosed, and stable. This probe is a re-read
corroboration of R-09's errata item 1 — no new defect.

## W4 Probe 4 — CONTENTION-SUSPECT guard wiring (RC-11)

**Verdict: SPOILER-CONFIRMED (doc-layer dangling enforcement).**

The guard exists and flips (RC-11's own receipt is exemplary: three
iterations because it failed to fire, flip margin 2.1-30x,
`lanes/lane-rc11-contention-guard.md:30-34,62-67`). Downstream N=7 rows
30/31/31a/32 in `11-measured-claims.md` DO cite "contention guard quiet"
as method. But the enforcement rule is dangling at the document layer:

- Repo-wide docs grep for `CONTENTION-SUSPECT`: the ONLY hit is RC-11's
  own lane receipt. ZERO hits in `dflss/03-msa-gauge.md`,
  `dflss/06-control-plan.md`, `dflss/09-ax-cpk-msa.md`,
  `dflss/02-voc-ctq-matrix.md`, `06-bench-protocol.md` (greps exit 1).
- The consumption rule — "CI/consumers MUST grep stderr for
  CONTENTION-SUSPECT and refuse to admit a marked run" — lives ONLY as
  comments in `crates/bench/benches/chains-ingress.rs:716-802` (and the
  mctq bench copy). Prose in a bench file, enforced by NO gate.
- CTQ M-CTQ-01 (`02-voc-ctq-matrix.md:185,198-207`) registers the row-28
  divergence but names no enforcement mechanism for it.

Attack this enables: a hostile/noisy run quoted against rows 5/6/7/8
carries NO machine-checkable refusal — nothing in the control plan
fails, no gate exits nonzero. The guard is a tripwire with a sign
nobody is required to read. Control-plan wiring gap: the MSA/control
docs must name the CONTENTION-SUSPECT stderr marker as the row-28
enforcement mechanism and gate on its absence, or the registration is
honor-system.

## W5 Standing + fences

- **Standing: ALIVE.** Every verdict above is grounded in greps/Reads
  run THIS session against the exact on-disk subjects (all cited
  file:line re-read; grep exits stated). Zero bench runs; contract
  read-only honored.
- Fences this receipt could break: none mechanical (docs-only, single
  new file, no shared-tree edits). The nearest fence is receipt_lint
  --strict claim-proximity law — every numeric claim here carries its
  receipt cite in the same sentence-class proximity.
- No ERRATA against prior receipts: R-09's errata items were
  independently corroborated (probe 3), not contradicted.

```json
{"lane":"ADV-09",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/redteam/corrosion/adv-09-msa-gauge-falsification.md"],
 "commands":[
  "Read dflss/03-msa-gauge.md (exit 0)",
  "sed 11-measured-claims.md 1-80 + rows 30-32 tail (exit 0)",
  "sed red-10-timing-jitter-exploit.md 1-100 (exit 0)",
  "sed cscb_c3_closure.rs 35-70 (exit 0; RUNS at :52 verified)",
  "Read lanes/lane-rc11-contention-guard.md (exit 0)",
  "grep CONTENTION across benches (exit 0)",
  "grep CONTENTION-SUSPECT over dflss/ + 06-bench-protocol.md (exit 1 — zero hits: probe-4 evidence)",
  "grep CONTENTION-SUSPECT over docs/ repo-wide (1 hit: rc11 receipt only)",
  "grep mv-05 max/tail/p99/RUNS/gate (exit 0)",
  "sed 09-ax-cpk-msa.md 1-60; sed 02-voc-ctq-matrix.md 180-210 (exit 0)",
  "grep mctq over 11-measured-claims.md (rows 30-32 present, exit 0)"],
 "courts_run":[
  "P1 batch-amortization disclosure audit — REGISTERED-BOUND (ADV-09-R1 residual)",
  "P2 max-of-7 tail-SLA sweep — IMMUNE (zero unregistered tail claims)",
  "P3 anchor-fix re-read — IMMUNE (RUNS=7 at :52 confirmed)",
  "P4 guard-wiring docs grep — SPOILER-CONFIRMED (dangling enforcement)"],

 "standing":"ALIVE",
 "divergences":[
  "ADV-09-R1: amortization disclosure covers batch means but never",
  "  states the within-window per-frame TAIL is unresolvable; row-28",
  "  124.06 'slowest' is a batch-amortized read, not a frame tail",
  "ADV-09-R2 (probe 4): CONTENTION-SUSPECT enforcement is prose in",
  "  bench headers only; no control-plan/MSA doc names it and no gate",
  "  refuses a marked run — control wiring SPOILER-CONFIRMED dangling",
  "no bench run needed; read-only contract honored (0 legs)"]}
```
