# FIX-14 — Deployed-Ops Requirements Register (remediates RED-16 A1/A2/A3/A6)

Lane: FIX-14. Contract: single output file, no git commands. Date: 2026-11-02.
Remediates: `docs/jira/v26.11.2/redteam/red-16-headcount-paradox-audit.md`
findings A1, A2, A3, A6 (BREACH-CONFIRMED) + honest thesis restatement.
Evidence re-verified this session (greps exit 0; CENSUS AS-OF QUALIFIED
2026-11-02 post-RC-05: the original 5-file `crates/cgap-ffi/src/`
listing — lib.rs, oabi_lookup.rs, receipt_chain.rs, sovereign.rs,
xip_tables.rs — PREDATES `stats.rs`, which landed later that day per
`lanes/lane-rc05-stats-block.md`; the tree now carries a runtime stats
surface, so the "zero telemetry modules" reading applies only as of
the original grep, not the current tree). `crates/cgap-storage/src/`
contains `atomic_commit.rs` (recovery chain) and `dred.rs` — no
runtime-ops surface.

## A1 — Runtime telemetry / failure detection

### Requirements (deployed substrate must expose)

- R-A1.1: A `#![no_std]` stats block written by the kernel on a real
  event (commit, lease grant/deny, status-code transition), not on a
  clock. Every counter is a typed observable backed by kernel state.
