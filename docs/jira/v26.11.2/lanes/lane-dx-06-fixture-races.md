Lane DX-6 — shared-fixture race sweep + court (receipt)
Repo: /Users/sac/cgap (canonical checkout, main). Date: 2026-11-02.

## Classified inventory (crates/*/tests/*.rs, /tmp/ literals)

SAFE (read-only fixture / comment / inert label):
- crates/cgap-git/tests/mcp_driver_court.rs:20,77,100 — Driver::new
  labels; cgap-git/src/mcp_driver.rs:150 Driver::new stores the string,
  never touches fs.
- crates/cgap-core/tests/unit_dma_ring.rs:271,316;
  cgap-core/tests/guard_flip_court.rs:36; cgap-ffi/tests/arrow_offset_pins.rs:4,93;
  crates/cgap/tests/self_play_*.rs (comments only);
  crates/cli/tests/mutation_kills.rs:4,7,63,65 (comments only);
  crates/cgap-storage/tests/closure_alloc_court.rs:89,133 (comments only).
- crates/cgap-branchless/tests/branchless_census.rs:61 — already
  per-process (G10 fix shape): OnceLock format!("/tmp/rep1-census-{id}").

RACE (fixed shared WRITE dir) — FIXED, branchless_census pattern
(OnceLock<String> keyed on std::process::id):
- crates/cgap-branchless/tests/transpose_courts.rs:202 `/tmp/ts4-census`
  (now work() OnceLock, lines 202-208; usages 353-365, 408-472 updated;
  twin dirs ts4-mutant/ts4-pristine now keyed on process::id, line 402).
- crates/cgap-branchless (tests/scan_courts.rs:299 `/tmp/ts2-census`
  (now work() OnceLock, lines 299-305; census_work_dir line ~420).

RACE (in-process class, discovered by the dynamic witness): process-
global ALLOC counters raced against sibling-test allocations in the
same test binary — NOT a /tmp dir, but a shared-fixture race of the
same family, witnessed live:
- crates/cgap-git/tests/egress_ring_court.rs — zero_alloc_serve_court
  false-FAILED ("allocated 1 times") under two concurrent cargo test
  runs. FIXED: static SERIAL Mutex; every test in the binary holds it
  for its body (all 5 tests serialize; measured windows exclusive).
- crates/cgap-git/tests/pack_demux_court.rs — c7_parse_path_is_zero_alloc
  false-FAILED ("feed allocated 1 times"). FIXED: same SERIAL pattern,
  all 10 tests.

LOUD-PENDING (owned by in-flight lanes, NOT fixed):
- crates/cgap-git/tests/soc_full_chip_court.rs, object_store_court.rs —
  mid-witness compile-aborts (E0061 digest_of arg-count mismatch) from
  GIT-16's in-flight landing; compile-abort is not a failure count.
  Re-run the concurrent witness after GIT-16 lands.

## Court (new)
crates/cgap-court/tests/shared_fixtures_court.rs — static classifier
over crates/*/tests/*.rs; asserts zero fixed-shared-write /tmp patterns
remain. Teeth twins: (1) classifier MUST flag an embedded violating
sample; (2) the sanctioned per-process shape must NOT be flagged.
Run: CARGO_TARGET_DIR=/tmp/dx6-target cargo test -p cgap-court
→ court_verdict 14 passed + shared_fixtures_court 3 passed, exit 0.

## Dynamic witness (G10 reproduction)
Two concurrent `cargo test -p cgap-git`, CARGO_TARGET_DIR /tmp/dx6-t1
and /tmp/dx6-t2:
- Pre-fix: t1=0, t2=101 (egress_ring_court zero_alloc false-FAIL);
  3-round batch: 0/0, 0/0, 0/101 (pack_demux c7 false-FAIL). Race
  witnessed, both instances captured.
- Post-fix (targeted, the two fixed suites only): 3 rounds, all
  0/0 clean. Full-workspace cgap-git reruns blocked mid-session by
  GIT-16 in-flight compile-aborts (see LOUD-PENDING).

## Fences checked
- `cargo test -p cgap-court` (host gate for the crate I added a file
  to): exit 0, 17 tests.
- `cargo test -p cgap-branchless --test transpose_courts
  --test scan_courts` (files I edited): exit 0, 7+8 tests.
- `cargo test -p cgap-git --test egress_ring_court --test
  pack_demux_court`: exit 0, 5+10 tests.
- Must-not-regress fences I could NOT run this session: full
  `cargo test -p cgap-git` (blocked by GIT-16 in-flight compile
  aborts — not introduced by this lane; witnessed before my edits in
  round 3 of the pre-fix batch, not after).

## Standing
ALIVE for every claim above (command+exit this session). Full-tree
concurrent witness on cgap-git after GIT-16 lands: PENDING.
