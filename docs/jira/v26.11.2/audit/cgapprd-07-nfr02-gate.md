# cgapprd-07 NFR-02 closure gate audit

Lane: CGAP-PRD-7 (implement, verify-and-extend), 2026-11-02, repo
`/Users/sac/cgap`, branch main. No git write commands.

## W1 Subject

`crates/csct/src/bin/cscb_c3_closure.rs` -- the CSCB-C3 1M-fact
transitive-closure SLA driver (MV-5 gate). MV-5's landed consts
(`DECIDED_BUDGET_MS=60_000.0`, `ASPIRATIONAL_MS=25.0`) were read fresh
and already present; this lane verified-and-extended, never duplicated.

## W2 Diff

- Added `PRD_ASPIRATIONAL_MS: f64 = 2.0` with a provenance comment
  citing the PRD's "<2.0 ms closure" claim
  (`docs/jira/v26.11.2/00-ard-prd.md`) and this audit receipt. It is
  REPORTED-NEVER-GATING by construction: it appears only on the
  loud-report path, never in the exit-code decision.
- Wired `PRD_ASPIRATIONAL_MS` into the DECIDED-GATE PASS verdict line so
  the PRD's 2 ms figure is loudly reported alongside the 25 ms CSCB
  aspirational figure, each with its measured multiple.
- No change to `DECIDED_BUDGET_MS`, `ASPIRATIONAL_MS`, `CORPUS_SEED`,
  `SM64_MULT`, corpus shape, or any gate/exit logic.

## W3 Commands + exits (this session)

| command | exit |
|---|---|
| `cargo build --release --bin cscb_c3_closure` (in `crates/csct`, `CARGO_TARGET_DIR=/tmp/cgapprd7-target`) | 0 |
| `/tmp/cgapprd7-target/release/cscb_c3_closure --self-gen` | 0 |
| `CSCB_MUTATE=radix-clamp CSCB_RUNS=1 .../cscb_c3_closure --self-gen` | 1 (flip by design) |
| `cargo test --release` (full csct suite, `crates/csct`) | nonzero (one pre-existing FAIL, below) |

## W4 Gate witness (ALIVE)

Baseline run (self-gen, 1,000,000 edges, seed 0xA20C_0FFE_E5A7_E000,
release profile, N=7 min-of-N, mutate=false), fresh this session:

```text
CSCB-C3 closure=1500000 baseline=1500000 batches=2 min=41563.196ms median=46486.917ms p99=56660.840ms
CSCB-C3 VERDICT DECIDED GATE PASS min=41563.196ms <= 60000ms (decided budget, lane MV-5 re-scope; ASPIRATIONAL 25ms NOT met: measured min is 1663x the aspirational figure; PRD_ASPIRATIONAL 2ms NOT met: 20782x, ASPIRATIONAL-UNVERIFIED -- both reported loudly, neither gates)
```

Gate PASS exit 0.

## W5 Mutant flip (anti-vacuity)

`CSCB_MUTATE=radix-clamp CSCB_RUNS=1` (8-bit clamp of the join-key
partition mask on s/o):

```text
CSCB-C3 closure=4837284 baseline=1500000 batches=2 min=170829.750ms median=170829.750ms p99=170829.750ms
CSCB-C3 VERDICT COURT-FLIPPED: radix-clamp closure differs from baseline as required
```

Exit 1 as designed (closure differs from baseline; the mutated run is a
typed non-zero outcome, not a gate pass). The `zero-tax` alias routes to
the same mutation branch in the bin and was not separately executed this
session -- UNVERIFIED as an independent run.

## W6 Three-layer truth (NFR-02)

1. **In-kernel witness**: 22.99 ms (release receipts, prior session;
   UNVERIFIED-as-gate per EXP-03 -- a prior-session measurement is
   hearsay at gate time; this session's witness is 41.56 s min on the
   C3 corpus under concurrent-lane load).
2. **Membrane-measured**: ~23 ns/fact. POSH-2 batch amortization court
   reports min=23.31 / median=23.48 ns/frame this session AND a typed
   GATE FAIL against its 10.0 ns AMORTIZED limit -- POSH-2 standing is
   FAIL, not PASS (the per-fact number is real; the limit is not met).
3. **PRD target**: <2.0 ms closure, ASPIRATIONAL-UNVERIFIED. No kernel
   path to 2 ms exists on this corpus (measured min is ~20,782x the
   target). Unblock prerequisites: (a) real kernel work on the fixpoint
   join (the 60 s DECIDED budget is the only measured, gated contract),
   (b) corpus definition -- the PRD does not pin the corpus, size, or
   shape its 2 ms claim is measured against (the 24-25 ms EXP-03
   figure was measured on a DIFFERENT corpus). Until both land, the
   2 ms figure is reported loudly (`PRD_ASPIRATIONAL_MS`) and never
   gates.

## W7 Fences + standing

Fences the diff could break: the MV-5 DECIDED gate itself (ran, PASS,
exit 0) and the csct suite. Suite census this session: 74 passed /
1 failed across all targets; the one failure is
`tests/posh_batch_court.rs::posh_batch_amortization_court` --
PRE-EXISTING and named as a typed FAIL in the lane contract; the diff
touches only `src/bin/cscb_c3_closure.rs`, which that test does not
link, so the failure is not session-introduced. Standing: gate ALIVE
(command+exit above), PRD 2 ms target ASPIRATIONAL-UNVERIFIED, POSH-2
FAIL (typed, pre-existing). Build root lease `/tmp/cgapprd7-target`
deleted at lane end.

```json
{"lane":"CGAP-PRD-7","files_written":["crates/csct/src/bin/cscb_c3_closure.rs","docs/jira/v26.11.2/audit/cgapprd-07-nfr02-gate.md"],"commands":["cargo build --release --bin cscb_c3_closure:0","cscb_c3_closure --self-gen:0","CSCB_MUTATE=radix-clamp run:1 (flip)","cargo test --release: nonzero (pre-existing POSH-2 FAIL)"],"courts_run":["CSCB-C3 DECIDED gate (PASS exit 0)","CSCB-C3 radix-clamp mutant (COURT-FLIPPED exit 1)"],"standing":"gate ALIVE; PRD 2ms ASPIRATIONAL-UNVERIFIED; POSH-2 typed FAIL pre-existing","divergences":["csct suite not fully green: posh_batch_amortization_court pre-existing typed FAIL (per lane contract); disclosed, not session-introduced"]}
```
