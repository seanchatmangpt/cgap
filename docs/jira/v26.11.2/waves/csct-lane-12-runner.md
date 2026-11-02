# CSCT Lane 12 Receipt — runner binary

Lane: CSCT-L12 · date: 2026-11-02 · branch: main ·
file: `crates/csct/src/bin/csct_runner.rs` (153 lines, rustfmt-clean)

## What landed

- `crates/csct/src/bin/csct_runner.rs`: hand arg-parse (std::env::args,
  no clap), `Action` enum (Run/Mutate/Receipt/Unknown), `classify()`,
  `exit_for()` mapping, `main()` consuming `csct::run_suite`,
  `csct::mutation::run_mutation_controls`, `csct::receipt::render_report`,
  `csct::types::ExitCode`. No JSON, no serde; stdout is the markdown
  receipt. 7 `#[cfg(test)]` unit tests over `classify()` shapes only.
- Exit mapping: unknown subject -> 2 + printed `[SKIP]` line;
  mutation Err -> 1 + printed `[VACUOUS]` line; first non-Pass stage ->
  that stage's exit as process exit (Pass=0, GateFail=1,
  SubjectMissing=2, LoudSkip=3, AndonRefusal=101); all-Pass -> verdict
  ? 0 : 1.

## Commands + exits (this session)

Build/test executed against a /tmp sandbox copy
(`/tmp/csct-l12-sandbox`, shared tree untouched; sandbox Cargo.toml
path-patched to absolute `cgap-blake3`, andon.rs/court.rs stubbed,
warmer.rs `}`-completed, solution.rs `hash(&bytes)` fixed — sandbox-only
edits, NOT shared-tree claims):

- `rustfmt --check --edition 2021 crates/csct/src/bin/csct_runner.rs`
  (shared tree) -> 3 cosmetic diffs; `rustfmt --edition 2021` applied
  -> exit 0.
- `cd /tmp/csct-l12-sandbox && cargo test --bin csct_runner`
  -> **exit 0, 7 passed; 0 failed**.
- `./target/debug/csct_runner --suite=nope` -> exit 2, prints
  `[SKIP] ... --suite=nope ...` line.
- `./target/debug/csct_runner --receipt` -> exit 3 (stage 6
  ingress_filter 75ns > 10ns threshold under debug build).
- `./target/release/csct_runner --receipt` -> all 6 stages Pass,
  `final_commercial_verdict | FAIL`, exit 1 (ROI 351 < 352 threshold —
  lib economics, not the runner).
- `./target/release/csct_runner` (default) -> exit 1.
- `./target/release/csct_runner --mutate=zero-tax` -> exit 1
  (mutation controls ran first, then suite).
- AndonRefusal=101 path: verified-by-inspection only — no in-tree stage
  currently emits it; no sandbox mutation was constructed to flip it.
  Marked UNVERIFIED:andon-101-path.

## Standing

- Runner bin: ALIVE (built + exercised this session, sandbox subject).
- Unit tests: ALIVE (7/7, command above).
- AndonRefusal=101 exit path: UNVERIFIED (inspection only).
- Process-exit mapping on the SHARED tree: UNVERIFIED:compile-pending —
  `cargo build` in `crates/csct` fails today on files I do not own:
  `src/lib.rs:17` E0583 (court.rs missing), `src/warmer.rs:147` E0583-
  adjacent unclosed delimiter (truncated file), `src/solution.rs:134-135`
  E0308 (`hash(bytes)` vs `&[u8]`). Owner lanes must land before
  shared-tree build; my file compiles once they do (sandbox-proven).
- `--mutate=zero-tax` stage ordering observed: controls ran before
  suite in both debug and release runs.

## Divergences (doc-yields-reality)

1. Contract said "consume ... andon" — `csct::andon` module is declared
   in lib.rs but andon.rs does not exist in the shared tree; my bin
   consumes `types::ExitCode::AndonRefusal` for the 101 mapping and
   does not link the andon module directly.
2. Contract's "Default: run_suite(); if final_commercial_verdict ->
   exit 0" — in the sandbox release run the verdict is FAIL (ROI 351x
   < REQUIRED_ROI_X 352x with SUT_ANNUAL_USD=14,400). That is lib
   economics owned by other lanes; the runner's mapping
   (verdict=false -> 1) behaved exactly as specified.
3. Shared-tree `cargo build` is blocked by three non-owned csct files
   (court.rs, warmer.rs, solution.rs) — compile-freeze SLA applies to
   their owners; I did not touch them in the shared tree. (Sandbox
   fixes were disclosure-only, to obtain evidence for MY file.)
4. `csct` is a standalone crate (own `[workspace]`, like crates/cli) —
   root `cargo build -p csct` does not see it; build must be
   `cd crates/csct && cargo build`.

## Fences

Must-not-regress surfaces this diff could break: the csct lib unit
tests and `tests/csct_pipeline_court.rs` (currently corrupted by
another lane — `5_068_6 STATUS_QUO_ANNUAL_USD_PLACEHOLDER` token error
at line 47). My diff adds a new bin target only; it cannot regress any
existing passing gate. Bin tests run via `cargo test --bin
csct_runner`; the corrupted integration court blocks full `cargo test`
on the csct crate until its owner repairs it.
