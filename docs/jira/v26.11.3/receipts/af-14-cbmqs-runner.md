# AF-14 CBMQS Runner Receipt
Standing: ALIVE

- Lane: AF-14 (implementation lane, Prompt 08 executable counterpart)
- Date: 2026-11-03
- Subject: `/Users/sac/cgap` — runner, doc, receipt, `Cargo.toml`
  dev-dep/test-target addition to cgap-aoq. No git operations run.

## What landed

- `crates/cgap-aoq/tests/cbmqs_runner.rs` — the CBMQS bake-off RUNNER
  (`harness = false` test target; the process exit status IS the
  verdict): 7-gate standard CBMQS-7G/1 (G1 alloc census, G2 timing
  census, G3 mock scan, G4 census replay determinism, G5 storm feed,
  G6 kill-cycles, G7 BLAKE3 report seal), self-test mode (subject =
  cgap-aoq in-process), `--court` non-vacuity mode, foreign-subject
  subprocess mode (frame protocol documented in
  `docs/cbmqs-runner.md`), gate-subset flag, BLAKE3-sealed JSON report
  stamped with the runner version (`26.11.3+af14.1`).
- `crates/cgap-aoq/Cargo.toml` — `[[test]] cbmqs_runner harness=false`
  + `cgap-blake3` dev-dep (dev-only; the embedded lib surface stays
  zero-dependency).
- `docs/cbmqs-runner.md` — usage + subject contract + AF-13 alignment
  and registered divergences.
- This receipt.

## Exit records (this session, exact subject)

- Self-test: `cargo test -p cgap-aoq --test cbmqs_runner` → exit 0,
  verdict PASS. Measured: G1 ops=8192 allocs=0 bytes=0
  control_allocs=1 (positive control live); G2 p50=791 ns p99=917 ns
  (budget 10000 ns, debug profile); G3 src scanned, 0 patterns; G4
  run1=run2=(0a,0B); G5 fully typed storm
  (HeaderTooShort/DigestMismatch/BadMagic/UnknownEpochWire); G6 typed
  SKIP SUBJECT_IS_LIBRARY; G7 seal verifies
  (f210c8707fa35ceb...).
- Court: `cargo test -p cgap-aoq --test cbmqs_runner -- --court` →
  exit 0, "CBMQS COURT PASS". Witnesses: overalloc fixture G1
  FAIL OVERALLOC (tracemalloc-attested allocs, per-op delta real);
  missing-census fixture G1 FAIL CONTRACT_UNSUPPORTED; bound-flip pair
  (same subject exit 2 at bound 0, exit 0 at bound 2); good fixture
  G5 PASS + G6 PASS (3 SIGKILL/respawn cycles, liveness each cycle);
  mock-tree G3 MOCK_DETECTED → exit 3; tampered report refuses the
  BLAKE3 seal.
- Suite: `cargo test -p cgap-aoq` → exit 0, 16 test targets
  "test result: ok", 0 failed (includes AF-13's cbmqs_gates.rs, which
  landed mid-session — 1 passed / 0 failed).

## Defects the gates caught during bring-up (anti-vacuity receipts)

- G3 self-scan initially hit the runner's own file (pattern literal
  "mockall" in the scanner) — fixed to scan `src/` only.
- G5 self-storm epoch frame: 0xFFFF is the valid Epoch::Retired
  sentinel; the frame must carry 0x7FFF to exit the closed enum —
  fixed; the gate flipped to PASS only after the fix.
- G5 foreign digest_tamper frame: digest law is subject-specific, so
  the contract-typed expectation (Expect::OkOrTypedErr) replaced the
  hard-coded class demand. Fixture `sum(64)` TypeError caught by the
  court (C4/C5 EOF) and fixed.

## Registered divergences vs AF-13 (cbmqs_gates.rs)

- G2 debug-profile budget (10000 ns) vs AF-13 RELEASE-only <100 ns.
- G6 kill-cycles (generic) vs AF-13 AB-17 churn (PASS-CITED).
- No G1b slab-throughput gate; the runner's standard is CBMQS-7G/1.

## Standing

ALIVE. Self-test exit 0 + court exit 0 + full-suite exit 0, all this
session, exact subject, on disk. Never run git (per lane contract).

## Falsifiers (live checks, not prose)

- `cargo test -p cgap-aoq --test cbmqs_runner` must exit 0; a FAIL
  verdict prints the typed gate row.
- `cargo test -p cgap-aoq --test cbmqs_runner -- --court` must exit 0;
  any non-flipping gate prints COURT FAIL: <C#>.
- Tampered report bytes must fail `verify_hash_law` (court C7).

## Known limitation (disclosed, not papered over)

Foreign G1 trusts subject-attested census frames. Adversarial
verification of a foreign binary needs LD_PRELOAD allocator
interception or source instrumentation — registered as the next-lane
surface, not silently accepted as strong.
