# F2-15 Bench Flips — Verdict Marker

GAP-6 marker file closing the F2-15 schema-drift finding: the F2-15
court-auditor's 4/4 bench-flip verdict previously existed only as the
audit lane's final message — no marker file (witnessed at terminus by
`f3-20-terminus.md`, "No `f2-15` marker file exists"). This file lands
the verdict as on-disk evidence. Source: F2-15 audit final message,
relayed verbatim by the GAP-6 dispatch, 2026-11-01. GAP-6 did not re-run
the flips; it transcribes, names the source, and backfills the missing
subject sha256s in `bench-bn3-receipt.md` / `bench-seal-receipt.md`
(sections "Identity (GAP-6 backfill)").

## Verdict table (as transcribed from the F2-15 audit)

| # | Lane | Subject | Mutation | Verdict (audit's numbers) |
|---|---|---|---|---|
| 1 | F2-11 ingress | `crates/bench/benches/ingress.rs` | M-F2-11-1 preflight-manifest court poison | FLIPPED — 21x |
| 2 | F2-12 parquet | `crates/bench/benches/parquet.rs` | oracle-corruption sandbox flip (retained-alloc channel semantics note) | FLIPPED — retained-alloc channel semantics note recorded with the row |
| 3 | F2-13 bn3 | `crates/bench/benches/bn3.rs` | binding-alias comparator substitution (SHIM-DOE4 class) | FLIPPED — alias wrapper 118.9 -> 232.2 ns/pair, 1.95x |
| 4 | F2-14 seal | `crates/bench/benches/seal.rs` | forced BLAKE3 recompute (digest-byte flip class) | FLIPPED — +12% with controls <1% |

**Standing: FLIPPED-4/4, zero VACUOUS.** Both /tmp sandbox restores
(bn3, seal) were sha-exact per the audit's record.

## Corroborating landed evidence (receipts, not re-run by GAP-6)

- Ingress: `bench-ingress-receipt.md` records M-F2-11-1 FLIPPED the
  preflight court, command 3, exit 101.
- Parquet: `bench-parquet-receipt.md` court 2 records the /tmp
  oracle-corruption flip, panic at benches/parquet.rs:112, exit 101,
  shared tree sha-exact before and after.
- Bn3 / seal: flip execution is F2-15's lease; the receipts name the
  mutation classes and explicitly defer execution to the auditor lane.

## Boundaries of this marker

- The four numbers above are the F2-15 audit's recorded verdicts as
  dispatched; GAP-6 holds no re-measurement. Restores were verified
  sha-exact by the audit, not by this lane.
- Receipt rows 1-2 carry GAP-6 transcriptions of already-landed receipt
  text; rows 3-4 carry the audit-only verdicts now given a marker file
  and subject identity.
