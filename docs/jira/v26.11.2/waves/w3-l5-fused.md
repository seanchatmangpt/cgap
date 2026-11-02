# W3-L5 — fused ASC450-into-P28 row fold

Lane W3-L5, wave v26.11.2, 2026-11-01. Lever spec:
`docs/jira/v26.11.2/plans/levers-L1-L5.md` §L5; court corpus:
`docs/jira/v26.11.2/07-court-corpus.md` §2 L5-COURT; engineering:
`docs/jira/v26.11.2/02-pareto-levers.md` §L5.

## Files written (sole writes)

- `crates/cgap-p28/src/asc450.rs` — `FusedOutcome` + `pub fn fused_row`
  (free fn, signature pinned by the dispatch: meter, rec, amount, crossing,
  tick). fused_row computes the row's 80 B digest and feeds it through the
  fail-closed §5↔§4 anchor match (recover's ceiling/digest check class,
  recover at asc450.rs:735-739); crossing rows route to record_unlicensed
  (§4.2 boundary-exact), licensed rows to record_licensed; the sealed meter
  state is folded into the same call (§4.4-3). No frozen pin touched: 80 B
  layout, LICENSE_MAGIC, flag bits, attestation offset tripwires
  (attestation.rs:752/:781 tests ran green inside the 37-test lib suite),
  SAFE_HARBOR_CEILING_MAX_CENTS semantics unchanged.
- `crates/bench/benches/chains-seal.rs` — added
  `governance_chain_fused_per_row` (consts [1024, 65536]); the unfused
  `governance_chain_per_row` kept verbatim as the PARETO control. Fused arm:
  per-row meter genesis, one hoisted `current_tick()` per row (§4.3), one
  `fused_row` call, chatman seal/verify with from_root chaining folded in
  the same loop body; final-state asserts incl. `proven == N/2`,
  `sealed_states == N` (a sealed state on EVERY fused row).
- `docs/jira/v26.11.2/02-pareto-levers.md` — L5 section + target-summary
  row updated with measured numbers.

## Commands (exit witnessed this session)

- `CARGO_TARGET_DIR=/tmp/w3-l5-target cargo test -p cgap-p28` → exit 0;
  37 lib tests incl. new `w3l5_fused_row_byte_equality_and_anchor_court`;
  `tests/lever_courts.rs` 2 passed (W2-13 pre-lever controls, still green
  over the landed kernel change).
- `CARGO_TARGET_DIR=/tmp/w3-l5-target cargo bench -p bench --bench chains-seal`
  → exit 0. Measured (divan, 100 samples, timer precision 41 ns):
  - unfused control `governance_chain_per_row`: 2.42 ms / 2.42 µs·row⁻¹
    @1024; 155.5 ms / 2.37 µs·row⁻¹ @65536 (matches the P3-19 baseline —
    control reproduces the known number).
  - fused `governance_chain_fused_per_row`: 3.12 ms / 3.05 µs·row⁻¹
    @1024; 199.1 ms / 3.04 µs·row⁻¹ @65536.
- Flip court (sandbox `/tmp/w3l5-sb`, target
  `/tmp/w3l5-sb-target`, both deleted at lane end):
  - Leg A baseline (sandbox copy, trace-only patch printing prev_root +
    folded checksum): exit 0; iter-1 root f2d37fb7… checksum
    0x5898fa94159e98b2.
  - Leg B mutation `W3L5-DEMOTE` (row 1 amount demoted to ≤ ceiling,
    crossing flag kept true): exit 101, `proven == N/2` fires (511 vs
    512); iter-1 root 8bac1258… and checksum 0x9a765cb070b23b1b — BOTH
    diverge from baseline (the demoted row's admissions anchor falls back
    to the attestation digest instead of the proof digest). FLIPPED.
  - Leg C restore: pristine copy restored sha256-exact
    (19cdf4086b673469fedecb357cc58e4d951a3ac71fd4f0629b0ebef97339e1af),
    mtime bumped, re-run exit 0, 199.4 ms @65536 — the mtime-bump restore
    law honored (stale-fingerprint VACUOUS re-run avoided).
- Sandbox build roots deleted at lane end (lease law).

## Standing: PARTIAL — lever gate NOT met

Measured regression, recorded honestly:

- Fused arm 3.04 µs/row vs unfused control 2.37 µs/row @65536 (same host,
  same session, same corpus builders). Target <1.45 µs/row NOT met; the
  fuse as specified ADDS the §4.4-3 sealed-state mint (2 keyed BLAKE3 + 1
  tick) on every licensed row that the unfused path never paid.
- The "one meter lineage per batch via recover" mechanism is refused by
  the kernel's own laws for this corpus: §4.2's one-shot latch + monotone
  seq make N/2 emissions from one lineage unrepresentable (max 1), and
  recover's §4.4-3 digest match refuses a lineage whose anchor changes
  per-row (bench records differ per row). proven == N/2 therefore forces
  per-row entity meters in the fused arm.
- What the fuse CAN legitimately save (per-row genesis validate_intrinsic
  + 80 B BLAKE3, per-row Lease::new) is ~O(100 ns) — far below the 0.92
  µs/row the target demands. The 02-pareto-levers bottleneck model
  ("redundant span walk") does not survive contact with the measured
  PARETO: the meter stage's cost IS the emission crypto, not traversal.
- Standing: the fused_row kernel seam + courts are ALIVE (ran+passed this
  session); the lever's perf gate is FAILED-BY-MEASUREMENT, not skipped.
  Recommendation recorded in 02-pareto-levers.md: re-PARETO (the
  irreducible cost is keyed BLAKE3 roundtrips; a real cut requires fewer
  crypto ops per row, e.g. amortizing seals or reusing the proof seal as
  the state seal — a frozen-layout question, not a lane-local one).

## Divergences (disclosed)

1. One meter lineage per batch via recover is not realizable with
   proven == N divergence-free asserts on the mixed corpus (latch law);
   realized instead as per-row entity meters + the anchor match inside
   fused_row (the :735-739 analog), with the recover chain law exercised
   in the unit court (terminal fused sealed state recovers; foreign
   anchor refuses SealedStateInvalid).
2. Fused arm seals a state on EVERY row (`sealed_states == N` assert,
   differs from the unfused `== N/2`); this is what FusedOutcome.sealed
   being non-Option demands, and it is the measured source of the
   regression.
3. Court corpus doc's kill signal "proven == N/2 assert flips" realized
   verbatim; prev_root + checksum divergence witnessed via a
   sandbox-only trace patch (shared tree bench file unchanged by the
   court; restored sha256-exact 19cdf408…).
4. Two transient compile freezes from sibling lanes (stardf-datalog
   Stats Copy derive, cgap-bn3 unify.rs delimiter) were observed and
   self-resolved by their owners within the SLA; no unblock applied.
5. One mid-write corruption of the bench file was repaired on-disk
   before any build (grep-verified); final on-disk bytes are the intended
   content (sha256 19cdf408…).
