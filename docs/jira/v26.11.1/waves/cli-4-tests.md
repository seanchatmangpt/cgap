# CLI-4 Test-Lane Receipt

- Date: 2026-11-01
- Lane: CLI-4 (tests for the v26.11.1 CLI crate-surface verbs)
- Repo: /Users/sac/cgap (canonical checkout, branch main, no git writes)
- Lease honored: only `crates/cli/tests/cli_new_verbs.rs` + this receipt.
  custom_handlers*, generated_*, ontology, Cargo.toml untouched by this lane.
- Target root: CARGO_TARGET_DIR=/tmp/c4-target (deleted at close).

## What landed

- `crates/cli/tests/cli_new_verbs.rs` — 23 Chicago courts over the real
  compiled binary (`CARGO_BIN_EXE_cgap`), extending the incumbent
  zero_code_cli.rs law:
  - `every_new_noun_verb_pair_resolves_to_usage`: all 17 admitted
    v26.11.1 pairs resolve (`--help` exits 0, usage printed).
  - `unknown_surface_refusal_still_fires`: unknown noun AND unknown verb
    on an admitted noun both fail closed (incumbent law extended).
  - 4 incumbent-verb courts: self version (== manifest version),
    align test (aligned == total > 0), bench mask / bench parquet
    (positive rates; min-of-5 receipts).
  - 17 handler smoke courts, each asserting handler-final state, e.g.:
    bont slot-lookup's printed slot re-derives independently in the test
    (blake3(BONT_NS_V1 ++ lexeme)[0..8] LE) — the §8 T4 court path;
    parquet pack-check / crypto receipt-verify / git dag-verify refuse
    their tamper witnesses; p28 wire_len == 80 with the frozen URN pin;
    beam info / court verdict-run assert real subprocess harness counts.

## Commands + exits (this session)

- `cd crates/cli && CARGO_TARGET_DIR=/tmp/c4-target cargo test`
  - first run: 4 passed / 2 failed (noun-verb argument order inverted by
    this lane; fixed in-test) → final:
    lib unit tests 12 passed / 0 failed; cli_new_verbs 23 passed / 0
    failed (26.85s); mutation_kills 6 passed / 0 failed;
    zero_code_cli 1 passed / 0 failed. TOTAL 42 passed / 0 failed.
  - isolated re-run `cargo test --test cli_new_verbs`: 23 passed / 0
    failed (26.83s), zero warnings from this lane's file.
- Smoke receipts (binary): all 17 pairs executed rc=0 with real kernel
  output (bont slot `0xdc41a81355c65e29`, git root
  `a08ef77c...f526af`, p28 wire_len 80, court verdict-run passed=14,
  beam info passed=2).

## Courts / anti-vacuity

- Unknown-surface refusal court fires (exit 1 both axes).
- Handler tamper witnesses asserted through the CLI: parquet pack-check
  (content byte flip refused), crypto receipt-verify (bit tamper
  refused), git dag-verify (wrong parent refused), p28 (over-ceiling
  refused), audit ocel-stats (two corruptions refused), gate dam-ingest
  (corrupt status refused), audit powl-stats (violating log refused).
- Independent re-derivation court: bont slot re-derived in the TEST from
  blake3(BONT_NS_V1 ++ lexeme), not by re-reading the handler's claim.

## Divergences / events

- CLI-2 landed mid-lane (polled ~13 min bounded; no cli-2-handlers.md
  receipt file existed at any poll — landed via handlers_kernel /
  handlers_reasoning / handlers_sovereign modules + regenerated
  generated_cli.rs instead). Smoke templates were taken from the landed
  handlers, not a receipt.
- Compile-freeze window ~10 min: CLI-2's first landed state had 19
  compile errors (missing cgap_beam dep, Display impls, MatchSet::len,
  mut bindings). This lane polled; CLI-2 repaired its own files within
  the SLA; no cross-lane unblock applied, nothing disclosed.
- `#[verb(a, b)]` is (verb, noun) — this lane initially inverted it and
  its own court caught it (2 failures), fixed in the test file only.
- `beam info` is a cargo-subprocess harness (cgap-beam is cdylib-only),
  not a bug: in-handler documented, asserted as counts.