- R-A1.2: Export via the existing C-ABI seam: one `extern "C"` getter
  returning the stats block by value (fixed-size, CRC'd), plus the
  existing `CgapStatus` register (`sovereign.rs:68`) as the error channel.
  No polling loops inside the kernel.
- R-A1.3: Detection is pull-based: the operator agent polls the C-ABI
  getter with a bounded deadline; no kernel-side sleeps or timers.
- R-A1.4: Anti-vacuity: a court mutates the stats block writer in a /tmp
  sandbox copy (stall one counter) and the detection court must flip
  (counter stalls => poll verdict degrades to typed UNHEALTHY). No
  Process.sleep anywhere; readiness is a state transition on an event.

### Owner recommendation

- cgap-ffi. The stats block is kernel state exported across the ABI
  boundary; `sovereign.rs` already owns `CgapStatus` and the reachability
  law, so counters live next to the register they annotate.

### Status

- PARTIAL (updated 2026-11-02, post-RC-05 merge — was MISSING).
  `crates/cgap-ffi/src/stats.rs` landed per
  `lanes/lane-rc05-stats-block.md` (ALIVE): `CgasStatsBlock` 120 B
  repr(C) with CRC, opt-in `StatsRecorder` + `Record` hooks,
  `health()` typed poll verdict, and the `extern "C"
  cgap_stats_snapshot` pull-based getter (R-A1.2/R-A1.3). Layout,
  counters, health-degrade, reset, and C-ABI getter courts all green
  (5/5, exit 0); anti-vacuity mutation flip witnessed in a /tmp
  sandbox (R-A1.4). rv32imc-clean via a single-writer `Cell64`
  (no atomic RMW on that target). HONEST RESIDUAL: the ring-owner
  integration (calling `stats::global().record_*` at the ring
  admission/refusal/quota/receipt sites, ingress_ring.rs/egress_ring.rs)
  is DOCUMENTED-NOT-WIRED — stats.rs carries the 1-line integration
  recipe; ring files are another lane's ownership and were not touched.
  Detection at runtime is therefore available at the ABI seam but not
  yet driven by kernel events.
## A2 — DR / restore courts (media-loss scenario)

### Requirements

- R-A2.1: A DR court must exercise real media loss against the REAL WAL
  framing (A05): corrupt/truncate the journal at every slot class (valid
  tail, CRC-bad tail, torn write, empty slot) and require the recovery
  verdict chain to classify each and roll back to the immutable base root.
- R-A2.2: Full-site scenario: base root + journal both lost => typed
  DATA_LOSS verdict, fail-closed (no fabricated root). Postmortem evidence
  via receipt_chain replay (cgap-ffi) is required output of the court.
- R-A2.3: Anti-vacuity: named single-fault mutation of the recovery
  validator chain in a /tmp sandbox copy with sha256 byte-exact restore +
  mtime bump; the DR court must flip on each mutation. A court that cannot
  flip is FAIL; skips are loud ([SKIP] <reason>).
- R-A2.4: Bounded-deadline restore budget asserted as a ratio fence against
  a sleep-free baseline, not an absolute ms bound (repo no-sleep doctrine).

### Owner recommendation

- cgap-storage. Recovery already lives here: `atomic_commit.rs` owns the
  journal recovery chain (`recover`, per-slot `DecodeRejection`,
  `RecoveryDegraded` verdict, base-root rollback) and already carries the
  named single-fault mutation protocol pattern (:342-359).

### Status

- PARTIAL -> kernel-side verdict LANDED (errata 2026-11-02, lane F-04;
  re-read from disk at errata time): the typed kernel-side DATA_LOSS arm
  EXISTS — `RecoveryVerdict::{Recovered,DataLoss}` +
  `recover_loss_classified` (`crates/cgap-storage/src/atomic_commit.rs:
  1076-1096`), with `DataLossEvidence` accessors (:1052-1069) — and the
  DR court carries S-04b legs: c6 journal-artifact-lost -> typed
  DataLoss(JournalMissing) and c7 base-root-lost -> typed DATA_LOSS with
  journal evidence preserved (`crates/cgap-storage/tests/
  media_loss_recovery_court.rs:409-460,462+`). Owning receipt:
  `docs/jira/v26.11.2/lanes/lane-rc06-media-loss-dr-court.md` —
  `cargo test -p cgap-storage --test media_loss_recovery_court` -> exit
  0, 6 passed / 0 failed (court run recorded there, session of that
  receipt). STILL MISSING on top: the receipt-replay postmortem wiring
  over cgap-ffi (R-A2.2 tail) and the operator restore runbook; R-A2.4
  ratio-fence restore budget still unmeasured. Original PARTIAL text
  below stands as the as-filed record.
- PARTIAL — existing coverage verified this session by reading
  `crates/cgap-storage/src/atomic_commit.rs`: journal-level recovery
  (clean tail, rollback to base root, typed degraded verdicts) EXISTS and
  is mutation-tested at module level (:262-359). MISSING on top of that:
  a cross-crate DR court over whole-media loss (R-A2.2), an operator
  restore runbook, and the receipt-replay postmortem wiring (R-A2.2 tail).

## A3 — RMA / fleet lifecycle

### Requirements

- R-A3.1: Per-node identity: a device identity record (serial, hardware
  revision, capacity ceiling — RingSink 65,376-lane class) minted at
  provisioning and hash-chained into the receipt chain so RMA swaps are
  receipted transitions, not silent hardware changes.
- R-A3.2: Fleet register (out-of-kernel): a host-side membrane crate
  owning (a) device inventory, (b) RMA state machine
  (provisioned -> active -> suspect -> retired), (c) patch cadence per
  A4, (d) per-node capacity ceilings aggregated to fleet capacity.
  State transitions are events; readiness is a typed transition, never a
  clock expiry.
- R-A3.3: Anti-vacuity: mutating a retire transition (retired node still
  answers ingress) must flip the lifecycle court; receipt chain must show
  the retirement as a hash-chained transition.

### Owner recommendation

- NEW membrane crate (recommend `cgap-fleet-ops`, publish = false).
  Neither cgap-ffi (kernel ABI surface) nor cgap-storage (journal
  semantics) is the right home: fleet inventory/RMA is a host-side
  coordination concern spanning many nodes. cgap-ffi contributes only the
  device-identity record (R-A3.1) minted at provisioning.

### Status

- PARTIAL (updated 2026-11-02, post-RC-13 merge — was MISSING).
  `crates/cgap-fleet-ops/` landed standalone (own `[workspace]`, not a
  root member, `publish = false`) per `lanes/lane-rc13-fleet-ops.md`
  (ALIVE): R-A3.2 fleet register + RMA state machine
  (Commissioned -> Healthy -> Degraded -> Retired) with hash-chained
  transition log, typed `FleetRefusal`s, `ingress_eligible()` (Retired
  never eligible); 4 courts green (exit 0) and 2 sandbox mutation flips
  witnessed (evidence-check removal, threshold `>` -> `>=`) — R-A3.3's
  flip law witnessed for the retire/evidence arm. `slo.rs` also lands
  the A6 host half: pinned status codes, 53-byte checksummed
  `SloSnapshot`, `AlertPredicate`, bounded `poll_until` with typed
  `SourceUnhealthy` stall verdict. HONEST RESIDUALS: R-A3.1 device
  identity is NOT minted in cgap-ffi yet (no provisioning-time record
  in the receipt chain); counter minting in cgap-ffi (A1/A6 kernel
  half) remains that crate's future lane — `SloSource` is a read-only
  seam with no kernel feed wired.


## A6 — On-call / alerting (detection gap)

### Requirements

- R-A6.1: SLO surface: typed SLO counters derived from the A1 stats block
  (commit latency class, lease-deny rate, status-code histogram over
  0x4347_0004 / 0x4347_0006 / 0x5032_38FF), exported through the same
  C-ABI getter — one mechanism, not two.
- R-A6.2: Alerting is host-side: a membrane process polls the C-ABI
  getter with bounded deadlines and evaluates typed alert predicates
  (threshold crossings on the SLO counters). Paging/escalation is
  explicitly OUT of kernel scope; the kernel exposes state, the host
  decides.
- R-A6.3: Postmortem: every alert links to the hash-chained receipt
  slice covering the alert window (cgap-ffi receipt_chain replay), so
  forensics is replay, not memory.
- R-A6.4: Anti-vacuity: mutating an alert predicate (threshold off-by-one
  or inverted) in a /tmp sandbox must flip the alert court (real counters
  -> missed/extra alert). No time-based flake surface.

### Owner recommendation

- cgap-ffi for the SLO counter export (same rationale as A1: counters
  annotate kernel state behind the existing ABI seam and the CgapStatus
  register). Host-side poller/alert evaluator belongs in the same new
  membrane crate as A3 (`cgap-fleet-ops`) — one host process, one poll
  loop, both concerns.

### Status

- PARTIAL (forensics + host poller). Existing coverage cited after
  reading: cgap-ffi `receipt_chain.rs` + hash-chained receipt replay
  (P1/P2/P4 per
  `docs/jira/v26.11.2/audit/cgapprd-prd-v2.md:74,111,120`) give real
  postmortem evidence, and `CgapStatus` (sovereign.rs:68) is a typed
  error channel. Post-RC-13, the host-side half of R-A6.2/R-A6.4 landed
  in `crates/cgap-fleet-ops/src/slo.rs` (`AlertPredicate`,
  `poll_until` bounded poll, corruption-rejecting `SloSnapshot`;
  predicate mutation flip witnessed — see
  `lanes/lane-rc13-fleet-ops.md`). STILL MISSING: SLO counter minting
  in cgap-ffi (kernel half, depends on A1 ring integration landing),
  the `SloSource` -> kernel feed wiring, and any paging/escalation
  surface.

## Honest headcount-thesis restatement (RED-16 finding 2, kept)

RED-16 found the "zero-headcount / SRE-excision" term UNSUPPORTED (zero
on-disk referents). The claims that DO exist (R1: HYPERSCALER
_REPLICATION_TIMELINE_REPORT.md:259-268; R2: 09_brooks_law_and
_coordination_drag.md:51-80, scoped away from headcount at :78-80) support
only this: agent-lane coordination reduces ENGINEERING coordination
headcount (O(n) hub-and-spoke vs O(n^2) pairwise). It survives RED-16.
What also survives: the deployed-ops floor RISES. Build-time gates and
receipt replay do nothing for runtime detection (A1), site DR (A2),
RMA/lifecycle (A3), or alerting (A6) — and the operator skill floor for a
no_std self-verifying appliance (reading hash-chained receipts, gate0
replay, CGAP status codes) is rarer expertise, not cheaper. This register
is the remediation floor the deployment thesis must fund before any
ops-headcount claim is made.

## Standing

- File written: docs/jira/v26.11.2/redteam/fix-14-ops-requirements.md (this file).
- Commands this session: grep sweeps over crates/cgap-ffi/src/ and
  crates/cgap-storage/src/ (exit 0, results cited above); Read of
  red-16 audit (exit 0); Read of atomic_commit.rs recovery sections
  (exit 0).
- No git commands. Shared tree untouched except this file.
- Standing: PARTIAL — requirements registered; A1/A3/A6 PARTIAL as of
  the 2026-11-02 RC-05/RC-13 merges (see per-section Status),
  A2 PARTIAL as originally verified by reading, not recalled.
  Original A1/A3 MISSING verdicts and the 5-file ffi census stand as
  of the original session only; errata here, not a rewrite of the
  grep evidence itself.
