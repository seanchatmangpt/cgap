# Lane G05 — LSP Wire Rebuild Receipt

Date: 2026-11-02. Repo: /Users/sac/lsp-max (branch main). Lane wrote only
build outputs + this receipt; no git commands; `src/rule_pack.rs` and the
python client untouched.

## 1. Rebuild

| command | exit |
|---|---|
| `cargo build --release -p branchless-lsp` | 0 (14.26s) |
| `cargo build -p branchless-lsp` | 0 (4.63s) |

Fresh binaries (Nov 2 12:53 mtimes):
- `target/release/branchless-lsp` — `strings | grep -c ANDON_REFUSED` = 3
- `target/debug/branchless-lsp` — count = 3

## 2. Live wire witness (branchy vs clean flip pair)

Probes (rule_pack.rs:368/370 fixtures):
BRANCHY `rf_branchy` (forward `while` branch), CLEAN `rf_clean`
(straight-line, black_box-wrapped).

Harness: /tmp/g05_drain.py using `BranchlessLspClient` against the rebuilt
RELEASE binary. Note: `analyze_text` returns only the FIRST
publishDiagnostics for a URI (branchless_client.py:263-272) — the did_open
tier-1 publish (empty) races and shadows the did_save tier-2 publish. The
probe drains ALL publishes instead. Known quirk preserved: the client sends
stderr to DEVNULL (branchless_client.py:217) — not fixed (not a lane file).

Observed (release binary, one session per fixture):

- BRANCHY: 2 publishes for the URI; publish[0] n=0 (tier-1); publish[1]
  n=3 — two `ANDON_REFUSED: fn rf_branchy carries a forward conditional
  branch` diagnostics, severity 1 (ERROR), line 3 (0-based, matching the
  `while` branch site); plus 1 `blocking diagnostic emitted without a
  corresponding ANDON push event` diagnostic.
  `conformance.refused = [{"custom":"ANDON_REFUSED"}]`.
- CLEAN: publish[0] n=0; publish[1] n=0. `conformance.refused = []`.

Flip pair witnessed: branchy NON-EMPTY with ANDON_REFUSED ERROR diagnostics,
clean EMPTY. Both binaries rebuilt from current source (rule_pack.rs mtime
12:27 predates binary mtime 12:53).

## 3. Classification classes (NoFix vs const-table)

Witnessed via the crate's unit courts (the dispatch allows "wire or unit
courts"; the python client exposes no codeAction request method, so the wire
witness of code actions is UNVERIFIED, not failed).

`cargo test -p branchless-lsp --lib` → exit 0, **72 passed, 0 failed**,
including:
- `match_jump_table_edit_and_items`: flat enum `Op` (discriminants 0..=2)
  → `OP_JUMP_TABLE: [Op; 3] = [10, 20, 30]` const-table + `get_unchecked(__idx)`
  rewrite with flat-enum note.
- `nonflat_enum_gets_documented_refusal`: non-contiguous discriminants →
  `NoFix::NonFlatEnum` with "not contiguous" reason.

Anti-vacuity corroboration: `cargo test -p branchless-lsp --test
riscv32imc_court` → exit 0, 3 passed, including
`riscv32imc_opcode_mutation_flips_the_court` (real opcode mutation flips).

## 4. Standing

- Rebuild: ALIVE (commands + exits above, run this session).
- Wire diagnostics (branchy refused / clean empty): ALIVE.
- NoFix-vs-const-table classification: ALIVE via unit courts (72/72).
- Wire codeAction: UNVERIFIED (client lacks the request method).

## 5. Divergences

1. Dispatch cited `src/rule_pack.rs`; actual source is
   `crates/branchless-lsp/src/rule_pack.rs` (rule_pack_server.rs at
   src/ is a different module). Line numbers matched (368/236/303).
2. `analyze_text` first-publish race documented above; the wire was never
   broken after rebuild — the harness's wait_for_diagnostics returns the
   tier-1 (empty) publish. Server behavior correct on both fixtures.
3. The BRANCHY tier-2 publish carries a third diagnostic ("blocking
   diagnostic ... without a corresponding ANDON push event") beyond the two
   ANDON_REFUSED entries — recorded as observed; classification is a WG2
   surface, not judged here.
