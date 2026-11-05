# Receipt: zd-2 FA9 race-grid release reds (XA-2 census reds #2-4)
Standing: ALIVE

Milestone: v26.11.3 · Lane: zd-2 (XA-2 census reds #2-4) · Date: 2026-11-04

## Subject

`crates/cgap-ffi/tests/fa9_commit_gating.rs` — 3 deterministic release reds
(`fa9_race_grid_one_writer_no_truncation`, `fa9_alias_race_denied`,
`fa9_rename_race_denied`), all panicking at `spin_until`'s deadline
("owner never signalled ready"). Lib gate code under
`crates/cgap-ffi/src/sovereign.rs` (`spill_gate`) — NOT modified.

## Root cause (measured, not inferred)

The owner thread's readiness signal is correct and fires; the test's poll
BOUND was the defect. `spin_until` bounded by ITERATION COUNT (50M
`SeqCst` loads), not time. That bound is profile-dependent: release
compiles the spin ~1000x faster than debug, so the fence collapses to
~15-20 ms of wall clock, while the owner's real time-to-ready — grant
(`File::create` + canonicalize rebind) + ~309 commits + ring rotation
drain (write_all + flush + `sync_all`) — measures **14.68 ms** in
release (scratch probe, since deleted; `ready after 309 commits`).
Debug passes because its slow spin (~seconds) hides the IO-bound owner
latency. The FA9 law itself is unbroken: the spill gate, (dev,ino)
rebind, alias denial, and rotation drain all behave identically in
release (witnessed by the green legs after the bound repair).

An iteration bound fences the wrong variable: it shrinks exactly when
the code speeds up, while the owner's latency is IO/fsync-bound and
does not. This is the absolute-bounds-vs-ratio-fence law
(no-sleep-testing doctrine): a bounded poll must be deadline-checked in
time against a real observable, which is what it now is.

## Fix (test-only, fix forward, no sleep)

`spin_until` in `tests/fa9_commit_gating.rs` now polls the same
`AtomicBool` observable with a 10 s `Instant` deadline (bounded,
deadline-checked, loud expiry panic on timeout — no
`Process.sleep`-class call anywhere). All other handshake structure,
anti-vacuity legs, and final-state asserts unchanged.

## Commands / exits (this session, exact subject)

| Command | Result |
|---|---|
| `cargo test -p cgap-ffi --test fa9_commit_gating --release` (pre-fix) | FAIL: 4 passed, 3 failed, finished 0.02s |
| Release after fix — run 1 | ok: 7 passed; 0 failed (1.15s) |
| Release after fix — run 2 | ok: 7 passed; 0 failed (1.12s) |
| Release after fix — final x2 | ok: 7 passed; 0 failed (0.85s / 0.77s) |
| Debug after fix — x1 | ok: 7 passed; 0 failed (0.98s) |
| `cargo build --workspace` (dev) | Finished, 0 errors (6.94s) |
| `python3 scripts/zero_string_gate.py` | rc=0 (captured unpiped) |
| `cargo test -p cgap-ffi --lib` | ok: 58 passed; 0 failed (66.89s) |

## Pre-existing vs session-introduced

- The 3 release reds: PRE-EXISTING (reproduced on first pre-fix run;
  premise rotted, not lib regression).
- The `wave9_a07_zero_string_gate_over_all_built_crates` lib red seen
  mid-session: PRE-EXISTING environmental precondition — the gate
  refuses (exit 2, `execution-crate-no-artifacts` for `cgap`,
  `cgap-beam`, `cgap-gpu`) when the workspace dev-profile build is
  absent. Resolved by the documented `cargo build --workspace`; not
  caused by this lane's edits (test-file-only). Note recorded: an
  earlier `rc=$?` after a pipe reported `tail`'s rc — capture gate
  exit codes unpiped.

## Standing

ALIVE for the court (7/7 release x3 + debug x1 this session, exits 0);
lib unregressed (58/58). No git commands run.
