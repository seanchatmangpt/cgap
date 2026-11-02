# Lane G06 — LSP did_save court (branchless-lsp)

Repo: /Users/sac/lsp-max (branch main). Date: 2026-11-02.

## Files written (ownership)

- `crates/branchless-lsp/tests/did_save_court.rs` (NEW — the court; dispatch
  named src/server.rs, but no server.rs exists and the did_save wiring is
  complete in lib.rs:169-202, so per the dispatch's fallback clause the court
  landed in the allowed tests/ dir instead — no wiring duplication).
- `crates/branchless-lsp/src/lib.rs` (ONE hunk, did_save): run_tier2 now runs
  on `tokio::task::spawn_blocking` — fixes the named async gap (a seconds-scale
  `rustc` subprocess was compiled inline on the async request loop).
- `crates/branchless-lsp/Cargo.toml` (+4 lines `[dev-dependencies]`:
  tower 0.4 util, futures 0.3) — REQUIRED for the wire court: the initialize
  state gate (`mod state` is private in lsp-max `service.rs:27`) can only be
  lawfully passed through the real tower `Service::call(initialize)` path,
  which needs the tower trait, and socket reads need futures StreamExt.
  Disclosed file-ownership extension.

## Inspection finding (verify-don't-rewrite)

did_save -> run_tier2 -> verdict_diagnostics -> publish_diagnostics ->
Tier2State/conformance update was ALREADY wired (lib.rs:169-202; inventory
witness confirmed). Not rewritten. The only gap found was the blocking gap;
fixed minimally as above. Debounce on rapid saves NOT added: each save
compiles once and handlers serialize correct verdicts; coalescing would
require Arc-shared gate state (structural change) and is left as a recorded
divergence, not silently skipped.

## Commands + exits (this session, exact subject)

- `CARGO_TARGET_DIR=/tmp/g06-target cargo test -p branchless-lsp --test did_save_court`
  -> exit 0, 4 passed / 0 failed (branchy ERROR+name, clean zero-ERROR,
  allowed-flip zero-ERROR, Tier2State+wire-conformance arms).
- `CARGO_TARGET_DIR=/tmp/g06-target cargo test -p branchless-lsp`
  -> exit 0: 72 lib + 4 did_save + 5 code_actions + 6 e9 + 3 riscv32imc
  = 90 passed / 0 failed.
- Non-vacuity corroboration: raw
  `rustc --crate-type=lib --emit=obj -O -g --target aarch64-apple-darwin`
  on the probe fixture = 45ms wall, /tmp/g06_probe.o = 3536 bytes -> the
  0.05s per-test wall time is a REAL rustc+capstone+DWARF pipeline, not a
  stub fast-path (compiler_worker has no cache; unique staging per call).

## Court shape (Chicago)

Real collaborators only: real LspService + Client/ClientSocket loopback, real
tower initialize (passes the real state gate), real rustc --emit=obj compile,
real capstone disasm court, real wire publishDiagnostics notification read off
the socket, parsed from the JSON-RPC Request. Asserts final state: severity
ERROR, message carries `g06_branchy`, code == ANDON-REFUSED-001; the CLEAN
and ALLOWED arms are the anti-vacuity flip (same pipeline, zero ERROR).
Bounded poll on the socket observable (tokio::time::timeout 180s); no sleeps.

## Standing

ALIVE for the court and the blocking-gap fix: both were compiled and executed
this session against the exact subject (commands+exits above).
UNVERIFIED: the debounce divergence (not implemented, by scope); G05's
live-binary proof (separate lane).
No mocks, no Process.sleep. Shared tree /Users/sac/cgap untouched except this
receipt. lsp-max tree: no git commands run.

## Fences (must-not-regress) and their runs

- Fence 1: existing branchless-lsp suites (rule_pack flip courts, e9
  precision, riscv32imc, code_actions) — RAN, all green with my lib.rs hunk
  in the build (90/90).
- Fence 2: the main.rs binary build (stdio entrypoint compiles against the
  edited did_save) — RAN as part of `cargo test -p branchless-lsp` (main.rs
  unittests compiled+ran, exit 0).
- Fence 3: lsp-max root crate untouched (read-only); cgap repo untouched
  except this receipt.

## Divergences from dispatch

1. Dispatch named src/server.rs; wiring lives in lib.rs (no server.rs
   exists). Per the dispatch's own fallback clause, edits landed in lib.rs
   and the court in tests/. Not duplicated.
2. Cargo.toml dev-dependencies addition (disclosed above) — outside the
   named file set, required for the wire court; minimal (2 crates, both
   already in the workspace dep graph).
3. Debounce not implemented (scope: minimal fix; recorded, loud).
