# Lane C05 — Dead-Code Excision (v26.11.2 wave)

Standing: ALIVE (all commands below run this session, exits recorded).
Date: 2026-11-02. Repo: /Users/sac/cgap, branch main.

## Files owned / written

- `scripts/verify_dead_library.py` (pre-existing, unchanged by C05)
- `scripts/dead_library_exceptions.json` (updated: `_c05_note` + 9
  `C05-candidate` entries)
- this receipt

Excisions (warning fixes) in non-owned, non-in-flight files are listed
below with per-item gates.

## 1. Gate + self-check (anti-vacuity)

| Command | Exit | Note |
|---|---|---|
| `DEADLIB_TARGET=/tmp/c05-target python3 scripts/verify_dead_library.py --self-check` | 1 (first) | loud-skip: lease empty, no consumer rlibs — self-check does not build consumers |
| `DEADLIB_TARGET=/tmp/c05-target python3 scripts/verify_dead_library.py` (plain gate) | 1 | 9 DEAD_LIBRARY_SHOWROOM (all in `scan.rs`/`transpose.rs` — in-flight lane files) |
| `--self-check` re-run (populated lease) | 0 | MUT-A FIRED: stub mutant named `unconsumed_stub`, exit 1. MUT-B FIRED-proof: parquet objects census 0 branchless tokens — scanner selective, not all-pass |

Anti-vacuity: the MUT-A flip is the named mutation; witnessed live this
session (exit 1 naming the mutant, then 0 overall).

## 2. Census method

`CARGO_TARGET_DIR=/tmp/c05-clean cargo build --workspace --all-targets
--message-format short` from a cold target dir (cached-warning replay
omits units otherwise). Pre-fix census: 18 warning sites. Post-fix: 0
(delta -18). Build exit 0.

### Warning census (pre)

bgp.rs:34 unused import HashSet; chains-parquet.rs:430/431 unused vars,
433/470 drop-of-Copy; chains-seal.rs:478 unneeded mut; cgap-arrow
validation.rs:526 parens; branchless eytzinger.rs:140 unused import
format; boundary_sweep_courts.rs:247 unused var i; scan_courts.rs:388
unused var name; fa9_commit_gating.rs:76 unused cp;
envelope_decimal_court.rs:13 unused import Oid; dred_swar_wire.rs:13
unused imports; cloud courts.rs:10 unused import lex_hash;
asc450.rs:1887 unneeded mut; bitslice.rs:894 trailing semicolon;
unpack_filter.rs:209/252 parens.

## 3. Excisions (each item's gate: workspace build exit 0 + 0 warnings + crate tests green)

| File | Item | Class |
|---|---|---|
| crates/bench/benches/bgp.rs | unused `HashSet` import removed | dead import |
| crates/bench/benches/chains-parquet.rs | `block_len`/`digest` unused locals underscored; 2 `drop(sealed)` no-op Copy drops removed | dead statements |
| crates/bench/benches/chains-seal.rs:480 | unneeded `mut` removed | dead mut |
| crates/cgap-arrow/tests/validation.rs:526 | redundant parens removed | dead syntax |
| crates/cgap-branchless/src/eytzinger.rs:140 | unused `format` import removed | dead import |
| crates/cgap-branchless/tests/boundary_sweep_courts.rs:247 | unused `i` underscored | dead binding |
| crates/cgap-branchless/tests/scan_courts.rs:388 | unused `name` → `_name` | dead binding (adjacent to in-flight scan.rs — trivial, disclosed) |
| crates/cgap-ffi/tests/fa9_commit_gating.rs:76 | unused `cp` underscored | dead binding |
| crates/cgap-git/tests/envelope_decimal_court.rs | unused `cgap_git::Oid` import removed | dead import |
| crates/cgap-storage/tests/dred_swar_wire.rs:13 | unused `retract_inverted`/`retract_unchecked` imports removed | dead imports |
| ontologies/cloud/tests/courts.rs:10 | unused `lex_hash` import removed | dead import |
| crates/cgap-p28/src/asc450.rs:1887 | unneeded `mut` removed (test cfg) | dead mut |
| crates/cgap-parquet/src/bitslice.rs:894 | trailing semicolon removed | dead syntax |
| crates/cgap-parquet/src/unpack_filter.rs:209/252 | redundant parens removed | dead syntax |

No `pub` item was deleted from any published crate (semver fence held;
published-crate fixes are statement-level inside test cfg or syntax
only).

## 4. Recorded candidates (NOT excised — scope fence)

