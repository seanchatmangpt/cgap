# Lane LSPA-1 Receipt: shutdown-seal wiring (G07 orphan closure)

Date: 2026-11-02. Repos: /Users/sac/lsp-max (code), /Users/sac/cgap (docs).
Wave: v26.11.2. Branch: main, NO git commands run.

## Files written

- `/Users/sac/lsp-max/crates/branchless-lsp/src/lib.rs` — dispatch-named
  exception to the general lib.rs off-limits rule. Wiring: (1) session
  accumulator field `session: Mutex<SessionConformance>` on
  `BranchlessLspServer` + `session_conformance()` accessor; (2)
  `Tier2State.session_seal: Option<[u8; 80]>` slot; (3) `record()` folded
  in `did_save` before the vector moves into the live slot; (4) `shutdown()`
  seals + stores + one typed `window/logMessage` line
  (`CONFORMANCE-SEAL admitted=.. refused=.. residual=.. alu=.. load_store=.. tag=..`).
- `/Users/sac/lsp-max/crates/branchless-lsp/tests/conformance_seal_court.rs`
  (new, 161 lines) — real tower loopback (G06 pattern, did_save_court.rs
  read not edited), 4 courts.
- `/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-lspa-01-shutdown-seal.md`
  (this receipt).

## Seal surface decision (documented per dispatch)

The `LanguageServer::shutdown` trait signature is fixed at `Result<()>`
(lsp-max `src/language_server.rs:29`) — no response-extension slot exists.
Minimal honest surface chosen: store the 80-byte seal in
`Tier2State.session_seal` (public, court/tooling consumers) plus one typed
log line. Documented in the field doc comment.

## Courts (anti-vacuity)

- Flip court `different_save_vectors_flip_the_seal_bytes`: two sessions,
  branchy vs clean real rustc compiles — different seal bytes AND different
  decoded counters.
- `shutdown_seals_a_nonzero_session_receipt`: branchy save folds into the
  session totals (>=1 admit/refuse/residual), shutdown mints a non-zero
  80-byte seal, `from_seal` round-trips all 5 counters exactly.
- `session_totals_observable_before_shutdown`: two clean saves -> exactly
  2 admitted in the running totals.
- `empty_session_seals_zeroed_framing`: a save-free session still mints a
  well-framed zeroed seal.
- Prior G07 unit courts (flip/saturation/zero-alloc) untouched and still
  in the run.

## Disclosed-in-both-directions

LSPA-3 applied a compile-freeze SLA minimal-unblock to my in-flight
`shutdown()` in lib.rs:187-203 (log_message positional signature,
LowerHex absent, and a `future not Send` error: session MutexGuard held
across the `log_message` await — totals copied out, guard dropped before
await). Disclosed here per law; the fix is mine to own going forward.

## Commands + exits

Backfilled 2026-11-02 by lane GAP-10 (fresh run this session, not
cited from memory). Binary freshness check first: `strings
target/release/branchless-lsp | grep -c ANDON_REFUSED` = 3, exit 0 —
matches the G05/dod-lsp-max-wire.md:38-40 witness, binary not stale.

HONEST RESULT: the suite does NOT currently pass end-to-end. Per-suite
(`cd /Users/sac/lsp-max && CARGO_TARGET_DIR=/tmp/gap10-target cargo test
-p branchless-lsp`, log /tmp/gap10-suite.log):

| Suite | Result | Exit evidence |
|---|---|---|
| lib (src/lib.rs) | 72 passed / 0 failed | gap10-suite.log:405 |
| unittests src/main.rs | 0 passed | gap10-suite.log:411 |
| code_actions_courts | 5 passed / 0 failed | gap10-suite.log:422 |
| conformance_seal_court | 3 passed; `session_totals_observable_before_shutdown` HANGS (>60s at :441, never finished; cross-lane corroborated: LSPA-1's own independent run `/tmp/lspa1-target` stuck in the same court, pid 35280, since 14:42 — deterministic hang, not load) | gap10-suite.log:437-441 |
| did_save_court | 6 passed / 1 FAILED: `lspa3_measured_budget_admits_branchy_on_wire` (did_save_court.rs:311 — max_branches=2 measured budget expected to admit the branchy fn, got 2x ANDON_REFUSED + 1 blocking-without-push) | /tmp/gap10-other-suites.log:57-71 (own build root /tmp/gap10-target2) |
| e9_precision_court | 6 passed / 0 failed | fresh run, exit 0 |
| riscv32imc_court | 3 passed / 0 failed | fresh run, exit 0 |

Hang mechanism (observed, not fixed — surface is LSPA-1/LSPA-2's):
`session_with_saves` two-save sessions fill the bounded loopback client
channel; the helper's own comment (conformance_seal_court.rs:56-57)
documents that publish_diagnostics blocks when nothing drains it. The
hang reproduces across independent CARGO_TARGET_DIRs, so it is the
court/server, not the build.

Standing consequence: this receipt's earlier claim set describes the
seal wiring; with one court hung and one did_save court red, the
shutdown-seal surface is PARTIAL, not ALIVE, as of this backfill. The
unfilled "TO BE FILLED" was flagged by receipt_lint RECEIPT-STRUCT-002
(doctrine-ship-plan.md:82-84); it is now filled with what RUNS — a
backfill that cannot be cited as a green gate.
