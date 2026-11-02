# Gate-1 Workspace Receipt — Morning DoD (2026-11-01)

- **Lane**: R1-regate (gate-runner), repo `/Users/sac/cgap`, branch main
  (HEAD cfd085a9 at session start).
- **Lease**: append to `f3-03-workspace-green.md` (Re-gate section) + this
  file (NEW). No git writes, no source edits, no repairs.
- **Precondition**: `gap-1-sovereign.md` present, standing ALIVE (bounded
  30s poll, FOUND poll #24, 17:30:14).
- **Verdict**: **GREEN**

## Gate-1 definition + exits (this session)

| # | requirement | command | exit |
|---|---|---|---|
| 1 | workspace builds | `CARGO_TARGET_DIR=/tmp/r1-target cargo build --workspace` | 0 |
| 2 | all suites pass | `CARGO_TARGET_DIR=/tmp/r1-target cargo test --workspace --no-fail-fast` | 0 |

## Census (leg 2, complete, no-fail-fast)

- 121 `test result:` lines: **1025 passed / 0 failed / 3 ignored**; 0
  panics. Prior F3-03 census was 1016/1/3; delta +9 passed / -1 failed is
  the GAP-1 landing (three sovereign bodies + T1 repin courts in
  `cgap-ffi`).
- Sole prior defect resolved by replacement, not by skip:
  `xip_t1_riscv_staticlib_size_court` no longer exists; successor
  `xip_t1_riscv_elf_size_court ... ok` — subject re-anchored to the
  xip.ld-linked §4 ELF (llvm-size measured), budgets UNCHANGED 64 KiB
  flash / 32 KiB SRAM.
- `court_verdict` (cgap-court): 13 passed / 0 failed (13.65s).
- Build receipt: 24 `Compiling` lines; five publishable kernels all
  v26.11.1 (cgap-arrow, cgap-blake3, cgap-git, cgap-parquet, cgap
  umbrella); fresh target dir `/tmp/r1-target` (197M, created 17:30:41);
  repo `target/` untouched (mtime 08:32).

## Contamination note (grounded on `git status --porcelain`, this session)

- 73 untracked paths, 0 foreign. Class census: 60 under `docs/jira` (lane
  receipts/docs); 12 lane-authored campaign files (5 `crates/bench/benches`,
  `crates/cgap-core/tests/ingress_layout_court.rs`,
  `crates/cgap-p28/tests/`, `crates/cgap-ffi/src/oabi_lookup.rs`,
  `crates/cgap-git/src/pack.rs`, `crates/cgap-court/a20-kill-ledger.json`,
  `scripts/hop_ratio.py`, `scripts/projection_coverage.py`); 1
  `.clap-noun-verb/` telemetry dir. All within this campaign's lane
  ownership — the "receipts/docs only" expectation is refined: 61
  receipts/docs/ledger, 12 code/scripts, 0 unrelated content.
- 30 modified tracked files (campaign lane edits, build/test green on the
  modified tree); 1 tracked deletion
  (`crates/cgap-blake3/tests/bench_tmp.rs`, lane cleanup).

## Boundary (what this receipt does not claim)

Gate-1 = root-workspace build + full test census only. NOT in this
receipt: campaign gates (`boundary_gate.py`, `zero_string_gate.py`,
`just doctor`), the ggen replay, and `crates/cli` (standalone workspace,
not a root member — outside `--workspace`, out of this lease).

## Replay

Same two commands with `CARGO_TARGET_DIR=/tmp/r1-target` on this tree;
expect 0/0 exits and >= 1025 passed / 0 failed (>= because later lanes may
grow the census).
