# Forensic Lane FA4 — "BFX zero-copy landed" adversarial audit

Date: 2026-11-02. Presumption under test: BFX closure was management
appeasement — neither no_std nor zero-heap, termination = iteration cap
in disguise. Only file written: this report. All probes /tmp-only; zero
repo source edits.

## Verdicts (per suspicion)

| # | Suspicion | Verdict |
|---|---|---|
| 1 | unify.rs still clones/to_vecs the store | **REFUTED** (for cgap-bn3) |
| 2 | Termination is a cap disguised as convergence | **REFUTED** — exit is `new_rows == 0` |
| 3 | Derived facts dropped, caller never sees final set | **REFUTED** — caller receives every derived row |
| 4 | "Zero-copy landed" blanket claim | **SPLIT**: TRUE for cgap-bn3 unify; FALSE for cgap-storage closure |

## 1. BFX / allocation forensics (cgap-bn3 unify)

Static sweep, this session:

- `grep -c "to_vec" crates/cgap-bn3/src/unify.rs` -> **0**.
  The legacy full-store copy (receipt-cited former :1371-1373) is gone.
- W6-BFX markers present: SplitStore docs at unify.rs:414-441
  ("borrowed-prefix + owned-delta-tail store view"), driver marker at
  :1477-1499 (hoisted JoinProbeScratch, per-rule SplitStore view).
- Residual clone sites are NOT the store copy: :1754
  `binding.clone()` (per-row u64 slot save/restore in the legacy
  row-at-a-time arm, not the fixpoint path) and :2398 `emitted.clone()`
  (cfg(test) court code). No `to_vec()` anywhere in the driver.

Dynamic probe (/tmp/fa4-probe, counting GlobalAlloc over System,
window = `unify_to_fixpoint` call only, CARGO_TARGET_DIR=/tmp/fa4-target):

- 3-cycle cyclic workload (edges 1->2->3->1, path rules r0/r1):
  `rounds=3 batches=6 emitted=51 allocs=156 bytes=17196`, exit 0.
- 64-node ring (path closure, n=64, 7 rounds, 4160 facts incl. 64
  base edges): `rounds=7 allocs=2050 bytes=102929890`, exit 0.
- Shape of the claim: allocations scale with DELTA size (156 allocs for
  a 12-fact closure; ~103 MB churned by BTreeSet `seen` growth + owned
  delta/batch buffers on the ring), NOT with base-store size. The
  caller's columns are borrowed (SplitStore prefix), never heap-copied.
- Positive control: COUNTER_LIVE=true (counter moves on a fresh Vec) —
  the zero-alloc court machinery is not vacuous.
- Honest bounds: the driver is NOT zero-heap during convergence. The
  BTreeSet `seen` termination set, the owned delta tail (ds/dp/do_),
  and the accumulating `batches` Vec all allocate as deltas grow.
  What BFX removed is the per-round FULL-STORE COPY, and that removal
  is real.

## 2. Termination forensics

Driver loop (unify.rs:1507-1550, post-BFX layout):

- Exit condition is `if new_rows == 0 { return Ok(batches) }`
  (unify.rs:1544-1546) — SEMANTIC convergence on the derived set.
- The round budget (DEFAULT_MAX_ROUNDS = 64, unify.rs:188) is a typed
  REFUSAL, not a disguised exit: `FixpointBudgetExceeded { rounds }`
  (unify.rs:1547; variant at :265) — returns Err, no partial set, no
  silent "close enough".

### Cyclic-rule probe (3-cycle, public RuleCompiler API)

Built `path(X,Y) :- edge(X,Y)` + `path(X,Z) :- path(X,Y), path(Y,Z)`
over a literal 3-cycle (1->2->3->1), rules compiled via
`RuleCompiler` inside `unify_to_fixpoint`; termination checked two
ways:

1. Driver returned Ok after **3 rounds** — no budget hit (budget 64).
2. INDEPENDENT naive oracle (separate BTreeSet fixpoint computed in
   the probe, seeded edge->path then closed over pred 99):
   driver final set == oracle, **MATCH=true**, both 12 facts
   (9 path pairs incl. the cycle wrap, 3 base edges).
3. Anti-vacuity flip (`fa4-probe flip`): same cyclic set with
   `max_rounds=2` -> `FLIP ok=true budget_exceeded rounds=2` — the cap
   refuses LOUDLY rather than faking convergence. A cap-only engine
   would have returned a still-growing set; it errors instead.

