# F3-03 WORKSPACE-GREEN — gate-runner receipt (BLOCKED)

- **Lane**: Wave F3 item 3 (gate-runner), repo `/Users/sac/cgap`, 2026-11-01.
- **Lease**: this marker only. No git writes, no source edits, no repairs.
- **Verdict**: **BLOCKED — gate RED. Publishers must NOT proceed past F3-03.**
- Precondition poll: bounded 20s-cadence poll for `f3-01-version-lock.md`,
  FOUND at poll #16 (~5m20s, 16:24:25). All legs ran on the post-bump tree.

## Exits (this session, all commands witnessed)

| leg | command | exit |
|---|---|---|
| 1 | bounded poll for f3-01-version-lock.md | found 16:24:25 |
| 2 | `CARGO_TARGET_DIR=/tmp/f3-3-target cargo build --workspace` | 0 |
| 3 | `CARGO_TARGET_DIR=/tmp/f3-3-target cargo test --workspace` | 101 |
| 4 | isolated rerun `cargo test -p cgap-court --test court_verdict` | 0 (13/13) |
| 5 | rerun `cargo test --workspace` (quiesced tree) | 101 |
| 6 | isolated rerun `cargo test -p cgap-ffi --lib sovereign::xip_courts -- --test-threads=2` | 101 |
| 7 | `cargo test --workspace --no-fail-fast` (full census) | 101 |

## Counts (leg 7, complete census: 122 suites executed)

- 1016 passed; 1 failed; 3 ignored; 121/122 suites ok.
- Sole failure: `cgap-ffi --lib`
  `sovereign::xip_courts::xip_t1_riscv_staticlib_size_court`
  panic at `crates/cgap-ffi/src/sovereign.rs:2399`:
  `T1 FLASH 264881 B >= budget 65536 B — exit gate §4 NOT met (measured, not estimated)`
  Deterministic: identical panic on the full run, and on an isolated rerun
  with `--test-threads=2` (3 passed / 1 failed, 0.92s). 4.04x over budget.
- Build receipt real: 24 `Compiling` lines; five publishable crates all
  v26.11.1; fresh target dir (195M, created 16:24).

## Race disclosure (leg 3 vs leg 4)

Leg 3 red (10/13 failed in `cgap-court court_verdict`, incl. FIPS 180-4
empty-input sha256 + §5.1 corpus pin `2142a6c3… != 03ed29d2…`) was a
mid-edit race: lane f3-04 was concurrently editing `tests/court_verdict.rs`
(mtime 16:29). Leg 4 isolated rerun on the settled file: 13/13 pass, exit 0.
Leg 5/7 confirm court_verdict green. Not counted as a gate defect; named
per compile-freeze SLA disclosure rules. `cargo test` fail-fast truncated
leg 3 and leg 5 at the first failing suite; leg 7 closed the census.

## Coordinates for the repair lane (not executed by this lane)

- Subject: `/Users/sac/cgap/crates/cgap-ffi/src/sovereign.rs:2399`
  (assert `flash < FLASH_BUDGET`, budget 65,536 B; measured 264,881 B).
- Pre-existing relative to this session (sovereign.rs mtime 15:58:57;
  this lane edited nothing). Court is documented in-source as a hard gate
  ("no sanctioned skip class remains — the size gate is hard").

## Cleanup

- `CARGO_TARGET_DIR=/tmp/f3-3-target` deleted via
  `python3 -c "import shutil; shutil.rmtree(...)"` post-receipt.

## Re-gate (2026-11-01, lane R1-regate — post GAP-1)

- Precondition: bounded 30s-cadence poll for `gap-1-sovereign.md`, FOUND at
  poll #24 (17:30:14). Receipt standing read on disk: all deliverable rows
  ALIVE; final `cargo test -p cgap-ffi --no-fail-fast` exit 0. The prior
  BLOCKED verdict above is preserved unchanged; this section is the current
  gate state.
- Verdict: **GREEN — workspace re-gate passed. Publishers may proceed past
  F3-03.**

| leg | command | exit |
|---|---|---|
| 1 | bounded poll for gap-1-sovereign.md | found 17:30:14 |
| 2 | `CARGO_TARGET_DIR=/tmp/r1-target cargo build --workspace` | 0 |
| 3 | `CARGO_TARGET_DIR=/tmp/r1-target cargo test --workspace --no-fail-fast` | 0 |

- Counts (leg 3, full census, 121 `test result:` lines): **1025 passed /
  0 failed / 3 ignored**; 0 panics. Prior sole defect resolved:
  `xip_t1_riscv_staticlib_size_court` no longer exists; successor
  `xip_t1_riscv_elf_size_court ... ok` (subject re-anchored to the §4 ELF,
  budgets unchanged — gap-1-sovereign.md "T1 repin").
- court_verdict: 13 passed / 0 failed (13.65s).
- No red suites -> no isolated rerun required.
- Counting-method note: prior census reported "122 suites executed"; this
  census counts `test result:` lines (121). Method stated so the delta is
  attributable, not silently absorbed.
- Tree disclosure: 73 untracked paths = this campaign's lane outputs (60
  under docs/jira, 12 lane-authored code/ledger/scripts, 1
  `.clap-noun-verb/` telemetry dir); 1 tracked deletion
  (crates/cgap-blake3/tests/bench_tmp.rs, lane cleanup). Formal receipt:
  `gate-1-workspace.md`.
- Cleanup: `/tmp/r1-target` deleted via `python3 shutil.rmtree` post-receipt.
