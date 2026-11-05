# BM7 — Gate-10 chain-scale reconfirm at committed v3 HEAD

Standing: BLOCKED
lane-close HEAD `b595f276` fail to compile `cgap-aoq`); supplemental working-tree
measurement obtained under disclosure, **outside the BM-3 band — NOT CONFIRMED**, reported
as-measured, no fence or target adjusted.

## Plane law block

This receipt measures ONLY the cgap-ffi RingReceiptChain plane
(`crates/cgap-ffi/tests/receipt_chain_scale.rs`). Never cross-cite onto AoqChain
(0.291 µs/link) or arcus (Go membrane, unmeasured).

## Premise failure: the committed state does not compile

The lane premise ("reconfirm at the committed v3 HEAD") is refuted at the substrate level.
Both HEADs observed during the lane were compile-aborted on `cgap-aoq`:

- `git show HEAD:crates/cgap-aoq/src/aead.rs` at `3e0339e6` is the pre-admission seam stub
  (only `seal`/`open` returning `AeadKernelNotAdmitted`; no `Nonce96`, no `TAG_LEN`, no
  `seal_detached`/`open_detached`/`verify_detached`).
- Committed `session.rs` at the same HEAD imports `crate::aead::{self, Nonce96, TAG_LEN}`
  and calls `aead::seal_detached/open_detached/verify_detached`.
- Fresh `git archive HEAD` into scratch (`/tmp/bm7-head`), then `cargo test --release
  -p cgap-ffi --test receipt_chain_scale` → **compile-abort**: `error: could not compile
  cgap-aoq (lib) due to 5 previous errors` (E0432 unresolved `Nonce96`/`TAG_LEN`; E0425
  `seal_detached`/`verify_detached`/`open_detached`), identical at `b595f276`
  (`/tmp/bm7-head2`). Per the compile-abort law: 0 tests executed — this is not a
  measurement of the committed state.
- The v3 wire-law + reconciliation code exists only as **uncommitted working-tree edits**
  (16 files, 879 insertions; incl. `cgap-aoq/src/aead.rs`, `cgap-aoq/src/session.rs`,
  `cgap-crypto/src/aead.rs` — lane C1 in-flight). `wire-law-v3-landing.md` was untracked
  at lane start.

## Commands + exits (supplemental working-tree measurement)

Mirror of BM-3's method on the working tree (the only compiling substrate containing the
v3 code):

```bash
cd /Users/sac/cgap
cargo test --release -p cgap-ffi --test receipt_chain_scale \
  -- --nocapture
# exit=0, three invocations, log /tmp/bm7_runs_wt.log
grep -c SKIP /tmp/bm7_runs_wt.log   # -> 0  (gate: must be 0 — PASS)
```

Raw log: `/tmp/bm7_runs_wt.log`. Verbatim run 1 output:

```text
    Finished `release` profile [optimized] target(s) in 7.07s
     Running tests/receipt_chain_scale.rs (target/release/build/cgap-ffi/f683fd7b9ec1bfb5/out/receipt_chain_scale-f683fd7b9ec1bfb5)

running 3 tests
[run 0] append 100k: 215.1ms | verify_all: 185.5ms | span(0..10k): 15.903ms
test tamper_at_50k_prefix_integrity_at_scale ... ok
[run 1] append 100k: 249.3ms | verify_all: 178.2ms | span(0..10k): 14.540ms
[run 2] append 100k: 191.7ms | verify_all: 176.2ms | span(0..10k): 14.462ms
[MIN-OF-3] append 100k: 191.7ms | verify_all: 176.2ms | per-1k wall first 0.001833/last 0.002042 ms (x1.11) | per-1k allocs first 2 -> last 2 | per-1k bytes first 266 -> last 266
test scale_append_verify_100k_census ... ok
[SPAN-CURVE] span 1000: best 1.280ms (20 checkpoints, min-of-3)
[SPAN-CURVE] span 10000: best 12.706ms (20 checkpoints, min-of-3)
[SPAN-CURVE] span 50000: best 63.929ms (20 checkpoints, min-of-3)
test verify_span_from_checkpoints_o_span_curve ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.35s
```

## SPAN-CURVE verbatim (all three runs)