**Verdict: termination is semantic convergence, not a disguised cap.**

### 64-node ring (bench-shape workload)

`fa4-probe chain 64`: rounds=7 (2^k hop-doubling to cover the 64-hop
cycle), driver Ok, result superset consistent with closure semantics
(4160 = 4096 path pairs + 64 base edges). No cap contact.

## 3. Delta destination

Post-BFX return type: `unify_to_fixpoint* ->
Result<Vec<DeltaBatch<DeltaRows>>, UnifyError>` (unify.rs:1461-1463,
driver :1466-1550). The internal `ds/dp/do_` delta tail IS dropped at
fn exit, but every derived row is ALSO accumulated into `batches`
(`batches.push(batch)`, unify.rs:1538) which is returned. My probe
reconstructed the exact closure from `batches` ALONE and it matched
the independent oracle — **the caller receives the final fact set**;
nothing derived is lost. (Re-derivations are re-emitted per round;
dedup is A15's division, and the driver's own `seen` set guarantees
append-only growth.)

## 4. cgap-storage closure.rs — the other engine

- `grep -rn BFX crates/cgap-storage/src/` -> **zero matches**. BFX
  never touched this crate. One commit total on closure.rs
  (a9827591, the p1-closure lane landing).
- The MB4-measured 1,117 allocs / 1.20 GB figure (receipt:
  docs/jira/v26.11.2/mega-bench/lane-04-datalog-closure.md:64-69,
  `N=1048576 best_secs=2.901682 rounds=31 allocs=1117
  bytes=1204813020`) is `semi_naive_closure` (closure.rs:94-136):
  fresh `next_x`/`next_z` delta Vecs allocated per round
  (:118-133), plus the std HashSet SeenSet (closure.rs:39 — the
  RandomState TLS blr=1 census finding, p2-asm-0blr-census.md:130).
- closure.rs is **std-only** (`std::collections::HashSet`, no
  `#![no_std]` attribute in the file; cgap-bn3 lib.rs:2-3 declares
  `#![cfg_attr(not(test), no_std)]` — the two engines differ).

### Plain statement (required by dispatch)

**"Zero-copy landed" is TRUE for the cgap-bn3 unify fixpoint engine
(SplitStore borrowed-prefix + owned-delta-tail; zero to_vec(); probe
allocation counts scale with delta, not store). It is FALSE for the
cgap-storage semi_naive_closure engine (fresh delta Vec per round,
std HashSet seen-set, 1,117 allocs / ~1.20 GB churned at N=1048576,
measured by MB4 and unmodified since).** MB4's own receipt states the
same split; a blanket "BFX zero-copy landed" headline over BOTH
engines would be management appeasement — scoped to unify, it is a
real, probe-confirmed repair.

## Commands and exits (this session)

- grep sweeps over unify.rs / closure.rs / waves receipts: exit 0/1
  as designed (0 matches = refuted claims).
- `/tmp/fa4-target/release/fa4-probe` (3-cycle default): exit 0 —
  rounds=3, MATCH=true, allocs=156, COUNTER_LIVE=true.
- `fa4-probe flip`: exit 0 — budget_exceeded rounds=2 (typed refusal).
- `fa4-probe chain 64`: exit 0 — rounds=7, allocs=2050 bytes=102929890.

## Standing

ALIVE for the probe-backed claims above (command + exit run this
session, exact subjects /tmp/fa4-probe against
/Users/sac/cgap/crates/cgap-bn3). MB4 closure.rs figures are cited
from the landed receipt, not re-measured this session (bench method
stated there) — those carry MB4's standing, marked as-cited.

## Divergences

- Dispatch cited cap at ":1411, :179"; post-BFX the exit/budget live
  at :1544-1547 and DEFAULT_MAX_ROUNDS at :188. Substance unchanged.
- Dispatch asked to confirm "zero to_vec() with W6-BFX ~:1470" —
  confirmed: 0 to_vec sites; W6-BFX driver marker at :1477-1499.
- Probe terminated the 64-ring at n=64 (512-ring was O(n^3)+ and
  exceeded the 120s window; not needed for any verdict — the 3-cycle
  cyclic case is the dispatch's falsifier and it converged).
- The anti-vacuity "mutation" for suspicion 2 is the flip probe
  itself (budget=2 refuses) — a court that cannot flip would be a
  FAIL; it flipped.
