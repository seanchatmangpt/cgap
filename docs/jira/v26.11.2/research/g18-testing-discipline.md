# R18 — ggen testing discipline (Explore brief)

Lane: R18. Source: `/Users/sac/ggen`. Claims cited file:line. Purpose: cgap v26.11.2
verification-ladder sections.

## Test organization

- Per-crate tests/ integration dirs + root tests/ (bdd, chaos, contract, e2e, integration,
  perf, security, stress, unit, chicago_tdd, clnrm, proof, otel_validation). Largest:
  crates/ggen-engine/tests/ (70+ files incl. cli_boundary.rs, dogfood_regression.rs,
  cross_pack_matrix.rs, dozens of *_pack_e2e.rs).
- Gated recipes, not ad hoc cargo (justfile): test-lib (30s→600s cold-compile escalation,
  justfile:181-198), test-integration (enumerates real kind==["test"] targets via cargo
  metadata, requires --features integration,a2a,mcp, :~310-330), mutate-replay (40-min
  bounded, :200-228), test-phase2, slo-check.
- Honest gate wiring: test-integration deliberately NOT in pre-commit because 15/372 targets
  remain genuinely red (TECH-DEBT-003, justfile:249-330) — "documented, explicitly time-boxed
  exception" vs "stays wired in and red".
- Pre-commit chain (justfile:534): fmt-check → check → lint → test-lib → coherence-check →
  guard-process-intelligence-boundary → guard-cheat-scan → guard-short-test-timeout →
  guard-fail-open-subprocess → guard-claims-schema → guard-pack-proofs →
  guard-generation-hash-pin → guard-pack-count → guard-gate-count → guard-pack-e2e-coverage →
  guard-ggen-toml-schema-parity → self-play → court-* (18+ gates).

## Chicago-style (strong evidence)

- Constitutional mandate: CONSTITUTION.md:58-64 Law IV "Chicago TDD (Zero Tolerance)" —
  state-based, real collaborators (real engines/files/subprocesses via CliHarness), reference
  oracles over hand-written assertions; Mechanized: true via cheat-scanner.
  CONTRIBUTING.md:149-200: "London TDD patterns (mocks, test doubles, behavior verification)
  are FORBIDDEN" — bans mockall, InMemoryStorage/FakeDatabase, .expect_x().times(1),
  DI-for-testability; prescribes reqwest real clients, SqlitePool :memory:/testcontainers,
  tempfile::TempDir, real LLM calls with OTEL verification.
- Mock gate mechanized: crates/ggen-cheat-scanner/src/lib.rs — CHEAT-T01 vacuous-assert (:65),
  CHEAT-T02 tautological-result-check (:71), CHEAT-T03 no-assertion-test (:77),
  CHEAT-T04 mock-import (:83-86): flags use mockall (:418-427), #[automock] (:435-447),
  cross-file MockXxx/FakeXxx impls standing in for traits real production code also
  implements (:534-606). Wired via guard-cheat-scan (justfile:740-741). Honest debt note:
  464 pre-existing findings (7 T01 / 456 T03 / 1 T04) tracked TECH-DEBT-001
  (justfile:729-738; CONSTITUTION.md:64) — detector live, workspace not yet clean.
- Real-subprocess harness: crates/ggen-engine/tests/cli_boundary.rs:1-13 — "spawn the real
  ggen binary as a subprocess via chicago-tdd-tools CliHarness. No mocks — real argv, real
  exit codes, real filesystem effects." Harness from dev-dep chicago-tdd-tools = 26.8.9
  (features testing-extras, testcontainers, cli-proof; Cargo.toml:245).
- Anti-fake-evidence: "evidence over narration — no completion claim without the command run
  and its exit code" (CONSTITUTION.md:114); claims ledger docs/aps/claims.toml
  (ALIVE/PARTIAL/BLOCKED/UNVERIFIED + named falsifier command + commit coordinate; declared
  nonclaim: single-maintainer CI is independent execution, not independent certification).
  Generated VERIFICATION.md:1-28 admits red checks with reasons.

## Test naming

Behavior-descriptive snake_case, no test_1: sync_run_second_invocation_is_idempotent
(cli_boundary.rs:237), ggen_regenerates_route_files_byte_identically
(dogfood_regression.rs:63), both_engines_sync_a_gated_pack_to_identical_outputs /
violating_fixture_is_refused_under_both_engines (reasoner_independence_e2e.rs). File names
encode tier: *_e2e.rs, *_test.rs, *_regression.rs, combinatorial_matrix.rs, *_court*.rs.
Claims-ledger falsifiers reference exact test binaries — names are load-bearing evidence
coordinates.

## Census / coverage

No llvm-cov/tarpaulin/grcov anywhere; "80%+ coverage" is prose, not a measured gate.
**Mutation testing is the measured oracle**: mutate-replay + mutations/mutants-replay.toml
(sync.rs, envelope, write, court/*, generated/*; killing oracles = determinism/sync test
binaries + envelope_receipt_binding_test dispatched by name). Census guards:
guard_wire_code_census.py (diffs wire codes vs typed_boundary_census.rs; exit 2 CENSUS_STALE,
exit 3 SKIP-loud); guard-pack-e2e-coverage.sh (ratchet baseline; "covered" = content-based
grep of pack name inside tests — filename-inference version proven wrong by execution and
replaced); guard-pack-count.sh, guard-gate-count.sh (count ratchets). E2E tier matrix:
TEST_INFRA.md:26-30 (Tier 1 ≥5 ontology tests, Tier 2 ≥5 boundary, Tier 3 pairwise, Tier 4
≥2 full compiled workspaces).

## Flake handling

- Load-sensitive-timeout guard (mechanized): scripts/ci/guard_short_test_timeout.py — born
  from a real 2026-08-01 flake (recv_timeout(500ms) false-failing under CPU contention);
  flags sub-1000ms recv_timeout/join_timeout/wait_timeout literals in test files, poll-loop
  carve-out (:41-60). Wired as guard-short-test-timeout (justfile:744-746).
- Fail-open subprocess guard: guard_fail_open_subprocess.py — flags Command::output() whose
  failure branch only logs; motivated by the v26.8.1 coverage_projection stale-evidence bug.
- Timeout escalation, not retries (30s→600s on cold compile, justfile:181-198). No automatic
  test-retry infra; philosophy fix-and-guard over retry-and-forget. "A gate that can hang
  forever is strictly worse than one that fails fast and red" (justfile:267).
- #[ignore] with reasons: 212 `#[ignore = "..."]` attributes, each carrying a cause — loudly
  skipped, never silently dropped.

## MOCKED gates

No gate literally named MOCKED; mock-policing is guard-cheat-scan (CHEAT-T04). Only "mocked"
language elsewhere: OTEL verification skill rules (span evidence "may be mocked" → UNVERIFIED)
— evidence-authenticity classification, not a CI gate.

## Ladder sections for cgap v26.11.2

1. Foundation: Constitution Law IV + forbidden-patterns list; cheat-scanner mechanized with
   honest debt disclosure.
2. Execution ladder: test-lib → test-integration (deliberately unwired, ratchet to promotion)
   → mutate-replay → slo-check → 18-gate pre-commit.
3. Evidence ledger: claims.toml falsifier coordinates + generated VERIFICATION.md red-state
   disclosure.
4. Census/ratchet gates: pack-e2e-coverage baseline, wire-code census, count ratchets.
5. Flake doctrine: guards born from named real bugs, no-retry policy, reason-carrying
   #[ignore].
