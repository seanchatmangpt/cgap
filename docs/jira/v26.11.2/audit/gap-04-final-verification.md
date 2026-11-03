# GAP-04 — Final workspace build & gate battery verification (terminal sweep)

Date: 2026-11-02. Repo: /Users/sac/cgap (canonical, main). Read-only
verification lane; no git commands; no repairs applied.

## W1 Subject + integrity pre-check

- `crates/cgap-ffi/src/lib.rs` read end-to-end (364 lines): CLEAN. The
  witnessed corruption ("expected `{`, found `lease`") is NOT present
  at lane start — all braces balanced, no `lease` token. Either
  repaired by its owner before this lane, or witnessed in an earlier
  tree state. Not a finding against the current tree.
- `python3 -m py_compile` over boundary_gate.py, zero_string_gate.py,
  verify_cscb_sram.py, gate_battery.py, receipt_lint.py,
  run_checked.py -> **exit 0** (all six parse).

## W2 Workspace build (with in-flight churn)

- Poll 1: `run_checked.py -- cargo build --workspace` ->
  **exit 101**: 3x `mismatched closing delimiter: ]` at
  `crates/cgap-git/src/ref_store.rs:842/847/853` — mid-write snapshot
  by R-ADV11FIX (file mtime 19s before check; the attrs read balanced
  on disk immediately after). Typed: compile-abort, in-flight churn,
  owner R-ADV11FIX.
- Bounded re-poll (the one allowed): **exit 0**, "Finished `dev`
  profile ... in 2.50s". Log /tmp/gap04-build2.log.

## W3 Gate battery

- `python3 scripts/gate_battery.py` -> **exit 0**, **4/4 PASS**
  (boundary E=0 externals; zero-string one loud rmeta [SKIP];
  cscb-sram report-only REFUSED-as-designed; ffi-surface 12==12).
- `--with-self-check` -> **exit 0**, **8/8 PASS**; all four
  self-check arms flipped/held as designed (boundary both-arms,
  cscb bn3-pin mutant refused exit 1, ffi byte-exact re-restore).
  Matches R-05 rollup exactly. (First attempt at the combined
  battery command died on a zsh `===` parse error — CLAUDE.md-warned
  glob bug; re-run split, disclosed.)

## W4 Test fences (cargo test, CARGO_TARGET_DIR=/tmp/gap04-target)

| crate | exit | result |
|---|---|---|
| cgap-bn3 | 0 | 139 passed / 0 failed (2 ignored target-only) |
| cgap-storage | 0 | 51 passed / 0 failed |
| cgap-p28 | 0 | 50 passed / 0 failed |
| cgap-git | 0 | all green; mid-log `mutation_conflict_injection_flip
  FAILED` is the DESIGNED flip witness (octopus_merge_court.rs:295
  re-exec child panics exit 101; parent asserts Some(101), passes) |
| cgap-ffi | **101** | 66 passed / **1 failed**: `fa9_rename_race_denied`
  panics at `crates/cgap-ffi/tests/fa9_commit_gating.rs:464`
  (`fs::rename` mid-lease -> expect fail). Reproduced on a bounded
  re-run of just that binary (1 failed / 6 passed). `sovereign.rs`
  mtime 3s before check — R-ADV10FIX actively mid-write. Typed:
  real failure on an in-flight surface, owner R-ADV10FIX; NOT fixed
  (outside ownership). |

## W5 Warning census vs the "zero warnings" bar

**15 concrete warnings — the bar is NOT met.** Honest count from
`grep -c '^warning'` minus the two summary lines = 15 (12 cgap-git
lib + 3 bench manifest).

Pre-existing (14; match lane-r01 W4 exactly):
- mcp_driver.rs:36 `#![no_std]` non-crate-root (unused_attributes)
- ref_store.rs deprecated `fetch_update` + unused `bound` (line
  numbers drifted 214/292 -> 224/267/345 from R-ADV11FIX edits; same
  warning classes)
- pkt_line.rs 7 dead_code (164/186/212/215/221/227/233)
- bench Cargo.toml:20/21/22 unused deps cgap-ffi/cgap-git/cgap-storage

