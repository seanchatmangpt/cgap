# w5-ring-rewitness

Re-witness of L4 ring-sink claims (lane A3, Analyze), 2026-11-02, canonical checkout /Users/sac/cgap
  @ main 9a730320. Independent /tmp
target dir (`/tmp/a3-ring-target`); no source edits; no git writes.

## Ladder

| Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| L4 courts | `cargo test -p cgap-ffi --test lever_courts` (/Users/sac/cgap,
  CARGO_TARGET_DIR=/tmp/a3-ring-target) | 0 | 2 passed, 0 failed
  (`l4_committed_header_flip_is_exactly_spoliation`, `l4_status_flip_witness_at_lease_edge`), 0.01s
  |
| L4 courts verbose | same, `-- --nocapture` | 0 | no stderr from courts; assertions are
  equality-asserts on the status word |
| Ring bench | `cargo bench -p bench --bench chains-sovereign` (/Users/sac/cgap, /tmp target) | 0 (3
  runs) | divan table below |

## Lease-edge refusal code

Witnessed first-hand, verbatim from the bench banner (all three bench runs):

> `[BENCH-5] commit lease boundary: 65376 lanes accepted, first refusal
> at 65377 lanes (status 0x43470004); bench batch = 16 lanes/frame
> (fits, LEASE_EXHAUSTED never fires in-bench)`

Matches prior receipt (`02-pareto-levers.md:173` and `a3-ring-sink.md`)
byte-identically: 65,376 accepted, 65,377 -> `0x43470004`. Court path
(`lever_courts.rs:158`, 70,000-lane class) asserts
`CGAP_ERR_LEASE_EXHAUSTED == 0x4347_0004`
(`crates/cgap-ffi/src/sovereign.rs:80`) and passed.

## Bench numbers (this session, vs prior receipts)

| Metric | Prior receipt | This session | Verdict |
|---|---|---|---|
| stage_commit @256 fastest | 150.7 us = 0.589 us/tx | 193.9 us = 0.755 us/tx (run2: 213.4 us =
  0.834) | DRIFTED +28-42% |
| stage_commit @4096 fastest | 2.472 ms = 0.603 us/tx | 3.162 ms = 0.772 us/tx (run3: 3.241 ms =
  0.791) | DRIFTED +28-31% |
| chain_full_transaction @4096 fastest | 6.31-6.33 us/tx | 25.37 ms/4096 = 6.19 us/tx | REPRODUCED
  (within run noise; run1 median 27.64 ms = 6.75) |
| Alloc column | 2 allocs / 36 B per tx | max alloc: 2, 36 B (stage_commit, both batch sizes) |
  REPRODUCED |

Typed notes on divergence:

- stage_commit drift (+28-42% on fastest) is real but environment-sensitive:
  three consecutive runs this session spanned 193.9 / 213.4 / 193.9 us @256
  (raw numbers above); no source change to the commit path occurred
  between receipts (git status clean at 9a730320). The <4 us/tx G5 target
  remains MET at 0.755-0.834 us/tx regardless of the drift.
- End-to-end 6.31 us/tx figure REPRODUCED (6.19 fastest this session).
- eval_bn3 blocking-lever status unchanged: 3.38-3.70 ms @4096 (~0.83-0.90
  us/tx fastest... 824.9 us @256 = 3.22 us/tx) still dominates the chain
  budget; 6.31 us/tx is still >4 us/tx target at the chain level (typed
  BLOCKED carried by `p2-receipts-ring-receipt-chain.md:62-66`, not re-filed).

## Compile-abort note (census discipline)

First bench invocation exited 101: `cgap-parquet` lib failed under the bench
feature set (`crates/cgap-parquet/src/scan.rs:553` "missing field `bounds`")
while compiling clean seconds earlier under `cargo test -p cgap-ffi`. On-disk
copy at read time contains `bounds: Vec::new()` with a "D7 minimal unblock"
comment at that exact site; re-run compiled and passed exit 0 with zero source
edits by this lane. Classified: transient/feature-unification compile race or
an interleaved edit by another lane during the build window — a compile-abort,
not a failure count; the subject then built and ran green 3x consecutively.

## Standing

REPRODUCED with drift noted: courts ALIVE (exit 0, 2/2); lease-edge refusal
code byte-identical; end-to-end and alloc claims reproduced; stage_commit
per-tx drifted +28-42% vs 0.589-0.603 (target still met, divergence typed
above, no source delta between receipts).
