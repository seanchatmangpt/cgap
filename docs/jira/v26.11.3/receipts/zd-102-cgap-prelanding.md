# ZD-102: cgap pre-landing terminal-state certification

- Date: 2026-11-04
- Subject: /Users/sac/cgap @ 95821317bb09419a71f34e93e2995ace49574fe1
- Verdict: **UNSAFE**
- Lane: verification lane (no edits, no git; this receipt is the only file written)

## Verification ladder

| # | Check | Command | Exit | Result |
|---|---|---|---|---|
| 1 | Working-tree census | `git status --porcelain` | 0 | Clean (0 entries; expectation ≤2 receipts met) |
| 2 | Compile gate | `cargo check --workspace` | 0 | PASS (2 pre-existing warnings in `cgap-ffi` bin `aoq_sidecar`, incl. unused `PAD_HI`) |
| 3 | Format gate | `cargo fmt --all -- --check` | **1** | **FAIL** — 1268 diff hunks across ~190 files |
| 4a | Advisory gate | `cargo deny check advisories` | 0 | PASS |
| 4b | License gate | `cargo deny check licenses` | 0 | PASS |
| 5 | Header gate | `make gate0` | 0 | PASS (`gate0: header compile PASS`) |

## FMT failure detail

`cargo fmt --all -- --check` exits 1 with 1268 hunks spanning 190+ files across
`crates/`, `tests/`, `ontology-packs/`, and `bench/`. Heaviest: `tests/court_verdict.rs`
(69), `crates/cgap-arrow/src/generated_layout.rs` (49), `crates/cgap-aoq/tests/cbmqs_runner.rs`
(49), `crates/audit/src/ocel2_slab.rs` (31). Full log: `/tmp/zd102_fmt.log`.

Diff classes observed: import reordering (`powl`/`ocel2_slab` module order), over-wide
single-line `write!` matches that rustfmt wants braced/multi-line, and long boolean
asserts rustfmt wants collapsed. This is repo-wide fmt drift, not a lane-local
regression — a `cargo fmt --all` (apply) would touch nearly every crate. Per lane
contract (no edits), the fmt fix is NOT applied here and blocks landing.

## Standing

ALIVE: checks 1, 2, 4a, 4b, 5 executed this session with real exits cited above.
UNSAFE verdict rests solely on check 3 (fmt), executed with exit 1 and hunk count
re-read from the on-disk log. Check 2's exit is the true process exit (re-run
without pipes after the initial tail-piped run masked exit codes).
