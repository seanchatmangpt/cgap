# Decision D1: wire-shm frozen-datum (CONFIRMED)
Standing: PARTIAL_ALIVE

## Decision

D1 CONFIRMED: `crates/wire-shm` is ORPHANED-DATUM-FROZEN. The PL8 ruling is
adopted as the standing law for the v26.11.4 cycle: no production-data-path
consumer exists, therefore the crate's byte format is frozen datum and the
delete-first escalation applies only to a named two-condition trigger.

## Consumers (frozen because they consume, not because they produce)

- `cgap-mcp` — runtime path dependency (bridge surface).
- `cgap-aoq` — dev-dependency (AB-11 court surface).
- arcus p02 / p09 — name wire-shm bytes as compat datum (UNBUILT).

## Escalation ladder (adopted from CLAUDE.md:84-93)

- Parked raise = cached `next_ordinal` + direct-slice write; the cgap-mcp
  `wire.rs` byte-identity court is the drift fence.
- Sanctioned bounded change = 3-line payload-offset shift iff arcus p09
  selects the 64B header.
- Delete-first applies only if cgap-mcp retires AND no arcus build lands in
  one release cycle.

## Evidence

- `/Users/sac/cgap/CLAUDE.md:84-93` — consumer inventory + escalation ladder.
- `/Users/sac/cgap/crates/wire-shm/src/lib.rs:1-10` — format law in source.

## Falsifier ladder

1. A production-data-path consumer lands (cgap-mcp runtime bridge promoted to
   a production data path) — frozen-datum standing no longer holds.
2. cgap-mcp retires AND no arcus build lands in one release cycle —
   delete-first becomes the lawful next transition.
3. arcus p09 selects the 64B header — bounded 3-line shift is sanctioned.

## Standing

ALIVE as a decision receipt; the frozen-datum law itself is PARTIAL_ALIVE
until arcus p02/p09 build or a consumer transition fires a falsifier rung.

Date: 2026-11-04
