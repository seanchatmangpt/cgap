# Repair 08 — AE7 alloc_boundary courts (stack overflow + ceiling mis-measurement)

Lane F1 (AE7 finish). Date: 2026-11-02. Branch main, no git writes.

## Files (lane ownership)

- `crates/cgap-core/src/tracking_alloc.rs`
- `crates/cgap/tests/alloc_boundary.rs`
- this receipt

## Root causes (diagnosed this session)

### RC1 — stack overflow at the first traced alloc (the post-fix SIGSEGV-class crash)

The positive control enabled `TRACE_ON` and the allocator's `alloc`/`realloc`
paths called `eprintln!` when `TRACE_ON != 0` in the `GlobalAlloc` methods.
Formatting/stdio machinery can allocate; that allocation re-enters the
allocator while `TRACE_ON` is still 1 — infinite recursion, stack overflow
(SIGABRT via "overflowed its stack"). Witnessed: binary run emitted ZERO trace
lines before the overflow — the recursion fired on the very first traced
print. `TRACE_ON` and both in-allocator `eprintln!` trace sites are removed
(the `breach()` diagnostic `eprintln!` survives: it runs only after
`ARMED=false` and, measured, does not re-enter enforcement).

### RC1b — poison-serve static overflow

The prior poison-arena design served a fixed 128 B block for ANY breach size;
a large breaching alloc (court 4's ~262 KB Vec grow, offending=262144
witnessed) would write `layout.size()` bytes into 256 B of static memory — a
static-memory buffer overflow (the dispatch's "callers write through the
poison pointer" SIGSEGV channel). The concurrent lane rewrite already
replaced poison-serve with serve-real-memory (inner.alloc after disarm), so
the channel is structurally closed on disk; the 128 B poison path was removed
by that rewrite (mtime law note: my early edits raced that rewrite; my
poison-arena enlargement was discarded in favor of the landed rewrite, which
is strictly stronger).

note: my first filter attempt used `thread::current()` INSIDE the allocator:
it allocates its fallback Thread when the TLS handle is cold — infinite
recursion, stack overflow (measured). The landed design avoids the std thread
API entirely.

### RC2 — court 4 ceiling measured on the wrong fixture/window

W4's 8192/524288 (and the interim 512/2 MiB) are breached: full-window
measurement on the exact 4096-row lever_courts corpus gives
allocs=278, bytes=14_216_246 (witness `AE7-FP-MEASURE` line,
`cargo test -c ... -- --nocapture`, exit 0). New ceiling = 2x measured
(allocs rounded to 1024): `AE7_FP_BUDGET_ALLOCS=1024`,
`AE7_FPBUDGET_BYTES=33_554_432` (H8 honesty: ceiling, not zero, loud).

There was also a suite-only flake: under the libtest harness the HARNESS
thread allocated (a stray 4 B alloc) mid-window and breached
`ae7-penstock-ingress-16` on a foreign thread — standalone passed, full-suite
failed (timing race). Design decision: budget state stays process-global,
but enforcement is now scoped to the OWNER thread via a stack-anchor
band (see RC3).

### RC3 — foreign-thread enforcement flake

Budget state is process-global; under libtest, foreign (harness) threads
allocate concurrently. Landed design: `OWNER_SP` — the stack address of an
anchor local captured at arm time; `is_owner_thread()` is a 1 MiB-band
stack-pointer band check (allocation-free: two stack reads + atomics; two
std-thread-API attempts — `thread::current()` fallback-alloc, Mutex —
measured to re-enter the allocator; Mutex.lock has no recursion but
`thread::current()` does). Foreign-thread stacks are separate mmap mappings
(macOS: TBs apart), so foreign SPs fall outside the 1 MiB band. This makes
courts deterministic under the parallel harness.

## Before/after court grid

| court | before (session start) | after |
|---|---|---|
| 0 positive control | stack overflow (SIGABRT) | PASS (flips: `BREACH site="ae7-positive-control" allocs=1 bytes=8 offending=8`) |
| 1 penstock ingress | blocked by court 0 crash | PASS, (0,0) witness |
| 2 parquet seal steady | blocked | PASS, (0,0) witness |
| 3 bn3 radix probe | blocked | PASS, (0,0) witness |
| 4 bn3 fixpoint (H8 ceiling) | breach at 204 allocs / 2,198,076 B (ceiling 512/2 MiB) | PASS under 1024/33_554_432; measured 278/14_216_246 |

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/f1-target cargo test -p cgap --test alloc_boundary` —
  exit 0 (before fixes: stack-overflow abort; then court-4 ceiling flip;
  then suite-only court-1 foreign-thread breach)
- `CARGO alloc_boundary -- --nocapture` — AE7-FP-MEASURE allocs=278
  bytes=14_216_246, exit 0 (ceiling measurement witness)
- `C 8x ./alloc_boundary-...` direct binary reruns — 8/8 exit 0 (flake check)
- `cargo test -p cgap` (full suite, 13 test binaries) — exit 0, all
  `test result: ok`, no FAILED lines
- Interim failures witnessed and fixed forward: cgap-storage compile break
  (owning lane fixed it within the SLA; no unblock fix applied by me) and
  `crates/cgap/src/pipeline.rs` delimiter error (resolved by owning lane
  two polls later) — disclosed per compile-freeze SLA, no edits by this lane
  to either file.

## Standing

ALIVE for the three owned files (commands + exits above, this session,
exact subject).

Divergences: another lane rewrote tracking_alloc.rs mid-flight (serve-real-
memory design; my poison-arena enlargement discarded); coordinator-owned
lib.rs is still without `pub mod tracking_alloc;`, so the test still includes
it via `#[path]`. Perf numbers are session-measured on the host court binary;
re-measure on target hardware before quoting.
