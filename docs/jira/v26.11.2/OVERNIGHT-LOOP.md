# OVERNIGHT-LOOP — Governing Runbook (v26.11.2)

Charter: CHARTER-CGAP-STARDF-v26.11.2. Lane: V2-14 OVERNIGHT-LOOP-CHARTER
(lane-runner). Repo: /Users/sac/cgap, branch main (ONE canonical
checkout). Lease: this file + `waves/v2-14-loop.md` (receipt). No git
writes performed or permitted by this lane.

Entry documents, read in order:
`00-ard-prd.md` (charter transcription) -> `05-wave-plan.md` (wave plan
of record) -> `02-pareto-levers.md` (lever specs; all five levers are
[TARGET-v26.11.2] — no lever has a landed waves/ receipt yet, verified
by waves/ sweep 2026-11-01).

## 1. Opening law

The loop executes from disk, not operator memory. On (re)start:

1. Read `HANDOFF.md` if present; else record
   `HANDOFF: MISSING at loop start` in the iteration log and proceed
   from the landed state below. HANDOFF absence is loud, never silent.
2. Read `00-ard-prd.md` -> `05-wave-plan.md` -> lever specs.
3. Read the newest waves/ receipts to establish current standing.

Landed state, re-verified on disk 2026-11-01 (W1-09): `HANDOFF.md`
EXISTS at the campaign root; the recovery clause below fires only on
real absence. `waves/` holds 26 receipt files (ls exit 0, this
session): bp-{1-claude, 2-settings, 3-hooks, 4-rules, 5-skills,
6-reviewer, 7-stardf-claude, 8-allowlist, 10-audit}.md,
overnight-readiness.md, sm-{1-root, 2-settings, 3-agents, 4-hooks,
5-audit, 6-index}.md, v2-{01-charter, 02-gates, 03-levers,
04-stardf-closure, 05-wave-plan, 07-census, 10-cli-state, 12-stop-list,
13-commit-manifest, 14-loop}.md. Root docs: 00-ard-prd.md,
01-release-gates.md, 02-pareto-levers.md, 03-ost-design-around-register
.md, 04-stardf-closure.md, 05-wave-plan.md, RESTART-NOW.md,
OVERNIGHT-PROMPT.md (the W1-W10 dispatch layer; read it for lane
leases), OVERNIGHT-LOOP.md (this file). Gate names in
01-release-gates.md (G1-PUBLISH ... G8-STARDF-F1) are the dispatch
surface; 00-ard-prd.md §4 one-liners are the intent.

## 2. Wave order (serialized; lanes within a wave concurrent, disjoint leases)

### V2-A — finish the v26.11.1 tail (FIRST; blocks V2-B)

Publish state, census-current per `waves/v2-07-census.md`: 19/22
cgap-family crates registry-verified PUBLISHED at 26.11.1 (17 of the
v4 table + fibo + cloud; cgap-integration still HTTP 404 with its
structural blocker cleared). RESTART-NOW.md's 18/22 census is
SUPERSEDED (its own header says so, 2026-11-01). Remaining:

1. cgap-integration publish: ONE invocation, sequenced post-window;
   stop-on-slide + 1hr backoff (§4 publish law).
2. Lawful non-publishes (never retried): cgap-court (test-only),
   cgap-beam (guarded, no receipt), cgap-cli (delisted), bench (name
   collision).
3. stardf F1 mappings -> cgap frame / bont scalar bindings.
4. License gate: pinned URN
   `urn:avec:license:p28-sil:v2.1.0-sovereign` present in the P28
   attestation path; gate exit 0.
5. std-strip probe: no_std build of the five kernel crates
   (`cgap-{arrow,blake3,git,parquet}` + bont scalars); divergence =
   typed FAIL.
6. 22-crate sweep census (compile-abort reported separately from test
   failures; census-vs-abort rule).

Gate exits required before V2-B: workspace build 0, license gate 0,
sweep census 0 (or loud exit-3 skips with named reasons).

