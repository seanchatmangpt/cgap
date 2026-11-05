# PL-X2 — Supervisor Failure-Domain Receipt (fd/orphan + BUSY drain)
Standing: ALIVE

Date: 2026-11-04. Lane: CODE-LANE-5 (docs+receipts). Status: ALIVE —
supervisor courts re-run this session, exit pasted; busy-drain court re-run
this session (see flake note).

## Subject

`/Users/sac/arcus/internal/arcus/flight/supervisor.go` — the sidecar
supervisor's failure domain, plus the cgap sidecar's multiplex-refusal drain
(`/Users/sac/cgap/crates/cgap-ffi/src/bin/aoq_sidecar.rs:570-592`).

## Premise correction: stdin-close, not fd-witnessable

The stop law's honest statement: closing the child's STDIN is the only lawful
stop path (`supervisor.go:18-21` STOP LAW; `:505-528` "THE STOP LAW: stdin
close. Nothing else.", no signal escalation; timeout is typed
`ErrSupervisorStopTimeout` "generation %d not reaped within %s after stdin
close" `:519`). "stdin closed" is not directly fd-witnessable from the parent
in a portable way — the WITNESSED flip is the bounded reap + the cleanup:
`os.RemoveAll(dir)` after every exit path (`:274,:280,:372,:393,:465`) so
kill/respawn churn leaks neither an fd per generation nor a /tmp dir per
generation (`:144` comment law, `:452-465` fault-path close+RemoveAll). The
fd court below is the witnessed flip, not an fd-count of the stdin pipe.

## fd/orphan court (the witnessed flip)

`supervisor_test.go` `TestSupervisorChurnFDCourt` — 8 spawn/kill/respawn
cycles, per-cycle fd delta asserted 0, and the last socket dir checked gone:

```
supervisor_test.go:328: cycle 1..8: fd delta 0
supervisor_test.go:353: FD-LEAK receipt: 8 cycles, total fd delta 0
(base 7), last dir /tmp/arcus-sidecar-3912335724
```

Boot-churn stays terminal-and-sticky (`TestSupervisorBootChurnStickyFault`:
sticky fault at generation 3, 3 fault lines, re-start refused — C3 receipt),
clean stop reaps exit 0 with no kill (`TestSupervisorCleanStopNoKill` — C2),
mid-stream kill respawns with socket moved and a fresh boundary (C1 receipt
`gen 1 -> 2, socket moved, fresh boundary installed`).

## BUSY drain + ConnectionAborted

`aoq_sidecar.rs:570-592`: when the single-tenant serve loop is busy, the
listener flips non-blocking and the multiplex-refusal drain accepts-and-
refuses every queued peer with the frozen-band BUSY word (0x4347A003),
`WouldBlock = backlog drained; re-block`; the drain treats
`ConnectionAborted` as continue-the-drain (`:592`) — a peer dying mid-drain
no longer breaks the serve loop. Drain != poison: the sidecar keeps serving.

Court: `/Users/sac/cgap/crates/cgap-ffi/tests/aoq_sidecar_busy_drain_court.rs`
— `busy_drain_refuses_backlog_peer_with_busy_word` (peer B queued behind a
busy A must receive the 4-byte BUSY word bounded, then see EOF; sidecar still
serving) + anti-vacuity `busy_drain_mutation_flips` (SHIM-COURT: /tmp rsync
sandbox, drain span textually removed with anchor+content assertions against
vacuity, mutant sidecar must NOT deliver BUSY to B, sha256 before/after on
the shared source).

## Re-runs (this session, 2026-11-04)

```
cd /Users/sac/arcus && go test ./internal/arcus/flight -run TestSupervisor \
  -count=1 -v
-> PASS (5 tests: C1/C2/C3 churn courts + ChurnChildReallyFailsToBind
   + ChurnFDCourt), ok ... 11.150s, exit 0

cargo test -p cgap-ffi --test aoq_sidecar_busy_drain_court
-> run 1: 1 passed / 1 FAILED (busy_drain_mutation_flips) — transient:
   the mutation leg passes deterministically in isolation and in two
   subsequent full-file runs (2 passed; 0 failed, 7.63s). Failure class:
   sandbox-build/poll timing flake, not a law regression — the leg's own
   anchor assertions all passed and the sha256 restore proof held.
```

## Standing

ALIVE — supervisor courts exit 0 this session. Busy-drain court ALIVE on
re-run with one recorded transient flake on the mutation leg (honest ledger,
not rounded to a clean sweep).