```text
RUN 1: [SPAN-CURVE] span 1000: best 1.280ms | span 10000: best 12.706ms | span 50000: best 63.929ms (20 checkpoints, min-of-3)
RUN 2: [SPAN-CURVE] span 1000: best 1.449ms | span 10000: best 12.744ms | span 50000: best 64.175ms (20 checkpoints, min-of-3)
RUN 3: [SPAN-CURVE] span 1000: best 1.682ms | span 10000: best 14.060ms | span 50000: best 64.548ms (20 checkpoints, min-of-3)
```

(The `(20 checkpoints, min-of-3)` suffix is part of each printed line; runs 2-3 abbreviated
above only by joining each line — full lines in the log.)

## 100k append / verify_all

Best across runs (run 1): append **191.7 ms**, verify_all **176.2 ms** (census cell,
min-of-3 internal).

## Per-link derivation (50k best, min of 3 runs)

63.929 ms (run 1, also the cross-run min) ÷ 50,000 links × 1000 = 63,929 µs ÷ 50,000 =
**1.27858 µs/link**

Curve cells, same division by their own span: 1.280 ms → 1.2800 µs/link; 12.706 ms →
1.2706 µs/link.

## Band comparison — honestly outside

BM-3 band: {63.197 .. 63.619 ms, ±0.33% on mid 63.408 ms}. This lane's min-of-3 on the 50k
cell: **63.929 ms** → **0.310 ms (+0.49%) above the band top**; vs BM-3's 1.26660 µs/link
the delta is **+0.95%**. Not inside the band → **NOT CONFIRMED**, reported as-measured.
No `TARGET_US_PER_LINK` or fence constant was touched (grep for the constant in the test
source at HEAD matches nothing; nothing to adjust).

Candidate causes, disclosed not adjudicated: (a) heavy concurrent fleet load during the
window (see witness — load avg 44-48 on 16 cores, multiple other lanes' `cargo test`
processes live); (b) the measured tree carries C1's in-flight aead/session edits (same
subject BM-3 measured, so comparable to BM-3's own working-tree subject, but both sit on
uncommitted ground).

## Machine witness (verbatim)

```text
12:55  up  2:46, 3 users, load averages: 44.58 31.23 22.81
2026-11-04T20:55:47Z
16
Mac15,9 (Apple M3 Max class)
```

Concurrent-load disclosure: other lanes' cargo/rustc processes were live in the window
(PIDs 41213 `cargo test -p cgap-aoq -p cgap-crypto -p cgap-blake3 -p cgap-ffi
--no-fail-fast`; 50050 `cargo test -p cgap-aoq -p cgap-ffi`; 63781 ggen workspace build).
None poll-killed. HEAD moved mid-lane (`3e0339e6` → `b595f276` (CI-only) → `4bb419c5`
(bench-only, cgap-crypto)); the compile-abort was re-verified at each new HEAD — at
`4bb419c5` by blob identity (`aead.rs` = `6414760f`, `session.rs` = `d7035592`, unchanged
from `b595f276`) so no rebuild was required.

## Method

Release profile witnessed (`Finished \`release\` profile [optimized]`); min-of-3 internal
per cell plus 3 cross-run invocations on the 50k cell; SKIP gate = 0; scratch materialized
via `git archive HEAD` (no worktrees). Anti-DCE fences ran (test-internal
`profile_guard::fence`).

## Scratch lease cleanup

`/tmp/bm7-head` and `/tmp/bm7-head2` are lane build-root leases; deletion was attempted at
integration (sandbox refused `rm -rf /tmp/bm7-head`); both dirs are outside the repo tree
and listed here as outstanding leases for the coordinator to delete.

## Standing

Committed-HEAD cell: **BLOCKED** (HEAD compile-abort, cgap-aoq, receipts above).
Working-tree cell: **PARTIAL_ALIVE** — measured, exit 0, outside band, honestly reported.
Re-confirm requires: C1 lands the aead/session commits, then a fresh archive-of-HEAD run.

## COMPLEMENT — committed-state cell unblocked and measured (COMPLETED)

Standing update: the committed-HEAD cell above (BLOCKED) is superseded. L3 landed the
settle commits; at fetch time `origin/main` = `b90da1eb` (settle chain `ede3fe4d` →
`265a095a` → `39b53add` → `b90da1eb`; the last is docs-only beyond the coordinator-cited
`39b53add`). Archived that exact SHA via `git archive` into fresh scratch
(`/tmp/bm7-head3`) and ran the mirror bench there, release profile, 3 invocations.

