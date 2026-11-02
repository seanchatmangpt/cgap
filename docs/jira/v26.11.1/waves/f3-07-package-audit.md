# f3-07-package-audit

Wave F3 item 7 — PACKAGE-AUDIT (gate-runner lane). Repo: /Users/sac/cgap.
Lease: this file only. No git writes. CARGO_TARGET_DIR=/tmp/f3-7-target (removed after run).

## Precondition poll

f3-01 version bump polled before auditing version strings: bounded poll, 20s step,
10 min budget; LANDED after 13 polls (~260s) on observable root `version = "26.11.1"`
(/Users/sac/cgap/Cargo.toml:39). No f3-01 marker file present at audit time.

## Versions (task items 2-3) — PASS

- All five manifests: `version.workspace = true` (line 3 of each); inherited root
  `[workspace.package] version = "26.11.1"` (Cargo.toml:39).
- Umbrella intra-deps (crates/cgap/Cargo.toml:12-15): all four siblings
  `path + version = "26.11.1"` — exact, not fallback.
- Kernel manifests: `[dependencies]` empty (zero-dep); only path-only dev-deps on
  unpublished siblings (cgap-arrow -> cgap-core/gate/storage at lines 22-24;
  cgap-parquet -> cgap-arrow at line 21). Dev-deps are stripped on publish.
- Cargo.lock rows: cgap 26.11.1, cgap-arrow 26.11.1, cgap-blake3 26.11.1,
  cgap-git 26.11.1, cgap-parquet 26.11.1.
- Resolution proof: `cargo package --list` re-resolves the workspace; exit 0 on all
  five post-bump manifests.

## Shipped-file audit (task item 1) — PASS with one flagged row

`cargo package --list -p <crate>` exits: blake3 0 (14 files), arrow 0 (38),
git 0 (22), parquet 0 (39), cgap 0 (7).
Contamination scan over all five lists (target-/, /target/, *.bak, credentials*,
.DS_Store, /tmp paths, *.tmp, scratch): zero matches.
include/cgap.h: zero hits in all five lists; headers (cgap.h, cgap_c_data.h,
cgap_format_codes.h) live in /Users/sac/cgap/include/ under cgap-ffi (unpublished) —
task item 4 confirmed.

Flagged (not contamination, not blocking): crates/cgap-blake3/tests/bench_tmp.rs
ships in the cgap-blake3 package. Git-tracked and committed at HEAD (pre-existing,
unmodified this session), 16 lines, a real benchmark `#[test]` over
`hash_wide_simd` (print-only, no assertion). Scratch-flavored name; real content.
Recommended follow-up: rename to tests/bench_throughput.rs or move to a bench
target in a later wave.

## Divergences

1. Parent premise "allow-dirty not needed for --list" is false on this toolchain:
   bare `cargo package --list -p <crate>` exits 101 (dirty-tree refusal: uncommitted
   Cargo.toml from f3-01) even for --list. Ran with `--allow-dirty`; --list remains
   read-only (no build, no publish, no git write). All cited file lists come from
   the --allow-dirty run.
2. f3-01 landed without its marker file at audit time; observable (root version
   string) was the landing evidence.

## Commands and exits (this session)

- bounded poll (python3, 20s step, 600s budget): LANDED poll 13
- CARGO_TARGET_DIR=/tmp/f3-7-target cargo package --list -p <crate> (x5): exit 101
  (dirty refusal, no --allow-dirty), then exit 0 (x5, with --allow-dirty)
- contamination grep over 5 list files: exit 1 = zero matches
- python3 shutil.rmtree /tmp/f3-7-target: removed, exists_after False

Standing: PASS (two flagged rows, no contamination).
