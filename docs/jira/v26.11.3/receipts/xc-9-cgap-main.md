# XC-9 Receipt: cgap main CI triage (courts + verify red runs)

Lane: XC-9 (implementation). Date: 2026-11-04. Repo: /Users/sac/cgap.
Constraint honored: no git commands run.

## Triage table

| # | Failure | Root cause | Class |
|---|---------|-----------|-------|
| 1 | `cli-verify` gate FAIL exit=101: `error: package(s) 'cgap-cli' not found in workspace /Users/sac/cgap` | `scripts/verify_v26.10.31.sh:136` ran `cargo run -q -p cgap-cli` from the repo root; `crates/cli` is a standalone ggen crate with its own `[workspace]`, deliberately NOT a root member (root `Cargo.toml:19-20`), so `-p cgap-cli` can never resolve from the root. | REAL, FIXED |
| 2 | `test-cgap-git` FAIL: `lineage_workflow` "could not execute process ... (never executed) No such file or directory" | Concurrent cargo invocations on the shared `target/` clobbered the test executable between build and run (witnessed 6+ concurrent cargo runs across repos on this host during triage, incl. two other lanes' cgap runs). | CONTENTION FLAKE — passes isolated (1 passed / 0 failed) |
| 3 | `test-cgap-beam` FAIL: `beam_kill_stress_100_iterations` BEAM `{init,do_boot,3,[]}` init crash, finished 2.24s | Load-induced BEAM boot failure under fleet contention. | CONTENTION FLAKE — passes isolated (6.66s ok) |
| 4 | `composed_transit_1m` (cgap-aoq `macro_transit_bench`) FAIL: `stage loop Blake3HashOnly allocated, left: 2 right: 0` | Global ALLOC_CALLS census captured 2 allocations not present in isolated runs (environmental noise during fleet load). Input is 128 B (`WIRE_BYTES=64+BODY_CAP`), so `cgap_blake3::hash` takes the stack-only single-chunk fast path; no allocation exists in the stage loop itself. | CONTENTION FLAKE — passes isolated (203.8s ok) |
| 5 | `i2_stall_walk_isolation` (cgap-aoq `macro_p99_torture`) FAIL: isolation fence ratio exceeded | p99 latency fence exceeded under host load. AF-43 owns this file (edited 00:30 during wave). | CONTENTION FLAKE — passes isolated (24.3s ok) |

## Fix

`scripts/verify_v26.10.31.sh:136` — `-p cgap-cli` replaced with
`--manifest-path crates/cli/Cargo.toml` (the convention CLAUDE.md already
documents for cli side-bins). Verified directly: the command now exits 0,
battery prints `7 rows, 0 FAIL, skips=[FR-07]`, `exit_ok:true`.

## Gate results this session (isolated runs)

- `cargo test -p cgap-aoq --test macro_transit_bench -- --test-threads=1` → ok (1 passed, 1 ignored)
- `cargo test -p cgap-aoq --test macro_p99_torture i2_stall_walk_isolation -- --test-threads=1` → ok
- `cargo test -p cgap-git --test lineage_workflow` → ok
- `cargo test -p cgap-beam --test kill_stress` → ok (6.66s)
- `verify_v26` CLI battery → exit 0, 7 rows, 0 FAIL, typed skip FR-07

## Foreign / disclosed

- `macro_p99_torture.rs` compile errors, CHANGELOG, CLAUDE.md: owned by AF-43
  (in flight during this lane; file edited 00:30 while triage ran).
- Full-script re-run was launched in background; its final summary lands in
  `docs/jira/v26.10.31/verify-latest.md` (regenerated per run).
- Residual risk (not fixed, disclosed): gates 2-4 are contention-sensitive
  courts; a CI runner without fleet co-tenancy should be stable, but the
  shared-host fleet fan-out can re-flake them. Structural fix (per-test
  isolated target dirs or a fleet-wide build lease) belongs to a separate lane.
