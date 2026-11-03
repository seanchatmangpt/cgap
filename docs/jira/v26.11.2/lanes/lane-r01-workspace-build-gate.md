# Lane R-01 — workspace build gate (tollgate, all wave lanes landed)

Date: 2026-11-02. Repo: /Users/sac/cgap (canonical checkout, branch main).

## W1 Subject

`cargo build --workspace` over the post-wave tree; suspects per dispatch:
`crates/cgap-ffi/src/stats.rs` (RC-05), `crates/cgap-bn3/src/noalloc.rs`
(RC-03), `crates/cgap-ffi/src/sovereign.rs` (RC-08 transient).

## W2 Command + exit (this session)

- `python3 scripts/run_checked.py --out /tmp/r01-build.log -- cargo build --workspace`
  -> **exit 0** (run_checked echoed `exit=0`), "Finished `dev` profile ...
  in 1.74s". Log retained at /tmp/r01-build.log.

## W3 Errors / repairs

- ZERO errors. No repair applied; no suspect file touched; no unblock
  disclosure (nothing blocked). Diffs: none by this lane.

## W4 Warnings (informational, pre-existing wave residue, NOT repaired —
   outside ownership)

- `cgap-git/src/mcp_driver.rs:36` — `#![no_std]` non-crate-root (unused_attributes).
- `cgap-git/src/ref_store.rs:214` — deprecated `fetch_update` (renamed `try_update`).
- `cgap-git/src/ref_store.rs:292` — unused `bound`.
- `cgap-git/src/pkt_line.rs` — 7 dead_code warnings.
- `bench/Cargo.toml` — 3 unused deps (cgap-ffi/cgap-git/cgap-storage).

## W5 Fences audited

- Root workspace green (CLAUDE.md gate) — held, exit 0 this session.
- Suspect-lane receipts re-read first (RC-05/RC-03/RC-01): RC-05 used
  Cell64 (no AtomicU64 on rv32imc) and reports its own rv32imc court pass;
  RC-03 noalloc.rs is append-only per its receipt; RC-01 reentrancy court
  is test-only. Build corroborates none of the three broke the shared compile.

## W6 Standing

- **ALIVE** — one command + exit 0 this session, exact subject = current tree.

## W7 Divergences

- Dispatch anticipated repairs; none needed (build already green at lane
  start). Suspect-transient hypothesis (sovereign.rs CoerceUnsized,
  noalloc.rs duplicate-field residue) is moot: zero residual errors.
- Warnings listed in W4 are outside this lane's ownership; left in place.
- No git commands run.

JSON terminus: {"lane":"R-01","files_written":["docs/jira/v26.11.2/lanes/lane-r01-workspace-build-gate.md"],"commands":["python3 scripts/run_checked.py --out /tmp/r01-build.log -- cargo build --workspace -> exit 0"],"courts_run":[],"standing":"ALIVE","divergences":["no repairs required; build green pre-lane"]}
