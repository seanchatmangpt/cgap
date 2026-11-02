# DoD audit — EXP-03, crate cgap-bn3 (2026-11-02)

Auditor lane EXP-03, read-only except this file.
Exit codes recorded per command, status vocabulary per no-overclaiming law.

## T1 — claims vs code

- RADIX_BUCKETS=4096 pinned:
  `crates/cgap-bn3/src/unify.rs:840` `pub const RADIX_BUCKETS: usize = 4096;`
  (doc lines 836-839 cite 13-arrow-star-schema.md, 16 KiB heads).
  Private hash `radix_bucket` unify.rs:850-852,
  `((y ^ (y>>16)) & 0x0FFF)` on the 56-bit payload window.
- Batch probing:
  `unify_batch_probe` (unify.rs; E2/E3/E5 exercise it via
  tests/edge_closure_courts.rs:205/311/458), `unify_to_fixpoint`
  unify.rs:1523, `DEFAULT_CHUNK_ROWS=4096` unify.rs:194.
- C03 courts landed: tests/edge_closure_courts.rs, 5 tests, all pass
  this session (see T2). E3 pins the REAL bound:
  `assert_eq!(RADIX_BUCKETS, 4096, "L3 bucket count (the REAL bound)")`
  tests/edge_closure_courts.rs:269; plus a court-side hash replica with
  self-check (lines 248-291) so a kernel hash change fails loudly.
  Verdict: courts pin the real bound. ALIVE.
- Publish metadata: Cargo.toml `publish = true` with P3-2b guard-flip
  comment citing A11 receipt
  (docs/jira/v26.11.1/waves/a11-finish-receipt.md — exists).
  BUT docs/jira/v26.11.2/waves/publish-halt.md: operator freeze
  ("do not publish anything yet"); cgap-bn3 is in the FROZEN set.
  Verdict: metadata ready, publication FROZEN-BY-OPERATOR.

## T2 — cargo test -p cgap-bn3 (CARGO_TARGET_DIR=/tmp/exp03-target)

Command: `CARGO_TARGET_DIR=/tmp/exp03-target cargo test -p cgap-bn3`,
exit 0. Counts:
- lib unittests: 119 passed, 0 failed (12.49s)
- bgp_hop_courts: 2 passed, 2 ignored (#[ignore = "expensive debug
  run - run with --release"], bgp_hop_courts.rs:269,275)
- edge_closure_courts: 5 passed, 0 failed (0.02s)
- lever_courts: 7 passed (19.51s)
- repair20_xorfilter_gate: 3 passed (19.02s)
- doc-tests: 0
Total: 136 passed, 2 ignored, 0 failed.

Coverage gaps (spot-check, honest inventory — not exhaustive):
- bgp: no src/bgp.rs in this crate; coverage lives in
  tests/bgp_hop_courts.rs only, and its 2 release-scale courts are
  #[ignore]d in the default run — the ignored half is UNVERIFIED in a
  debug-profile run.
- unify.rs: 88 fns vs 32 tests in the single cfg(test) module
  (unify.rs:1617); heavy internal machinery (probe cache, join masks,
  retraction bookkeeping) exercised mostly indirectly via integration
  courts; direct unit coverage is the minority.
- sigil.rs: 24 fns vs 12 tests (sigil.rs:454) — roughly half the fn
  surface has no direct test.
- builtins.rs: 26 fns vs 8 tests; noalloc.rs: 8 fns vs 8 tests
  (two cfg(test) blocks, lines 139/617).
- No bench directory in this crate; timing behavior untested here.

## T3 — the 22.99 ms closure figure

Measured fact only, NOT a test gate in cgap-bn3:
- Zero occurrences of "22.99" in crates/cgap-bn3 (src+tests).
- Zero occurrences in crates/bench/benches (no closure gate there).
- It lives as a CONSTANT CLAIM value:
  crates/csct/src/new_way.rs:112/158 (`closure_ms_1m: 22.99`) fed to
  `evaluate()` which compares observed vs `spec.max_closure_ms_1m`.
  That gate compares a hardcoded claim against a hardcoded spec
  threshold — it does NOT measure anything. The measured-fact status
  is further downgraded by
  docs/jira/v26.11.2/repairs/repair-20-xorfilter-gate.md:57-59:
  "The 22.99 ms fence number ... was NOT reproduced this session
  (both arms 24.2-25.3 ms median) ... recorded UNVERIFIED against
  this session's method". lane-git-09:61 also keeps it as "witness
  context figure, never claimed here".
Verdict: 22.99 ms is receipt/witness documentation plus a self-
referential CSCT assertion over constants. It is NOT an executable
gate over measured cgap-bn3 behavior, and it is already flagged
UNVERIFIED. Verdict: measured-fact / witness figure only.

## T4 — allocator hooks (E5 zero-alloc)

tests/edge_closure_courts.rs:43-50: AE7 `EnclaveTrackingAlloc<System>`
wired via `#[path = "../../cgap-core/src/tracking_alloc.rs"]` (cgap-core
lib.rs does not re-export the module — recorded divergence, disclosed
in the court doc lines 37-44).
- Asserts 0 bytes: yes — E5 window 1 lines 491-496 asserts
  `(allocs, bytes) == (0, 0)` over 20 steady-state probe calls.
- Positive control: yes — window 0 lines 428-439: budget(0,0) + real
  `vec![7u64;4]` must breach, site name asserted.
- Honesty arm (window 2, lines 504-513): fixpoint driver asserted
  ALLOCATES (>0 allocs, >0 bytes) — not overclaimed as zero-alloc.
Verdict: steady-state 0-alloc claim is ALIVE and self-critical.

## Cleanup honesty

`rm -rf /tmp/exp03-target` was DENIED by the permission system.
/tmp/exp03-target still exists and is owed operator cleanup.

## Overall verdicts

- T1 claims vs code: PASS (RADIX_BUCKETS pinned unify.rs:840; courts
  pin the real bound, edge_closure_courts.rs:269).
- T2 tests: 136 passed / 2 ignored / 0 failed, exit 0.
- T3 22.99 ms: documentation/witness figure, NOT a gate; UNVERIFIED.
- T4 E5 allocator: PASS — 0-byte assert + positive control present.
- Publish: metadata `publish = true` (Cargo.toml, A11 receipt-backed),
  but publication is FROZEN-BY-OPERATOR per
  docs/jira/v26.11.2/waves/publish-halt.md; no publish action
  authorized.
