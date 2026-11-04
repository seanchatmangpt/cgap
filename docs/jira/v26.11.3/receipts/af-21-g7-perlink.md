# AF-21 — G7 Per-Link Receipt-Chaining Cost (measured)

Status: ALIVE — bench ran this session (2 release runs, exit 0); G7
"measured" line appended to
`/Users/sac/arcus/docs/jira/v26.11.3/cbmqs-standard.md` (coordinator
correction: the standard is in the arcus tree, not cgap).

Plane disclosed: **cgap-aoq** (AB-7's chain.rs append API is LANDED —
`crates/cgap-aoq/src/chain.rs:215` `AoqChain::append` — so the aoq
plane is benched, not cgap-ffi's ring).

Subject: `crates/cgap-aoq/tests/receipt_link_bench.rs` (new file, this
lane). Surface: `AoqChain<Blake3Digest>` binding the repo's sovereign
`cgap_blake3::hash` to the caller-injected `ChainDigest` seam — real
BLAKE3 every link, no stand-in digest.

## Method

- (a) per-link append: min-of-1000 individually-timed `append` calls
  (`H(payload)` + `H(prev|epoch|index|H(payload))`, two BLAKE3 calls
  per link) + whole-leg (min of 5 legs x 1000 appends / count).
- (b) 64B per-call latency: `cgap_blake3::hash` over 64B input, n=1000
  individually-timed calls, min of 3 reps, release. Per-call wall
  including call overhead — the honest replacement for the UNSUPPORTED
  18.1 ns figure (AF-2's correction). No fence invented; measured only.
- (c) verify: `verify_all` whole-leg, min of 5 legs, us/link at
  1k / 10k / 100k.

Command + exit (both runs, `test result: ok. 1 passed`, exit 0):

```
cargo test -p cgap-aoq --test receipt_link_bench --release -- --nocapture
```

## Results (release, Apple Silicon, this session)

| Metric | Run 1 | Run 2 | Adopted (min across runs) |
|---|---|---|---|
| (a) append min-of-1000 (us/link) | 0.291 | 0.541 | 0.291 |
| (a) append mean-of-1000 (us/link) | 0.363 | 0.614 | 0.363 |
| (a') append whole-leg 1k (us/link) | 0.332 | 0.384 | 0.332 |
| (b) 64B per-call min (ns) | 83.0 | 125.0 | 83.0 |
| (b) 64B per-call mean (ns) | 143.4 | 161.5 | 143.4 |
| (c) verify 1k (us/link) | 0.224 | 0.264 | 0.224 |
| (c) verify 10k (us/link) | 0.226 | 0.265 | 0.226 |
| (c) verify 100k (us/link) | 0.226 | 0.265 | 0.226 |

Verbatim machine-parseable lines from both runs:

```
G7RESULT append_min_us_per_link=0.291 append_mean_us_per_link=0.363 append_leg1k_us_per_link=0.332 hash64_min_ns=83.0 hash64_mean_ns=143.4 verify_us_per_link_1k=0.224 verify_us_per_link_10k=0.226 verify_us_per_link_100k=0.226
G7RESULT append_min_us_per_link=0.541 append_mean_us_per_link=0.614 append_leg1k_us_per_link=0.384 hash64_min_ns=125.0 hash64_mean_ns=161.5 verify_us_per_link_1k=0.264 verify_us_per_link_10k=0.265 verify_us_per_link_100k=0.265
```

## Findings

- (a) per-link append cost: **0.291 us/link** min (mean 0.363). Two
  BLAKE3 calls per link plus a 120-byte record mint. Measured only —
  no numeric ceiling or fence claimed; the CBMQS standard records G7
  structurally with none.
- (b) the 18.1 ns figure in earlier prompts was UNSUPPORTED and is NOT
  reproduced: measured 83.0 ns min / 143.4 ns mean per 64B call
  (single-call wall including call overhead).
- (c) verify is O(1) per link and FLAT across 1k/10k/100k
  (0.224-0.265 us/link) — chain cost is length-independent.

## Non-vacuity

Before timing, the bench asserts the record digest equals the
independently constructed `H(prev|epoch|index|H(payload))` over the
real BLAKE3 kernel, and that the digest is not zero-folded. Both runs
passed.

## Receipt

- Command + exit: `cargo test -p cgap-aoq --test receipt_link_bench
  --release -- --nocapture` — `test result: ok. 1 passed` twice.
- Diff: exactly one new test file,
  `crates/cgap-aoq/tests/receipt_link_bench.rs`; nothing under `src/`;
  nothing outside the lane-owned file + this receipt. Never ran git.
- Standing: bench ALIVE (command+exit above); G7-row append LANDED —
  coordinator correction: the standard lives in the arcus tree at
  `/Users/sac/arcus/docs/jira/v26.11.3/cbmqs-standard.md` (AF-13 landed
  it; my earlier absence search covered only the cgap tree). A
  "Measured (AF-21, ...)" line was appended under the standard's G7
  section citing this receipt, with the AF-22 cross-cite (AF-21 min
  83 ns vs AF-22 p50 167 ns = same surface, min vs median statistics —
  one measurement family, not conflicting numbers). The earlier
  PARTIAL/BLOCKED status on the G7 append is superseded by this line;
  the bench itself was never blocked.
