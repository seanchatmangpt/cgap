# CSCT Lane 09 Receipt — Mutation Meta-Court (RFC-0042 Rule 2)

Standing: **ALIVE (sandbox)** — compile+tests via /tmp sandbox copy; the shared
tree compile is UNVERIFIED:compile-pending because sibling stage lanes (L3–L8,
L10) had not landed their modules/tests at lane close, and root `Cargo.toml`
is off-limits to this lane.

## Identity

- Subject: `crates/csct/src/mutation.rs` (sole owned file) +
  `docs/jira/v26.11.2/waves/csct-lane-09-mutation.md` (this receipt).
- Contract honored exactly: `MUTATION_REGISTRY: [(&str,&str); 6]` (THE_REFRAME→zero-tax,
  RATIONAL_DROWNING→ledger-drift, EMOTIONAL_IMPACT→gc-quiesced, THE_NEW_WAY→gc-claim,
  THE_SOLUTION→input-flip, THE_WARMER→shard-starved); `validate(&[(&str,&str)]) ->
  Result<(), ExitCode>`; `run_mutation_controls() -> Result<(), crate::types::ExitCode>`
  with GateFail=VACUOUS semantics.

## Commands + exits

- `cargo test --manifest-path crates/csct/Cargo.toml --lib mutation` → exit 1
  (package outside workspace; root manifest off-limits to this lane).
- `cp -R crates/csct /tmp/csct-l9-sandbox`; sandbox-only: `[workspace]` already present
  (added by lane CSCT-L5 minimal unblock, disclosed per compile-freeze SLA); path dep
  `cgap-blake3` repointed to absolute; in-flight lane tests
  (`tests/csct_pipeline_court.rs`, contains a raw `STATUS_QUO_ANNUAL_USD_PLACEHOLDER`
  token, lane L-whatever owns it) removed; sandbox lib reduced to `types + mutation`.
- `cd /tmp/csct-l9-sandbox && cargo test mutation` → **exit 0, 4 passed / 0 failed**:
  - `mutation_controls_pass` (shipped registry Ok)
  - `missing_entry_flips_gate` (ANTI-VACUITY: 5-entry mutant → Err(GateFail)) — flips
  - `duplicate_stage_flips_gate` (distinctness) — flips
  - `empty_mutation_id_flips_gate` (empty id) — flips
- On-disk bytes verified post-write.

## Fences (must-not-regress surface of my diff)

- Root workspace build and other crates untouched; only `crates/csct/src/mutation.rs`
  written. `lib.rs` already declared `pub mod mutation;` pre-lane.
- Disclose-in-both-directions: lane L5's `[workspace]` append to
  `crates/csct/Cargo.toml` was consumed, not authored, by this lane.

## Divergences

- Contract said stage modules L3–L8 "each expose an #[cfg(test)]-visible evaluate-flip";
  at lane close `crates/csct/src/` held only warmer/drowning/types — reframe, emotional,
  new_way, solution, andon, receipt, court absent, and `tests/csct_pipeline_court.rs`
  does not compile (raw placeholder token
  `5_068_6 STATUS_QUO_ANNUAL_USD_PLACEHOLDER` in tests/csct_pipeline_court.rs:47).
  The contract's meta-court design (registry validation only) was implemented as
  specified; cross-stage evaluate-flip invocation is deferred to the owning lanes and
  the `court` module (L-whatever owns sequencing courts).
- Shared-tree `cargo test -p csct` remains compile-pending on sibling lanes.