### V2-B — five Pareto levers (SECOND; looped)

One lane-runner per lever; each iteration lands exactly ONE lever,
flips its court, records its Divan PARETO row, then the next iteration
starts. Loop until G2-PARETO green (all five: flip witnessed + PARETO
row admitted, gate exit 0). A lever that cannot flip blocks only its
own iteration with a typed BLOCKED row; the loop proceeds to the next
lever. Skips are loud: `[SKIP] <reason>`.

Lever -> crate -> anti-vacuity flip -> Divan PARETO (from
05-wave-plan.md and 00-ard-prd.md §3; targets are
[TARGET-v26.11.2]):

- L1 ingress: two-stage digest escalation, cgap-arrow/penstock seam.
  Flip: corrupt a slot byte / stage code; per-stage checksum assert
  fires and escalation refuses a spoliated frame. Target <15 ns/frame
  (vs 102.7 measured).
- L2 SWAR FNV: word-wise u64 SWAR on the cgap-core hash path. Flip:
  corrupt one SWAR lane; hash differs. Target <10ms@1M (vs 41.54).
- L3 PLNF semi-join + noalloc radix probe (bont/storage join). Flip:
  drop a radix probe partition; result changes. Target >20x@1M
  (vs 81.2% nested-loop share).
- L4 ring sink: pre-leased ring telemetry sink (storage; sovereign.rs
  File::create share). Flip: overwrite one ring slot; tail reads it.
  Target <4us/tx (vs 45.8).
- L5 fused governance: fused ASC450 into P28 fold (gate+p28). Flip:
  mutate one fused branch; verdict differs. Target <1.45us/row
  (vs 2.37).

Divan PARETO: before/after on the same subject, same method (divan,
min-of-N, sleep-free); the PARETO row is admitted only if the measured
delta is in the claimed direction; otherwise the lever lands UNVERIFIED,
never rounded up.

### V2-C — positioning + release (THIRD)

1. Like-for-like mat bench vs RDFox at the 2-6.1M triple scale,
   identical workload shape, same hardware, stated method; perf numbers
   are TARGETs until this lands (MW4/G4).
2. QEMU stack paint: full no_std stack exercised under QEMU; result
   recorded as receipt; divergence = typed FAIL (G5 paint input).
3. position-paper-v2 grounded: every RDFox comparison row cites a
   measured receipt from item 1; no ungrounded capability claims.
4. 26.11.2 version bump + tiered publish: kernel crates first
   (`cgap-{arrow,blake3,git,parquet}`), then moonshot crates, then
   membranes (`ffi`, `beam`, `gpu`), then `cgap` umbrella. Each tier
   publishes only after the prior tier's court exits 0.

## 3. Gates G1-G8 (AFTER the three waves, in order, first-hand witnesses)

Authoritative spec: `01-release-gates.md` (COMMAND/RECEIPT/FLIP/BLOCKED
per gate). The loop runs each gate first-hand, exit recorded. Exit-code
contract: 0 pass / 1 fail / 2 subject missing / 3 loud skip. Mutations
only in /tmp sandbox copies, sha256 fail-closed restore. Order:

1. G1-PUBLISH — release_check over the 19/22 publishable set
   (court/cli lawfully delisted, beam guarded); version must read
   26.11.2 (observed 26.11.1 on disk — gate self-fails until the bump).
2. G2-PARETO — five lever thresholds (L1 <15ns/frame, L2 <10ms@1M,
   L3 >20x@1M, L4 <4us/tx, L5 <1.45us/row), each with a witnessed
   kernel flip; threshold met without a flip = `VACUOUS`, exit 3.
3. G3-MAT-BENCH — like-for-like materialization vs RDFox 2-6.1M
   facts/s envelope, >50M closure facts/s/core target (bench target
   currently absent in crates/bench — BLOCKED path documented).