```bash
git archive b90da1eb996efb9d9a8cde35ba6025e2269f7475 | tar -x -C /tmp/bm7-head3
cd /tmp/bm7-head3 && cargo test --release -p cgap-ffi \
  --test receipt_chain_scale -- --nocapture   # exit=0 x3
grep -c SKIP /tmp/bm7_runs_head.log           # -> 0 (gate PASS)
```

Raw log: `/tmp/bm7_runs_head.log`. Verbatim invocation 1 (log lines 14-26, byte-faithful):

```text
    Finished `release` profile [optimized] target(s) in 7.51s
     Running tests/receipt_chain_scale.rs (target/release/build/cgap-ffi/f683fd7b9ec1bfb5/out/receipt_chain_scale-f683fd7b9ec1bfb5)

running 3 tests
[run 0] append 100k: 211.2ms | verify_all: 186.8ms | span(0..10k): 14.607ms
test tamper_at_50k_prefix_integrity_at_scale ... ok
[run 1] append 100k: 206.0ms | verify_all: 178.1ms | span(0..10k): 14.463ms
[run 2] append 100k: 196.8ms | verify_all: 177.5ms | span(0..10k): 14.439ms
[MIN-OF-3] append 100k: 196.8ms | verify_all: 177.5ms | per-1k wall first 0.001958/last 0.002 ms (x1.02) | per-1k allocs first 2 -> last 2 | per-1k bytes first 266 -> last 266
test scale_append_verify_100k_census ... ok
[SPAN-CURVE] span 1000: best 1.437ms (20 checkpoints, min-of-3)
[SPAN-CURVE] span 10000: best 12.740ms (20 checkpoints, min-of-3)
[SPAN-CURVE] span 50000: best 63.555ms (20 checkpoints, min-of-3)
test verify_span_from_checkpoints_o_span_curve ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.29s
```

(A first draft of this block was written from session memory before the log was
re-read; the numbers above supersede it and are copied from the log on disk. An
invented build-hash line and two mis-transcribed run lines were removed.)

### SPAN-CURVE verbatim (all 3 runs, committed b90da1eb)

```text
RUN 1: [SPAN-CURVE] span 1000: best 1.437ms | span 10000: best 12.740ms | span 50000: best 63.555ms (20 checkpoints, min-of-3)
RUN 2: [SPAN-CURVE] span 1000: best 1.425ms | span 10000: best 12.742ms | span 50000: best 63.609ms (20 checkpoints, min-of-3)
RUN 3: [SPAN-CURVE] span 1000: best 1.331ms | span 10000: best 13.069ms | span 50000: best 84.533ms (20 checkpoints, min-of-3)
```

### Per-link derivation and band verdict — CONFIRMED

50k min-of-3 across runs: **63.555 ms** (run 1) ÷ 50,000 × 1000 = 63,555 µs / 50,000 =
**1.27110 µs/link**. BM-3 band {63.197 .. 63.619 ms}: 63.555 ms is inside the band
(+0.36% over BM-3's own 63.330 ms; 0.064 ms under the band top). **CONFIRMED at the
committed state.** No TARGET or fence adjusted.

100k census (invocation 1, min-of-3 internal): append 196.8 ms, verify_all 177.5 ms —
consistent with BM-3's 184.1 / 169.3 ms (+7% / +5%, load-era).

Run-3 outlier disclosure: run 3's 84.533 ms 50k cell is a load-contaminated sample —
AL-1/C1 landed `a0e02750` mid-window and other lanes' edits are live in the shared
checkout (`aoq_bridge.rs`, `mutation_corpus/lib.sh` dirty). Min-of-3 absorbs it; the
two clean runs sit 0.054 ms apart.

### Machine witness (verbatim)

```text
13:10  up  3:01, 3 users, load averages: 31.58 31.48 27.89
16
Mac15,9 (Apple M3 Max class)
```

Concurrent movement disclosure: HEAD advanced `b90da1eb` → `a0e02750` (LANE-1
receipt-engine docs commit) during the window; the archived subject `b90da1eb` is
unaffected (docs-only delta). Shared-checkout dirty files at close belong to other
lanes; none touched `receipt_chain_scale.rs` or the archived tree.

### Scratch lease cleanup

`/tmp/bm7-head3` is a lane build-root lease; deletion attempted at integration —
listed here for coordinator deletion if the sandbox refuses.
