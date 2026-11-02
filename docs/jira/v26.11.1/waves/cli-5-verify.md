# CLI-5 verify — wave receipt

Date: 2026-11-01. Lane: CLI-5-verify (gate-runner). Repo: /Users/sac/cgap.

## Precondition deviation

`docs/jira/v26.11.1/waves/cli-4-tests.md` never landed during a bounded poll
(>2 min this session); it is MISSING, not ALIVE. The CLI-4 expansion landed
mid-session directly in the tree (ontology + custom_handlers.rs edits observed
concurrently), so verification proceeded on the on-disk surface.

## Ladder (all commands this session, cwd /Users/sac/cgap/crates/cli)

| Step | Command | Exit | Result |
|---|---|---|---|
| Build | `cargo build` | 0 | green (after two regen attempts; see incident) |
| Full suite | `cargo test` | 0 | 19 passed / 0 failed (12 lib unit + 6 mutation_kills + 1 zero_code_cli) |
| Version | `./target/debug/cgap self version` | 0 | `"26.11.1"` |
| Incumbent verbs | `self version`, `align test`, `bench mask`, `bench parquet` | 0 / 0 / 0 / 0 | all pass |
| New verbs (17) | `gate dam-ingest`, `p28 attest-verify`, `court verdict-run`, `bn3 unify`, `bn3 eval`, `bont pack-verify`, `bont slot-lookup`, `audit powl-stats`, `audit ocel-stats`, `crypto seal`, `crypto receipt-verify`, `arrow slab-stats`, `beam info`, `gpu info`, `git dag-verify`, `parquet pack-check`, `parquet pack-emit` | 1 (each) | typed refusal `UNIMPLEMENTED: <verb> handler lands in CLI-2` — routed, not broken |
| Incumbent law | `./target/debug/cgap frobnicate` | 1 | unknown surface refuses |
| Regen replay 1 | `rm Cargo.toml && ggen sync run && python3 manifest_patch.py` | 0 / 0 | projected 21 verbs from ontology |
| Regen replay 2 | same flow, cmp | 0 / 0 | `generated_cli.rs` and `docs/command-reference.md` BYTE-IDENTICAL across replays |

## Byte-stability verdict

No `waves/cli-1-ontology.md` receipt exists; the intended-change reference is
taken from the on-disk ontology delta itself: 21 `verb("noun", "verb")`
routes generated from 21 ontology-declared verbs, zero unexpected files
written by regen (git status delta vs pre-regen: generated_cli.rs +
regen-owned docs only). No unexpected delta. VERDICT: byte-stable.

## Incident (disclosed)

My first regen ran while the CLI-4 lane had landed the ontology but not yet
`custom_handlers.rs`; the projection referenced 3 not-yet-existing handlers
(`bn_3_eval`, `bn_3_unify`, `p_28_attest_verify`) and broke the shared build
(E0425). I restored `generated_cli.rs` from HEAD and my Cargo.toml backup
(compile-freeze SLA); once handlers landed (observed during poll), the regen
was redone and the build went green. Final tree state is the lane's intended
state, not my intermediate one.

## Standing: ALIVE

Full ladder green. New-surface section appended to crates/cli/README.md.
Docs written by regen: crates/cli/docs/command-reference.md (21 verb sections).

## Handler-green re-verify (CLI-2 landed) — 2026-11-01

CLI-2's 17 handler bodies landed in `custom_handlers.rs` + three handler
modules (`waves/cli-2-handlers.md`). Ladder re-run this session,
cwd /Users/sac/cgap/crates/cli:

| Step | Command | Exit | Result |
|---|---|---|---|
| Build | `cargo build` | 0 | green |
| Full suite | `cargo test` | 0 | 25 passed / 0 failed: 12 lib unit + 6 mutation_kills + 6 new CLI-4 tests + 1 zero_code_cli |
| Version | `./target/debug/cgap self version` | 0 | `"26.11.1"` |
| New verbs (17) | all of `gate dam-ingest`, `p28 attest-verify`, `court verdict-run`, `bn3 unify`, `bn3 eval`, `bont pack-verify`, `bont slot-lookup`, `audit powl-stats`, `audit ocel-stats`, `crypto seal`, `crypto receipt-verify`, `arrow slab-stats`, `beam info`, `gpu info`, `git dag-verify`, `parquet pack-check`, `parquet pack-emit` | 0 (each) | real kernel output each: e.g. dam-ingest `corrupt_refused: true / status 0x00000000`; p28 attest-verify frozen URN `urn:avec:license:p28-sil:v2.1.0-sovereign`; court verdict-run 14 passed / 0 failed (49s); bn3 unify 3/3 matches; crypto receipt-verify `tamper_refused: true`; git dag-verify Merkle root `a08ef77c…`; parquet pack-check `tamper_refused: true` |
| Incumbent law | `./target/debug/cgap frobnicate` | 1 | unknown surface still refuses |
| Byte-stability | regen replay (`rm Cargo.toml && ggen sync run && python3 manifest_patch.py`) | 0 / 0 | `generated_cli.rs` + `docs/command-reference.md` BYTE-IDENTICAL across replays; rebuild 0 |

The typed UNIMPLEMENTED refusals from the first pass are replaced by exit-0
real kernel output; negative-path witnesses ride in-band
(`corrupt_refused`/`tamper_refused`/`monotonic_id_refused` flags true in the
verb outputs themselves). VERDICT: ALIVE (handler-green).

Also updated: crates/cli/README.md verb-table statuses
UNIMPLEMENTED (CLI-2) -> ALIVE (handler-green, CLI-5 re-verify).
