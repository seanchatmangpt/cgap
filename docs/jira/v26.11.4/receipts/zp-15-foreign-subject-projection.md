# ZP-15 — Foreign-Subject Projection Receipt (lane S-4)
Standing: PASS

Date: 2026-11-05. Lane: L4 (S-4, backlog item S-4 / next-wave-seeds.md B row).
Falsifier under test (seed row): "projection court renders P1-P8 rows with
`projection_class` on a real subject" — DISCHARGED (court exit 0, 5/5 green,
P1-P8 rendered this session, output quoted below).

## Subject and identity

- Court: `/Users/sac/cgap/crates/cbmqs-cli/tests/zp15_foreign_subject_projection_court.rs`
  (sha256 `b7a98cd9b98fa8f7b323887db9cb8d14cc1c8894b07c9453d1a34ed3fc323628`),
  already landed at cgap HEAD `2aba0309` via `b3fae1f9`.
- Kernel: `/Users/sac/cgap/crates/cgap-aoq/src/rfc0089_certificate.rs`
  (sha256 `7d64b9684b1594b828bc57c70f4f293e4e7c3ca7a829aeb39bdd1a4c368fae31`).
- Real subject receipt: `/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md`
  (sha256 `b22af70343e511253398ce3cd18c812df00529681c3cc208855497f6fdc01390`).

## Executed gate (this session, 2026-11-05)

```
cd /Users/sac/cgap
cargo test -p cbmqs-cli --test zp15_foreign_subject_projection_court -- --nocapture
# exit 0 — test result: ok. 5 passed; 0 failed; 0 ignored
```

Rendered register (court stdout, abridged — full rows are deterministic from
the subjects above):

```
== ZP-15 foreign-subject projection register ==
P1   <- runner-G1 ... ANALOGUE         EXTERNAL   NO_CENSUS law -> ContractUnsupported -> cap PROVISIONAL
P2   <- RFC§6-G1  ... PROJECTABLE      ATTESTED   0 allocs / 0 B per op (AB-9)
P3   <- RFC§6-G1b ... PROJECTABLE      ATTESTED   12.2M/s slab push+reset (AB-19)
P4   <- RFC§6-G2  ... PROJECTABLE      ATTESTED   86 ns amortized-min encode+decode (AB-6)
P5   <- RFC§6-G3+G4 ... PROJECTABLE    SELF       drift court flips on injected mutation (in-repo)
P6   <- RFC§6-G5  ... PROJECTABLE      ATTESTED   50k-frame storm; FrameTooLarge typed (AB-10)
P7   <- RFC§6-G6  ... ANALOGUE         EXTERNAL   200 churn cycles (lsp-max cross-repo cited)
P8   <- RFC§6-G7  ... NOT-PROJECTABLE  SELF       1276.01 ms / 50,000 links = 25.5202 µs/link vs 25 µs target (AB-14)
```

## Court legs (all five green)

1. `court_projection_renders_p1_p8_on_real_subject` — 8 rows, each carrying
   `projection_class ∈ {PROJECTABLE, ANALOGUE, NOT-PROJECTABLE}` +
   `attestation_class ∈ {SELF, ATTESTED, EXTERNAL}`; P8 correctly
   NOT-PROJECTABLE on the ≤25 µs/link kernel-internal gate (plan law).
2. `court_oracle_flips_on_mutated_copy` — independent oracle parse of the
   AB-14 receipt; flipping a measured byte flips the G7 verdict (anti-tautology).
3. `court_tampered_register_refuses_typed` — BLAKE3 seal over the register;
   tampered register refuses typed (`SealMismatch`).
4. `court_real_subject_seals_and_g7_follows_disk` — certificate issued AND
   verified over the real on-disk AB-14 subject.
5. `court_missing_source_refuses_fail_closed` — missing source path refuses
   fail-closed.

## Plan-law conformance

Matches ggen `docs/jira/v26.11.4/PLANS.md` ZP-15: namespaced P1-P8 rows
(avoiding the runner G1-G7 / RFC §6 G1-G7 collision), NOT-PROJECTABLE used
for kernel-internal gates (P8 receipt-chain), NO_CENSUS →
CONTRACT_UNSUPPORTED → PROVISIONAL honest cap for foreign JVM incumbents (P1).

## Replay

```
cd /Users/sac/cgap && cargo test -p cbmqs-cli \
  --test zp15_foreign_subject_projection_court -- --nocapture
```

## Register effect

Closes the S-4 seed row's UNVERIFIED claim (no receipt existed in
ggen/cgap/arcus trees for ZP-15; this file is the missing receipt, in the
subject repo). Closes backlog item S-4 in `~/.claude/dfcm/backlog.json`
evidence terms: falsifier discharged with command + exit receipt above.
No commits made; file landed in the working tree per lane contract.
