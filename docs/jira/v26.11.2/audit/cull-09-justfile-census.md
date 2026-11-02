# cull-09-justfile-census

CULL-09 justfile census (2026-11-02, read-only). Subject: justfile
(94 lines, 13 recipes). Fresh probes: just --list exit 0;
just --evaluate exit 0 (sole var CENSUS_ARGS, correctly prefixed).

## Stale-claim correction

Header (:8-9) + comments call dod and status "fail-closed
placeholders". BOTH HAVE LANDED: scripts/dod.sh exists (real
branch: 0 no-red / 1 red / 2 usage / 3 fail-closed);
crates/cli/src/bin/status.rs exists (dashboard, exit 0 always
by design — it reports, it does not gate). Neither is dead.
Only genuinely dead path: dod's else-branch (exit 1 when
dod.sh missing — now unreachable). Proposal for lane F: delete
the placeholder branch, update the stale header lines.

## Overlap analysis

- doctor vs doctor-agents vs doctor-merged: DXD-02 decided
  extension-not-composition; DXD-10 lane F appending
  doctor-merged calling the SAME script as doctor-agents =
  two names one binary = DXD-07 Anti-pattern-1 in slow motion.
  Proposal: doctor-merged becomes canonical; doctor-agents
  retained one wave as alias, then removed (coordinator edit).
- census vs direct callers: sole wiring is the passthrough;
  direct callers are lane receipts, not live code. KEEP both.
- receipts vs status: status computes the same receipt tally
  plus CTQ/gate/dirty. Low-priority fold proposal (receipts =
  cheap no-build form; status cargo-runs ~7s).

## Per-recipe verdicts

default/doctor/verify/bench/courts/branchless/receipts/dod/
status/leases/census: KEEP. doctor-agents: MERGE into
doctor-merged after lane A lands. Proposed: interop (KEEP;
DXD-01 one build+vet+test recipe at EOF), clean-interop (KEEP;
DXD-08). Rejected: interop-build/vet/test/bench (stages of
interop or folded), interop-seal (low loop value).

## Minimal set after the DXD-10 wave

13 -> 15 (interop, clean-interop) -> 14 after the
doctor-agents consolidation. Zero REMOVEs justified: dod/status
alive (stale comments only); every other recipe wires a
distinct landed script/bin. The cull: 1 duplicate comment
block, 1 dead placeholder branch, 1 eventual recipe alias,
1 optional fold.

## See Also

audit/dxd-01-justfile-census.md · audit/dxd-07-justfile-collisions.md ·
audit/dxd-10-dx-wave-plan.md
