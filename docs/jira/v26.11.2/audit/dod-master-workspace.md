# DoD master workspace audit — EXP-10 (2026-11-02)

Workspace member count: 23 (Cargo.toml members list, tomllib-parsed;
`cargo metadata --no-deps` workspace_members = 23). G20's 23 is CONFIRMED;
CLAUDE.md line 3 ("22-crate root") and line 56 ("22-member cargo metadata
list") are STALE. Also on disk but not members: crates/cgap-verification,
crates/csct, crates/wasm-footprint.

## 1. boundary_gate.py

- `--deps`: exit 0. "DEPS GATE: E=0 runtime externals across 15 covered
  crates (§6.4)". All 15 rows E=0.
- `--strings`: exit 0. 0 hits across 15 covered crates; A07 register v2
  consumed (sha 7778078dfe11, 6 canonical patterns, 0 allowlist entries);
  cross-gate artifact scan GREEN (scanned=92, hits=0, emitter_hits=2).
- COVERED_CRATES (scripts/boundary_gate.py:865-881, tuple of 15):
  cgap-core, cgap-storage, cgap-bont, cgap-bosym, cgap-bn3, cgap-gate,
  cgap-crypto, cgap-p28, cgap-court, audit, cgap-arrow, cgap-blake3,
  cgap-git, cgap-parquet, ontologies/fibo.
- csct: crates/csct exists on disk (Cargo.toml + src) but is NOT a
  workspace member, and boundary_gate.py never mentions it. G16's
  exclusion finding is consistent with current topology. Adding csct to
  COVERED_CRATES is NOT warranted while csct stays outside the
  workspace; if it is promoted, the tuple must gain it (the gate is
  fail-closed on missing rows, so this is discoverable).

## 2. verify_ost_design_around.py

Exit 0. G7-PATENT: PASS 4/4 DISTINCT-MECHANISM + behavioral court.
Rows (expect 3, saw 4):
- US10,817,467 -> DISTINCT-MECHANISM
- US11,216,456 -> DISTINCT-MECHANISM
- US11,409,698 -> DISTINCT-MECHANISM (src hits: 1)
- US11,645,278 -> DISTINCT-MECHANISM (src hits: 1)
Behavioral court compiled and ran (patent_behavioral_court, exit 0).

## 3. Packaging viability (FREEZE active)

Freeze citations: docs/jira/v26.11.2/waves/publish-halt.md (operator
order 2026-11-02, "do not publish anything yet"; 8 crates landed, 12
frozen); .claude/hooks/publish-guard.sh blocks cargo/npm publish
unconditionally (FROZEN-BY-OPERATOR). No publish attempted.

`cargo package --dry-run` does not exist in this toolchain (error:
"unexpected argument '--dry-run'"). Full test would be `cargo publish --dry-run`
(frozen); used `cargo package --no-verify --allow-dirty`
(CARGO_TARGET_DIR=/tmp/exp10-target) on 5 representative crates:

| crate | exit | notes |
|---|---|---|
| cgap-core | 0 | WARNING: no license or license-file in manifest (comment-only header) |
| cgap-git | 0 | clean, 57 files |
| cgap-parquet | 0 | clean, 43 files |
| cgap-arrow | 0 | clean, 44 files |
| cgap-bn3 | 0 | clean, 17 files |

Dirty-workspace note: cargo package refuses on dirty trees; --allow-dirty
was required (repo heavily dirty, branch main). On a frozen lane this is
procedural, not a manifest defect.

## 4. cargo build --workspace --all-targets

CARGO_EXIT=101 — HARD FAIL. Two bench compile errors (blockers):

- crates/bench/benches/chains-parquet.rs:479:33 — E0425 cannot find value
  `block_len`; `_block_len` defined at :467 (underscore rename).
- crates/bench/benches/chains-seal.rs:493:48 — E0596 `&mut lease` but
  `lease` not declared mut (bound at :480).

Warning census: the expected "~6 bench-manifest warnings post-C05" is NOT
what ships. Observed:

- crates/bench/benches/chains-seal.rs:478:9 — unused mut `prev_root`
  (the only bench warning).
- cgap-git test targets (dead_code heavy): patent_court_git.rs:119:9
  (unused mut); object_store.rs:182:12 (`is_empty` never used, included
  via tests/../src path); per-test summary counts: delta_resolver_court
  4, ref_store_court 3, mcp_driver_court 1, pkt_line_court 3,
  commit_synth_court 3, soc_full_chip_court 46, patent_court_git 1,
  object_store_court 1, admission_gate_court 11 — ~73 cgap-git test
  warnings total.
- ZERO cargo-level bench [[bench]] manifest warnings fired — the C05
  cleanup appears landed; what remains is dead-code noise in cgap-git
  courts, not manifest warnings.

Caveat: census taken incrementally; cached targets do not re-emit. The
two bench errors dominate — fix those first, then re-census on a clean
CARGO_TARGET_DIR.

## Verdicts

- boundary deps gate: PASS (exit 0, 15/15 E=0)
- boundary strings gate: PASS (exit 0, 0 hits)
- OST design-around: PASS (exit 0, 4/4 DISTINCT-MECHANISM)
- packaging (5-crate sample): PASS with 1 finding — cgap-core lacks a
  `license` key (comment-only license header)
- workspace build: FAIL (exit 101, 2 bench errors) — blocker for any
  all-targets DoD claim
- member count: 23, CLAUDE.md stale (says 22) — update CLAUDE.md
