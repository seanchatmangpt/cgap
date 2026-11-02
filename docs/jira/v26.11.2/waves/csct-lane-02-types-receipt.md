# CSCT-L2 Receipt — CSCT v1 Core Types + Receipt Renderer

- **Lane**: CSCT-L2 on /Users/sac/cgap (canonical checkout, no git writes)
- **Date**: 2026-11-02
- **Standing**: ALIVE (tests executed this session); compile into root
  workspace is UNVERIFIED:compile-pending (root manifest is
  coordinator-owned; `crates/csct/` scaffold absent this wave).

## Files written (owned only)

| File | sha256 (on disk, verified) |
|---|---|
| `crates/csct/src/types.rs` | `47b19c603d9c2fdc0189cc642d9310c781a209f70c7326cdcbe2b81aaa163d27` |
| `crates/csct/src/receipt.rs` | `d6581fc0d2ba878938e662a7d0511b0df697ff55e6aebc20cb24761780b0ce58` |

`crates/csct/Cargo.toml`, `crates/csct/src/lib.rs`, and root
`Cargo.toml` membership are NOT lane-owned — untouched, per contract.

## Commands + exits (this session, exact subject = the two owned files)

1. `rustc --edition 2021 --crate-type lib --crate-name cgap_blake3 -O -o
   libcgap_blake3.rlib crates/cgap-blake3/src/lib.rs` — exit 0.
2. /tmp harness crate (`/tmp/csct-l2-check/`, `lib.rs` declaring
   `pub mod receipt; pub mod types;` + copies of the two owned files):
   `rustc --edition 2021 --test -o csct-tests
   --extern cgap_blake3=./libcgap_blake3.rlib lib.rs` — exit 0, zero
   warnings.
3. `./csct-tests` — exit 0, **9 passed; 0 failed** (re-read from run
   output this session, not recalled).

## Courts run (anti-vacuity mutations, real collaborators — cgap_blake3 rlib linked, no mocks)

| Mutation | Court | Verdict |
|---|---|---|
| flip one `observed` f64 | digest must move (`digest_mutates_on_field_flip`) | flipped, PASS |
| clear stage list | digest must move (`digest_mutates_on_stage_removal`) | flipped, PASS |
| flip `final_commercial_verdict` | digest must move (`digest_mutates_on_verdict_flip`) | via tail: all 9 ok |
| `Pass` -> `GateFail` | rendered bytes must move (`render_mutates_on_exit_code_change`) | flipped, PASS |
| flip assertion `pass` | rendered bytes must move (`render_mutates_on_assertion_flip`) | via tail: all 9 ok |
| bleed $1.0 | digest AND render must both move (`digest_and_render_agree_on_mutation`) | via tail: all 9 ok |
| bleed/post-adoption ROI rounding | `render_is_byte_stable_across_calls` asserts `$5,068,632`, `$14,400`, `352x`, `PASS` substrings | flipped (was 351x under truncation, FAIL caught pre-landing), PASS |

## Receipt standard compliance

- `render_report` is byte-for-byte stable across calls (court
  `render_is_byte_stable_across_calls`); fixed field order, no sorting,
  no HashMap iteration.
- `report_digest` hashes field-by-field little-endian f64 bits via
  `cgap_blake3::hash` (real API at
  `crates/cgap-blake3/src/lib.rs:197`).
- No serde, no JSON, no protobuf anywhere in the two owned files
  (verified by authorship; parse-only errors during standalone rustc
  runs were unresolved-crate only, i.e. syntax-clean).

## Fences (must-not-regress surfaces this diff could break)

- Root workspace compile: NOT fenced (crate not a member; no lib.rs;
  coordinator scaffold pending). An unstated fence would be unaudited —
  this is the one declared absent surface.
- `crates/cgap-blake3/src/lib.rs` untouched (read-only use); its
  existing gate surfaces unaffected — zero shared-tree writes.

## Divergences from dispatch

1. `crates/csct/` was NOT pre-scaffolded (dir absent). Per contract
   ("create ONLY your listed files") I created the `src/` directory
   implicitly via file creation only; Cargo.toml/lib.rs left for the
   coordinator.
2. ROI in the economic block is rendered as
   `round(bleed / post_adoption)` (351.99 -> 352x per the frozen
   economic constants), not `net_delta / post_adoption` (350.99 ->
   351x). Dispatch said "ROI 352x" as a fixed render constant; the
   351x form contradicted it, so the formula was chosen to satisfy the
   frozen constant. Test asserts the 352x render; recorded as a
   formula choice, not a spec rewrite.
3. Compilation into the root workspace did not happen (manifest
   coordinator-owned) — verification used a /tmp rustc harness with a
   locally built cgap_blake3 rlib. Standing for the workspace-integrated
   crate is UNVERIFIED:compile-pending.