4. G4-DRED — 90% churn protocol, <3.0ns/batch, <0.15ns entropy
   variance, >15M facts/s, N>=9 batches (bench absent — BLOCKED path).
5. G5-SILICON — riscv32imc no_std build + QEMU stack paint, flash
   <32,768B, SRAM+stack <4,096B, 0 alloc/panic symbols.
6. G6-NO-DF — zero datafusion hits across cgap + purrdf manifests/
   locks/src (stale comment at Cargo.toml:33 must be deleted first).
7. G7-PATENT — verify_ost_design_around.py 4/4 DISTINCT-MECHANISM +
   RDF-11 auditor sign-off (script + patent/claims.txt absent —
   loud-skip BLOCKED path).
8. G8-STARDF-F1 — stardf F1 census 11 mappings, 10/10 golden digests,
   KD-12/KD-14 differential flips (census scripts absent in purrdf —
   BLOCKED path; work spec in 04-stardf-closure.md).

## 4. Poll / budget laws

- No `Process.sleep`. All waiting is a bounded poll on a real
  observable (file existence, process exit, gate output) with an
  explicit deadline; on deadline expiry record typed BLOCKED, never
  spin.
- Handoff poll: bounded poll (~60s interval, <=45 min total) for
  `HANDOFF.md` + the 00-05 doc set; the loop writes/acts from the
  landed state, and HANDOFF absence is logged, not awaited forever.
- New-crate publish law: ONE publish attempt per crate, sequenced
  post-window (after the overnight loop closes, not concurrently with
  lever lanes); on a registry slide (index reject or timeout) stop and
  back off 1 hour before the next attempt. No retry storms.
- Budget: peak concurrency 7 agents (05-wave-plan.md); compile-freeze
  SLA ~10 min with disclosed minimal unblocks; census-vs-abort rule —
  a compile abort (exit 1, 0 tests executed) is never counted as test
  failures.

## 5. Terminus law

The overnight loop's terminal state is exactly: the 22-crate table
complete (every cell pass / registered-divergence / loudly-MISSING)
PLUS an eight-gate receipt (G1-G8, each with first-hand command+exit).
BLOCKED is a valid terminal cell; silence is not. The loop may not
claim done with any cell unrendered or any gate unwitnessed.

## 6. Prohibitions (restated; each is gate-enforced, not advisory)

- Zero DataFusion: no DataFusion dependency re-enters any workspace
  member. Citation:
  docs/jira/v26.11.1/cgap-datafusion-removal.md.
- Zero runtime string interning: `strings_admitted` counter stays 0;
  any nonzero is a gate failure, not a warning.
- Zero epistemic laundering: every perf/status claim carries a waves/
  file:line receipt or is marked `[TARGET-v26.11.2]`. GB/s is never
  quoted as a multiple of facts/s — throughput units are
  workload-specific and never converted implicitly.
- Design-around law: restating a patented mechanism as a design-around
  = typed REFUSED. Parity is achieved by independent mechanisms, not
  renamed ones.

## 7. Coordinator's post-wave instruments (LANDED)

- `waves/v2-12-stop-list.md` — coordinator action table: rows STOP
  (superseded) vs KEEP (open obligations: P3-13 17-crate sweep, RDF-11
  auditor sign-off, RDF-12, PP-4, PP-7).
- `waves/v2-13-commit-manifest.md` — PLANNED ordered commit plan over
  the 265 dirty cgap paths + 30 stardf paths; coordinator executes
  AFTER wave close; zero git writes by lanes.
- `RESTART-NOW.md` — session-kill recovery: publish census 18/22 is
  SUPERSEDED (see §2-A; census-current 19/22 per v2-07);
  cgap-integration owes ONE invocation (publish-
  ready; stop-on-slide + 1hr backoff per §4); lawful non-publishes =
  court/beam/cli/bench.
- `waves/v2-12-stop-list.md` + RESTART-NOW supersede any in-session
  memory of wave state; read them before re-dispatching any lane.