Wave-introduced (1; NOT in R-01's list):
- `unexpected cfg condition name: implcell_critical_provided` at
  pkt_line.rs:27:11 — appeared with this wave's width-gated atomic
  work (cgap-git). Typed loudly; not repaired (outside ownership).

## W6 Standing

**PARTIAL.** Workspace build exit 0, battery 4/4 and 8/8 exit 0,
bn3/storage/p28/git test fences exit 0 — all commands+exits this
session. The single defect is `fa9_rename_race_denied` (exit 101) on
the R-ADV10FIX in-flight surface; it blocks ALIVE and blocks the
"zero warnings" bar independently (15 warnings).

## W7 Fences + divergences

- Fences audited: R-01 build gate (held on re-poll), R-05 4/4+8/8
  battery (held, byte-for-byte same rollups), octopus flip court
  (held), ffi digest courts (held, in the 52-test green binary).
- Divergences: (1) dispatch's "expected 4/4 and 8/8 if settled" held
  even amid churn; (2) lib.rs corruption did not reproduce; (3) fa9
  failure is new vs R-01 (R-01 ran build only, no ffi tests) — no
  prior receipt claimed fa9 green this wave, so no errata owed.
- No git commands run. Build lease /tmp/gap04-target left for
  coordinator janitor (`python3 scripts/lease_janitor.py --clean` or
  `just leases` reclaims).

JSON terminus: {"lane":"GAP-04","files_written":["docs/jira/v26.11.2/audit/gap-04-final-verification.md"],"commands":["py_compile 6 gate scripts -> 0","cargo build --workspace -> 101 (mid-write R-ADV11FIX) then re-poll -> 0 (15 warnings)","gate_battery.py -> 0 (4/4)","gate_battery.py --with-self-check -> 0 (8/8)","cargo test -p cgap-bn3 -> 0 (139)","cargo test -p cgap-storage -> 0 (51)","cargo test -p cgap-p28 -> 0 (50)","cargo test -p cgap-git -> 0 (flip court by design)","cargo test -p cgap-ffi -> 101 (fa9_rename_race_denied, reproduced on re-run)"],"courts_run":["boundary self-check (flipped)","cscb-sram bn3-pin mutate (flipped, refused exit 1)","ffi-surface byte-exact re-restore (held)","octopus mutation_conflict_injection_flip (held: child 101 asserted by parent)"],"standing":"PARTIAL","divergences":["lib.rs corruption not present at lane start","fa9_rename_race_denied fails on in-flight R-ADV10FIX surface (fa9_commit_gating.rs:464), not fixed (read-only lane)","15 warnings vs zero bar: 14 pre-existing + 1 wave-introduced pkt_line.rs:27 implcell cfg"]}


## ERRATA (2026-11-02, lane WARN-01; appended, original W5 untouched)

1. **Warning site misattributed.** W5's "wave-introduced" warning
   `unexpected cfg condition name: implcell_critical_provided` is at
   **`crates/cgap-git/src/ref_store.rs:230:13`** (predicate arm
   `not(implcell_critical_provided)`; second use at :224), NOT
   `pkt_line.rs:27:11` — pkt_line.rs:27 is a doc comment above
   `PKT_BUF_LEN`. rustc's own `-->` line names ref_store.rs. The
   14-pre-existing / 1-wave-introduced split stands; only the site was
   wrong.
2. **Fix landed (lane WARN-01):** `[lints.rust]
   unexpected_cfgs = { level = "warn", check-cfg =
   ['cfg(implcell_critical_provided)'] }` added to
   `crates/cgap-git/Cargo.toml` per rustc's printed suggestion. Post-fix
   build this session: the unexpected_cfgs warning is GONE; cargo-git
   lib warning delta **12 -> 11** (this session's re-measure; W5's
   "15" counted differently — 12 lib + 3 bench manifest — and
   included the cfg warning). No new warnings introduced. Fence:
   `cargo test -p cgap-git --test ref_store_width_arms_court` -> exit 0,
   8/8 (cfg mechanism untouched).
3. **fa9RenameRace supersession.** W4's cgap-ffi 101 on
   `fa9_rename_race_denied` (in-flight R-ADV10FIX surface) is
   superseded by
   `docs/jira/v26.11.2/lanes/lane-r-adv10fix-spill-gate-rename-toctou.md`
   W3: post-fix `cargo test -p cgap-ffi` full suite exit 0, **83 tests,
   0 failed** (its table line "52+4+4+4+7+2+5+5+0 = 83"). GAP-04's
   exit-101 observation was the in-flight state, not the settled one.
