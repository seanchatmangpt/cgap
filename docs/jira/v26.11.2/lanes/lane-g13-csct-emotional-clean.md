# Lane G13 — csct emotional.rs residual-duplication audit

Date: 2026-11-02. Subject: `crates/csct/src/emotional.rs`.
sha256 at audit: `3a226ef111caedc29ef780e247882e50338805bed32d10bac1caa7de55d228c4`
(209 lines; file unchanged by this lane — no edits made).

## Findings

- Duplicate-AST hunt: NONE. Single `impl Default`, one each of
  `compute`/`evaluate`/`emotional_impact`; no orphaned tail fragment;
  the L10 SLA unblock + L6 owner-byte repair left the file coherent.
- No-sleep law: `grep -c sleep` → 0. Time is a `*_ms` parameter,
  never awaited (file header states the law at line 9).

## Commands + exits (all run this session)

| Command | Exit | Evidence |
|---|---|---|
| `sha256sum crates/csct/src/emotional.rs` | 0 | hash above |
| `cargo test emotional` (CARGO_TARGET_DIR=/tmp/g13-target) | 0 | 3 passed; 0 failed; 34 filtered |
| `cargo build` warnings census (`grep -c warning`) | grep exit 1 → 0 warnings | integrated tree clean |
| `RUSTFLAGS="-D warnings" cargo build` on /tmp scratch copy | 0 | 0 warnings, compile not denied |

## Courts

Both emotional courts green under the emotional filter (3 tests = the
2 courts + the contract-values test):

- `default_params_witness_crash_exit_pass` — PASS
- `mutation_shrunk_tail_flips_evaluate_false` — PASS (named
  anti-vacuity mutation: shrunk tail gc 5 ms / transit 1 ms flips
  `evaluate` false; court flips, not vacuous)

## deny(warnings) audit method

Per dispatch, no crate-wide `deny(warnings)` was added (lib.rs not this
lane's file). Evidence ladder: (a) integrated-tree warnings census = 0;
(b) full `RUSTFLAGS="-D warnings"` build of a /tmp scratch copy of
`crates/csct` — BUILD_EXIT=0. Scratch needed a stub workspace root
(`/tmp/Cargo.toml` with workspace.package edition/version/repository/
authors, since `cgap-blake3` path-dep inherits those) + a `cgap-blake3`
symlink; both stubs deleted after. emotional.rs is warning-free under
deny(warnings), not just census-free.

## Standing

ALIVE — every claim above backed by a command+exit run this session
against the exact on-disk subject (hash pinned). File unchanged; no
mutation applied to the shared tree (courts are the tests' own named
in-model mutations, run in-process).

## Divergences

- Dispatch named `docs/jira/v26.11.2/lanes/`; the dir did not exist —
  created by this receipt (Write created parents). The research lane
  artifact `research/g13-doc-taxonomy.md` exists under that other
  g13 prefix; unrelated.
- Dispatch expected "both courts" = 2 tests; the filter matched 3
  (`default_params_match_contract` also carries `emotional`-adjacent
  filter... actually matched by the `emotional` string in module path).
  Counted honestly: 3 passed / 0 failed.
- No anti-vacuity mutation was applied to a /tmp sandbox copy of the
  FILE itself; the courts' named mutations are in-model parameter
  mutations (real collaborators, real `evaluate` calls). Recorded as
  method note, not a skip.
