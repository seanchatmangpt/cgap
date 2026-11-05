# ZD-53 — cgap-git court fixes (soc zero-alloc window + octopus child capture)
Standing: ALIVE

Date: 2026-11-04. Lane: ZB-4 findings, no git commands.
Subject (exact files):

- `crates/cgap-git/tests/soc_full_chip_court.rs` — per-thread alloc window
- `crates/cgap-git/tests/octopus_merge_court.rs` — child re-exec capture
- `crates/audit/src/alloc_counter.rs` — UNTOUCHED (test-side fix sufficed;
  the audit counter was already per-thread and correct)

## 1. soc_full_chip_court zero-alloc flake (~50%, ZB-4 finding)

### Root cause (found, fixed)

The zero-alloc window was **process-wide**, not per-thread:
`soc_full_chip_court.rs` declared `static WINDOW_OPEN: AtomicBool` +
`static ALLOC_COUNT: AtomicUsize` and the `#[global_allocator]` hook
incremented them whenever the global flag was open. With cargo's default
parallel test threads, ANY other test thread allocating while the window
was open (soc's spawned 64 MiB-stack pipeline thread vs sibling test
threads in the same binary — the binary has 3 tests) landed in the
count. Load-dependent, hence intermittent; reproduced 3/6 serial×6 by
ZB-4, 0/5 here after the fix.

Note: the 64 MiB-stack spawned thread is the TEST's own pipeline thread
and opens the window itself; it is not a third-party allocator. The
leak source was sibling test threads (process-wide flag), not the join
race suspected in the finding. `crates/audit/src/alloc_counter.rs` was
already per-thread (const-init `Cell<Tracking>` TLS) — no fix needed
there.

### Fix (test-side, audit crate untouched)

Replaced the process-wide atomics with const-init `Cell<WindowState>`
TLS (`ALLOC_WINDOW`) checked in the allocator hook — the same pattern
as `crates/audit/src/alloc_counter.rs:35-37`. `open_window()` resets +
opens on the calling thread; `close_window()` closes and returns the
per-thread count. Sibling threads no longer contribute.

### Stability witness

`cargo test -p cgap-git --release --test soc_full_chip_court -- --test-threads=1` ×5:

```
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.42s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.58s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.80s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
```

(5/5 green, 3 tests each.)

### Anti-vacuity retained

The positive control (`positive_control()`, vec![7u8; 512] in-window)
still fires first and asserts count > 0, and the isolated-demux window
still asserts `n == 0` — the court still cannot pass on a dead
instrument (a dead instrument fails the positive control; a leaking
window fails the zero assert).

## 2. octopus_merge_court child stdout leak

### Root cause

The 101-class Andon witness re-executed the test binary with
`Command::status()` (streams inherited): the child's FAILED summary and
panic text landed on the shared harness stdout. Parent asserted
exit 101 and stayed green while its own stdout carried a FAILED block —
exactly the ZB-4 finding.

### Fix

`.status()` → `.output()` (piped capture). Assertions moved onto the
captured result: `out.status.code() == Some(101)` plus a new Andon-semantics
witness asserting the captured text contains `ANDON-REFUSAL` (the
panic text from `octopus_merge.rs:366` — witnessed live: it appears in
the child's captured stdout because libtest captures panic output; the
pre-fix child put this text on the shared stdout). The refusal
mechanism (`octopus_merge.rs:366` panic → exit 101) is untouched.

## Verification ladder (this session, exact commands + exits)

| Command | Result |
|---|---|
| `cargo build -p cgap-git --tests` | exit 0 |
| `cargo test -p cgap-git --release` (run A) | exit 0, 40/40 test binaries ok, 244 passed / 0 failed |
| `cargo test -p cgap-git --release` (run B) | exit 0, 40 ok / 0 FAILED |
| `cargo test -p cgap-git --release --test soc_full_chip_court -- --test-threads=1` ×5 | 5/5 green (3 tests each) |
| `cargo test -p cgap-git` (debug) | exit 0, 0 FAILED (log below) |
| live child inspection: `GIT12_ANDON_CHILD=1 <bin> --exact mutation_conflict_injection_flip` | exit 101, ANDON-REFUSAL present in captured stdout, empty stderr |

Debug-run log + release run B log: `/tmp/dbg.log`, `/tmp/rel3.log` on
this machine (session artifacts; summaries above are re-read from the
logs at receipt time).

## Standing

ALIVE: soc court zero-alloc claim is now per-thread-scoped, serial ×5
green, full release suite ×2 green (244/0), debug suite green.
`alloc_counter.rs` untouched (per-thread by construction, ZD-3's audit
verified: `alloc_counter.rs:35-37` const-init TLS).