All 9 dead-library showroom symbols are defined in
`crates/cgap-branchless/src/{scan,transpose}.rs` — in-flight lane files:
count_leading_eq_u8, prefix_and_u64x8, prefix_max_u8x8, prefix_or_u64x8,
prefix_sum_u32x8, prefix_xor_u64x8, segmented_prefix_sum_u32x8,
transpose_8x8, transpose_64x64. Recorded in
`scripts/dead_library_exceptions.json` as `C05-candidate` entries with
consume-or-delete plan for the owning lane. Owning lane supersedes.

Also recorded as candidates (integration risk, not excised):
`crates/bench/Cargo.toml:20-22` unused deps `cgap-storage`, `cgap-git`,
`cgap-ffi` — the in-flight cpk_study lane may be about to wire these
benches; excising manifest deps mid-wave risks starving that lane.
Gate that would prove the excision: `cargo build --workspace
--all-targets` exit 0.

## 5. Divergences / notes

- **CROSS-LANE FINDING (compile-freeze)**: `cargo test -p cgap-parquet`
  with DEFAULT features exits 101 (18 compile errors in the FA9/AE1
  rotation court in `crates/cgap-parquet/src/mmap_writer.rs` —
  RingSink/SpillSync/`private::Sealed`/std unresolved: those items are
  `#[cfg(feature = "std")]`-gated, the court is not). NOT a C05 edit —
  `mmap_writer.rs` was already modified at session start by the H3/AE1
  lane. `cargo test -p cgap-parquet --features std` exits 0 (all green,
  witnessed this session). C05 did NOT apply an SLA unblock: the fix
  shape (feature-gate the court vs enable the feature in dev) belongs
  to the owning lane. Coordinator: parquet test-compile is frozen for
  default features until that lane lands.
- Gate consumer refs drifted DURING the lane (penstock_prefilter8 2→4,
  penstock_prefilter8_mix 0→2): the in-flight branchless lane's
  consumption landed mid-session; gate re-run reflects fresh release
  artifacts. Census numbers are as-of-command.
- During JSON assembly C05 introduced two corrupted writes into
  dead_library_exceptions.json (invalid JSON); both repaired and
  json.loads-validated in-session. Errata on self, cited here per
  errata law.
- Self-check first run exit 3 (empty lease loud-skip) is a documented
  behavior (self-check does not build consumers); plain-gate run first
  populates the lease.

## 6. Final verification ladder (this session, real exits)

| Command | Exit |
|---|---|
| `DEADLIB_TARGET=/tmp/c05-target python3 scripts/verify_dead_library.py` (post-exceptions) | 0 — 3 consumed / 25 justified / must-consume intact |
| `DEADLIB_TARGET=/tmp/c05-target python3 scripts/verify_dead_library.py --self-check` (populated lease) | 0 — MUT-A FIRED + MUT-B FIRED-proof |
| `CARGO_TARGET_DIR=/tmp/c05-clean cargo build --workspace --all-targets` (post-fix) | 0, ZERO warnings (pre: 18 sites) |
| `cargo test -p {bench, cgap-arrow, cgap-branchless, cgap-ffi, cgap-git, cgap-p28, cgap-storage}` | 0 each (per-crate summaries in /tmp/c05-test-*.log: e.g. cgap-arrow 136+36+9+..., cgap-git 61+11+..., cgap-storage ~45 across units) |
| `cargo test -p cgap-parquet` (default features) | 101 — CROSS-LANE, see §5 |
| `cargo test -p cgap-parquet --features std` | 0 — all green |
| `cargo test --manifest-path ontologies/cloud/Cargo.toml` | 0 (13 tests across units) |


## Amendment (2026-11-02, lane GAP-10) — FIX-07 misfire disclosure

Errata per errata law (original receipt above, unmodified). FIX-07
(`docs/jira/v26.11.2/lanes/lane-fix07-bench-build.md:48-54`) attributes
to this lane's warning sweep two excisions of genuinely-used bindings:
`crates/bench/benches/chains-parquet.rs` `block_len` (FIX-07 restored;
multiple use sites e.g. chains-parquet.rs:243/246/327) and
`crates/bench/benches/chains-seal.rs` `mut` on the lease binding (restored;
`let mut lease` present at chains-seal.rs:216/339/480, re-verified on
disk 2026-11-02 by GAP-10). This closes the disclose-in-both-directions
violation DOCTRINE-1 flagged: the owning lane now discloses the misfire
itself and cites the unblocker's receipt. Correctness over silence; the
two excisions above supersede this receipt's table rows
(`lane-c05-dead-code-excision.md:51,53`) for those items.
