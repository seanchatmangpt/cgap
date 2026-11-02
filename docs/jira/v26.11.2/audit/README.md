# audit

Index for the v26.11.2 audit directory. Seeded by lane SEED-A
(2026-11-02) from session receipts already on disk; nothing re-derived.

## Deliverables

- `audit-ledger.md` — one row per landed receipt: lane id, standing
  (as the receipt claims), load-bearing command+exit fact, OPEN item.
  Absent receipts are PENDING, never inferred.
- `dod-*.md` — the EXP-01..10 DoD lanes (dispatched, pending) will each
  land a per-crate matrix here named `dod-<crate-or-surface>.md`.
  Until those files exist, the EXP lanes are PENDING, not PASSING.

## Release-checklist matrix schema (unified, for all dod-*.md)

Every dod file carries one row per crate with these columns:

1. `crate` — workspace member name.
2. `publish` — publish flag: yes / no (bench, beam, court are
   `publish = false`; see `../../01-release-gates.md:28-32`).
3. `no_std verified` — ALIVE with command+exit, or UNVERIFIED.
4. `zero-alloc court` — court name + exit, or ABSENT.
5. `package dry-run verdict` — exit code of
   `cargo package --allow-dirty --no-verify` (or packaged run), with date.
6. `blocking gaps` — free text; empty only if the row is fully green.

A cell with no receipt is UNVERIFIED; a skipped check is loud
(`[SKIP] <reason>`), never a blank read as a pass.

## Publish status (frozen)

Operator freeze, 2026-11-02: `../waves/publish-halt.md`
(standing FROZEN-BY-OPERATOR). 8 crates live at 26.11.2 before the halt
(immutable); the frozen remaining set and the addendum readiness matrix
(7 READY dry-run exit 0; cgap-ffi and wasm-footprint BLOCKED; three
26.11.3 live-repairs owed on lift) live in that file. No publish
attempt is authorized until the operator lifts the freeze.

## Session-level known blockers

Mirrored from `audit-ledger.md` (see that file for receipt citations):

1. G2-PARETO rollup OPEN (G08 receipt; HANDOFF.md:128).
2. B05 cscb C6 honest throughput FAIL: 248M vs 800M facts/s target.
3. C06 scripts 90% coverage floor HONEST RED: exit 1, four red scripts.
4. C05: cgap-parquet DEFAULT-features build exits 101 (18 compile errors).
5. B08 zero-tax anti-vacuity court cannot flip (receipt claims FAIL).
6. Publish freeze + cgap-ffi BLOCKED on parquet 26.11.3 re-publish;
   wasm-footprint host-link defect undiagnosed.
7. HANDOFF gate ledger rows G1-PUBLISH, G3-MAT-BENCH, G4-DRED,
   G8-STARDF-F1 BLOCKED at receipt time (G7-PATENT since flipped by
   G18/w6-g7-patent-gate-flip).
8. G20: verify_disassembly_invariants.py standalone exit 2
   (emit-asm build 101) — open, attributed.
9. AE4 full-court arm E loud [SKIP]: binary-mutant arm found no
   patchable candidate in the witness archive (environmental).

## See Also

- `audit-ledger.md` — the receipt-by-receipt ledger this index summarizes.
- `../waves/publish-halt.md` — frozen publish state + readiness matrix.
- `../HANDOFF.md` — gate ledger and cross-session handoff.
- `../DEFECT-REGISTER.md` — G08's defect rollup.
