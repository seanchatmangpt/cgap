# Lane R-05 — gate battery 4/4 recount (post XC-01)

Date: 2026-11-02 · Branch: main · Read-only battery run; no script edits; no git commands.

## W1 Subject

`python3 scripts/gate_battery.py` rollup over the four house gates,
re-run after XC-01 landed the 12/12 fresh-dylib census. Supersedes the
3/4 mid-flight rollup in `lanes/lane-xc02-gate-battery.md:14-21` (that
FAIL was `cgap_stats_snapshot` prototyped-but-not-exported — since
resolved; not restated as errata, XC-02 already classified it as
in-flight cross-lane state).

## W2 Commands + exits (this session)

- `cargo build -p cgap-ffi` (CARGO_TARGET_DIR=/tmp/r05-target) -> **exit 0**
  (compile-error watch: green first pass, no re-poll needed)
- `python3 scripts/gate_battery.py` -> **exit 0**, 4/4 PASS:
  - boundary PASS exit=0 (E=0 runtime externals, 15 crates, §6.4)
  - zero-string PASS exit=0 (one loud [SKIP]: cgap-cloud rlib
    `members=(lib.rmeta)` — rustc metadata member, not an object)
  - cscb-sram PASS exit=0 (bn3 state-table charge 6520984 B, in
    ceiling: NO -> admission REFUSED as designed; report-only leg)
  - ffi-surface PASS exit=0 (12 exported cgap_* symbols == 12 header
    prototypes, 1:1)
- `python3 scripts/gate_battery.py --with-self-check` -> **exit 0**,
  8/8 PASS; per self-check leg:
  - boundary:self-check PASS ("both arms flipped", non-vacuous)
  - zero-string:self-check PASS (sandbox copy, same rmeta [SKIP])
  - cscb-sram:self-check PASS (MUTATE COURT: bn3-pin mutant refused
    exit 1 — NaMatches 1081352 B != pinned 2162696 B)
  - ffi-surface:self-check PASS (final re-restore byte-exact -> exit 0)

## W3 Courts run

- cscb-sram bn3-pin mutation court — FLIPPED (mutant refused, exit 1)
- boundary two-arm self-check — FLIPPED (clean=0, poison=1)
- ffi-surface byte-exact re-restore court — HELD (exit 0)
- No [SKIP] beyond the two documented rustc-rmeta metadata-member rows.

## W4 Fences

Read-only lane: no script, lib.rs, or Cargo.toml edits; the four gates
themselves are the regression surface and all ran green this session.
Build used an isolated lease root /tmp/r05-target (deleted at lane end).

## W5 Divergences

None. XC-02's ffi-surface FAIL did not reproduce; the mismatch row
capture path was not needed.

## W6 Standing

ALIVE — every rollup above carries a command + exit run this session.

## W7 JSON terminus

{"lane":"R-05","files_written":["docs/jira/v26.11.2/lanes/lane-r05-gate-battery-recount.md"],"commands":["cargo build -p cgap-ffi -> 0","python3 scripts/gate_battery.py -> 0 (4/4 PASS)","python3 scripts/gate_battery.py --with-self-check -> 0 (8/8 PASS)"],"courts_run":["cscb-sram bn3-pin mutate (flipped)","boundary two-arm self-check (flipped)","ffi-surface byte-exact re-restore (held)"],"standing":"ALIVE","divergences":[]}
