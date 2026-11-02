# Lane DX-8 — Per-Target Test Census (scripts/test_census.py)

Wave: v26.11.2 starDF · Standing: ALIVE (all commands below run this session)

## Files written (lane-owned only)

- `scripts/test_census.py` (NEW, 182 lines) — census tool
- `justfile` — `census:` recipe appended at the very END (last recipe in file)
- this receipt

No other files touched; no git commands run.

## What the tool does

Enumerates test targets via `cargo metadata --no-deps` (explicit
`[[test]]` targets via `--test <name>`, lib unit tests via `--lib`),
runs each target individually under a per-lane
`CARGO_TARGET_DIR=/tmp/dx8-target` lease, and classifies:

- **COMPILE-ABORT** — nonzero rc, no `test result:` line; names the
  broken `file:line` from `-->` error locations + first `error[..]:`.
  Prints `LOUD-PENDING[COMPILE-ABORT]`, exits 0 unless `--strict`.
- **PASS** — n passed / i ignored, exit 0.
- **FAIL** — n failed; verbatim failure blocks captured and printed;
  exit 1.
- **TIMEOUT** — bounded per-target deadline (default 900s, `--timeout`);
  counts as not-passed (exit 1).

Flags: `--crate` comma list, `--timeout`, `--strict`, `--output <file>`
(writes markdown table), `--root` (workspace root override; the courts
exercise this against /tmp sandboxes).

## Anti-vacuity courts (/tmp sandbox copies, real cargo runs)

Three minimal standalone crates, deliberately mutated:

| court | mutation | census verdict | exit |
|---|---|---|---|
| /tmp/dx8-court-pass | `assert_eq!(id(1),1)` | `PASS 1 passed` | 0 |
| /tmp/dx8-court-fail | `assert_eq!(1,2, "court mutation: deliberate failing assert")` | `FAIL 0 passed; 1 failed` + verbatim panic block captured | 1 |
| /tmp/dx8-court-abort | unclosed delimiter `fn broken() {` in tests/t.rs | `LOUD-PENDING[COMPILE-ABORT] >tests/t.rs:1 error: this file contains an unclosed delimiter` | 0; `--strict` -> 1 |

All three flip: PASS stays PASS, FAIL exits 1, COMPILE-ABORT names the
file and only fails under `--strict`. A cannot-flip census would be FAIL;
none observed.

## Real census run

Command: `CARGO_TARGET_DIR=/tmp/dx8-target python3 scripts/test_census.py
--crate cgap-git,cgap-parquet --timeout 600 --output /tmp/dx8-census.md`
(exit 1 — one real FAIL, correctly surfaced).

Verbatim per-target table is in `/tmp/dx8-census.md`; summary:

- **cgap-git** — 28 targets: 27 PASS, 1 FAIL
  (`octopus_merge_court`: 5 passed, 1 failed —
  `mutation_conflict_injection_flip` panics at
  `crates/cgap-git/src/octopus_merge.rs:356:19` with
  `ANDON domain clash ... heads 0 and 2 conflict; refusing the merge`.
  Recorded as it IS; likely an in-flight lane's mutation court. Not
  touched by DX-8.)
- **soc_full_chip_court**: PASS (1 passed) — green now, witnessed.
- **cgap-parquet** — 14 targets: 14 PASS (incl. `mutation_kills` 17,
  `rle_court` 8).
- Totals: **42 targets | PASS 41 | FAIL 1 | COMPILE-ABORT 0 | TIMEOUT 0**.

## justfile recipe

```just
export CENSUS_ARGS := ''
census:
    python3 scripts/test_census.py {{CENSUS_ARGS}}
```

Usage: `just CENSUS_ARGS="--crate cgap-git,cgap-parquet" census`.
Witnessed: `just CENSUS_ARGS="--root /tmp/dx8-court-pass --crate
court-pass --timeout 120" census` -> exit 0; strict-abort variant -> 1.
Note: recipe-param form (`census ARGS=':'`) was tried first — this
just version leaks the `ARGS=` prefix into the interpolated command
(exit 2); the exported-variable form is the working one. Recipe is the
LAST recipe in the file (DX-1's `doctor-agents` precedes it).

## Commands + exits (this session)

| command | exit |
|---|---|
| `python3 ast.parse` on test_census.py | 0 |
| census court pass /tmp/dx8-court-pass | 0 |
| census court fail /tmp/dx8-court-fail | 1 (expected) |
| census court abort /tmp/dx8-court-abort | 0 (LOUD-PENDING) |
| census court abort --strict | 1 (expected) |
| real census cgap-git,cgap-parquet | 1 (1 real FAIL, octopus) |
| `just CENSUS_ARGS=... census` pass court | 0 |
| `just CENSUS_ARGS=... --strict` abort court | 1 |

## Fences

`justfile` grew one trailing recipe — no existing recipe edited. No
shared lib.rs/Cargo.toml touched, so no compile-surface fence is in
play. The `octopus_merge_court` FAIL is pre-existing on main and
belongs to the octopus lane, not DX-8; no fence asserted against it.

## Cleanup

`/tmp/dx8-court-{pass,fail,abort}` crates and the `/tmp/dx8-target`
build lease deleted (verified absent). `/tmp/dx8-census.md` and
`/tmp/dx8-court-pass.md` left as evidence tables.

## Standing

**ALIVE** — every claim above backed by a command + exit run this
session against the exact subject.
