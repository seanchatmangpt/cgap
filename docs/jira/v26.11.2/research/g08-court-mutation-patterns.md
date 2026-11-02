# R8 — ggen court/mutation patterns (Explore brief)

Lane: R8. Sources: `/Users/sac/ggen/crates/ggen-engine/src/court/`,
`/Users/sac/ggen/scripts/`, `/Users/sac/ggen/mutations/`. Read-only; claims cited file:line.

ggen implements a full "court" discipline: transiently mutate a subject, prove a witness
flips, restore byte-exact, fail closed on anything else, typed exit codes (0/1/2/3).

## 1. Canonical exit-code contract (engine-level)

- `crates/ggen-engine/src/court/mod.rs:44,46,48` — `EXIT_VACUOUS = 1`,
  `EXIT_COURT_BLOCKED = 2`, `EXIT_SKIP = 3` (doc table :25-27: 0=PASS, 1=VACUOUS GATE,
  2=COURT_BLOCKED, 3=loud SKIP, never silent).
- `court/mod.rs:93-135` — `CourtVerdict` enum (Pass/Refused/Unknown, "never collapsed, never
  silent-pass") with `exit_code()`: Pass→0, Refused("WITNESS_STALE")→1, other Refused→2,
  Unknown→3.
- CLI surface `crates/ggen-cli/src/cmds/court.rs:7-11` documents the same contract.
- `court/compile.rs:55-90` — witness-ordering rule: flip authorized only when
  `witness_ts >= shim_ts`; stale witness = Refused(WITNESS_STALE) exit 1; missing timestamps =
  Unknown (loud SKIP exit 3).
- Variant in ggen-abb-sbb: `tests/envelope_courts.rs:17-19` (Pass=0, VacuousGate=1, Refused=2)
  — collapses SKIP into Refused; the general contract is the engine one.

## 2. Shim-court protocol

- `scripts/shim_court_lib.sh:1-23` — SHA-BASELINE → BEFORE (old signature holds) → SHIM
  (subject edited in place, must carry `SHIM-COURT` marker) → AFTER (new outcome must hold;
  this flip IS the non-vacuity proof) → RESTORE (sha256 must equal baseline byte-exact,
  fail closed on mismatch).
- Exit contract: 1 = VACUOUS GATE / `[FAIL:PHASE]`; 2 = COURT BLOCKED (:57-62); 3 = SKIP-loud
  (:66-70); witnesses run under `bash -uc` so unset vars fail loud (:100-103).
- Self-check `shim_court_self_check` (:76-99): `dd`-corrupts a real mktemp copy and asserts
  `corrupt != base` before any real shim — proves restore-detection is non-vacuous.
  Required-first by `scripts/fresh_epoch_lease_court.sh:34-36`.
- Interrupt safety `shim_court_trap` (:103-114): byte-exact restore on EXIT/INT/TERM mid-AFTER;
  `[FAIL:RESTORE]` if restore doesn't verify.

## 3. Concrete courts

- `scripts/fresh_epoch_lease_court.sh:1-60` — shims
  `crates/ggen-engine/src/write.rs::check_lease_generation` (early `Ok(()) // SHIM-COURT`);
  BEFORE = backdated-receipt test passes pristine; AFTER = same test must FAIL under shim,
  else exit 1 VACUOUS GATE; RESTORE byte-exact fail-closed. Exits documented :21-24.
- `crates/ggen-abb-sbb/tests/envelope_courts.rs:92-188` — Court 1: nominal cap → exit 0 +
  stable digest; Court 2: mutants must flip verdict nonzero; Court 3: undeclared cap → typed
  VacuousGate exit 1 ("the exact vacuity shape the court exists to kill").
- `crates/ggen-abb-sbb/tests/falsifiers.rs:788-804` — `tampered_receipt_is_refused` →
  `Refusal::ReceiptTampered`.
- `crates/ggen-config/tests/generated_mutation_court.rs:1-60` — anti-vacuity over a generated
  mutant surface: every generated mutant must move the canonical-JSON digest (:24
  "vacuous gate"); canonicalization deterministic (:44-56).
- `crates/ggen-config/tests/canonical_identity_courts_test.rs:98-129,173-182,234` — fn-pointer
  mutant array (one mutation per semantically meaningful field), each MUST move the SHA-256
  digest ("mutant left the digest unchanged — identity is vacuous").
- `crates/ggen-cli/tests/sabotage_tests.rs:187-350` — fail-closed sabotage: remove pack.toml
  under `sync --locked` → nonzero; corrupt ggen.lock → nonzero; delete verifying key → receipt
  verify returns INVALID (never fake pass); empty packs dir install → nonzero typed stderr,
  no receipt.
- Court-compiler seam: `crates/ggen-engine/src/court/{mod,catalog,compile}.rs`, templates
  `crates/ggen-engine/templates/court/{runner.sh,register_row,fixture.rs}.tera`; cataloged in
  `docs/jira/v26.10.20/RESOLUTIONS.md:23`.

## 4. Mutation testing (cargo-mutants)

- `mutations/README.md` — MU3 lane over `crates/ggen-engine/src/sync.rs` (243 mutants),
  test-side kills only, 40-min bound per pass, RESUME accumulates; baseline v26.10.5 @
  c1c4703a: 243 generated / 6 tested / 5 caught / 0 survivors — explicitly SAMPLED, not
  exhaustive. Root-causes hazard: stale installed `ggen` on PATH silently poisons copied-tree
  runs (fix `--copy-target true`).
- `justfile:206-232` — `just mutate-replay` (two chained passes, oracle filters
  determinism/sync, outcomes in mutations/outcome-replay-a/b); :359 workspace mutation score;
  :631 court-test invocation of canonical_identity_courts_test + generated_mutation_court.
- `scripts/ci/mutants_wave_report.md` — honest non-completion report: cargo-mutants refused to
  run on an unmutated tree with 2 pre-existing defects (compile break
  fresh_epoch_lease_court_test.rs:256; `ggen sync run` exits 0 where typed non-zero refusal
  required, envelope_undeclared_cap_e2e.rs:118 — flagged REAL finding, report-only).
- Config: `mutations/mutants-replay.toml`.

## 5. Fail-closed guards / CI courts

- `tools/truth-gate/src/main.rs:1-8,144,156,191,217` — hook mode exits 0/2; `--ci` scan mode
  propagates typed exit; fail-closed policy modules.
- `scripts/ci/guard_fail_open_subprocess.py` + `test_guard_fail_open_generated.py:1-60` —
  flags fail-open verifier shapes (`!status.success()` branch that only logs); RED test feeds
  a real fail-open fixture and fail-closed sibling, asserts exactly one is caught.
- `scripts/checks/byte-identity.sh:1-56` — re-sync idempotency court: hash every
  receipt-listed output, real second `ggen sync run`, exit 0 iff every hash matches; exit 3 on
  missing receipt, exit 4 on build failure.
- `scripts/ci/dialect_declaration_court.py` + test — dialect admission court with refusal
  registry.
- `scripts/verify-evidence.sh:8,48` — records REAL exit codes as `ver:Check` facts
  (no fabricated evidence); `scripts/produce-receipts-check.sh:11`.
- Court/mutation enforcement lives in justfile recipes + `scripts/ci/guard-*`, not in
  ci.yml/quality.yml.

## 6. Reusable court-pattern catalog

| # | Pattern | Mutation | Restore | Citation |
|---|---|---|---|---|
| 1 | Typed exit contract | — | fail-closed verdict mapping | court/mod.rs:44-48,93-135 |
| 2 | Shim court (5-phase, marker-enforced) | in-place edit w/ SHIM-COURT marker | sha256 byte-exact fail-closed + trap | shim_court_lib.sh:1-200 |
| 3 | Restore-detection self-check | dd corruption of temp copy | clean+corrupt pairs | shim_court_lib.sh:76-99 |
| 4 | Witness-ordering (no stale authorization) | — | WITNESS_STALE → 1 | court/compile.rs:55-90 |
| 5 | Digest-move anti-vacuity | fn-pointer mutant array | N/A | canonical_identity_courts_test.rs:98-129 |
| 6 | Generated-surface mutation court | generated mutant fns | N/A | generated_mutation_court.rs:1-60 |
| 7 | cargo-mutants lane | 243 sync.rs mutants, bounded+resumable | lib untouched | mutations/README.md; justfile:206-232 |
| 8 | Sabotage suite | corrupt lock/delete key/empty dir | typed nonzero, no receipt | sabotage_tests.rs:187-350 |
| 9 | Re-sync idempotency court | real second sync | hash-before/after | checks/byte-identity.sh |
| 10 | Fail-open detector (+ own RED test) | fail-open fixture vs fail-closed sibling | guard assertion | scripts/ci/test_guard_fail_open_generated.py |
| 11 | Honest non-completion reporting | zero-executed mutants disclosed | — | mutants_wave_report.md |

Key distinction: BLOCKED (subject not exercisable, exit 2), SKIP (environment missing, exit 3,
always loud with reason), VACUOUS (gate ran but proved nothing, exit 1); a court whose AFTER
witness still passes under the shim is the defined vacuity event, never a pass.
