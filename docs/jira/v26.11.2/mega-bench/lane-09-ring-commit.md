# Lane 9 — Ring commit (mega-bench, 2026-11-02)

Subject: /Users/sac/cgap (main @ 9a730320 working tree), read-only lane;
 sole write = this receipt. Build root: /tmp/mb9-target (lease).

## Ladder

| Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| bench | `CARGO_TARGET_DIR=/tmp/mb9-target cargo bench -p bench --bench chains-sovereign` (/Users/sac/cgap) | 0 | banner verbatim below; divan 100 samples/leg |
| lever courts | `cargo test -p cgap-ffi --test lever_courts` | 0 | 2 passed (l4 spoliation + lease-edge) |
| tamper battery | `cargo test -p cgap-ffi --lib receipt_chain` | 0 | 5 passed (incl. tampered_receipt_or_block_is_typed_refusal, lease_exhaustion typed refusal) |

## BENCH-5 banner (verbatim)

```
[BENCH-5] commit lease boundary: 65376 lanes accepted, first refusal at 65377 lanes (status 0x43470004); bench batch = 16 lanes/frame (fits, LEASE_EXHAUSTED never fires in-bench)
```

Ceiling note: 65,376 is a count of LANES; it derives from the 65,536-BYTE
StaticPageBuffer. Bytes-vs-lanes distinction per
docs/jira/v26.11.2/waves/m3-lease-ceiling-audit.md.

## Numbers (per-tx = divan time / N; 100 samples/leg)

stage_commit / ring commit legs:

| Leg | N | fastest | median | µs/tx (fastest..median) |
|---|---|---|---|---|
| stage_commit_parquet | 256 | 193.4 µs | 218.3 µs | 0.755 .. 0.853 |
| stage_commit_parquet | 4096 | 3.407 ms | 4.102 ms | 0.832 .. 1.001 |
| ring_sink_commit | 256 | 205.5 µs | 207.0 µs | 0.803 .. 0.809 |

chain_full:

| N | fastest | median | µs/tx |
|---|---|---|---|
| 256 | 1.894 ms | 2.128 ms | 7.40 .. 8.31 |
| 4096 | 30.47 ms | 74.49 ms | 7.43 .. 18.2 |

Stage legs @256 (fastest, µs/tx): admit_frame 1.06, eval_bn3 4.39 (largest —
the blocking leg, consistent with witness attribution), commit 0.755, verify_p28 0.911, spill 17.4.

## Drift vs witnesses

- stage_commit @256 fastest 0.755 µs/tx sits exactly at the low end of the
  w5-ring-rewitness window 0.755–0.834 (a3-ring-sink witness 0.589–0.603 is
  26–41% below today's numbers; drift consistent with the environment
  attribution already recorded). Target <4 µs/tx USL: MET with ~5x margin.
- chain_full @256 fastest 7.40 µs/tx vs witness 6.19–6.33: +17% to +19.5%
  (median 8.31: +31–34%). Same environment drift direction, milder than the
  +28–42% the stage legs showed between a3 and w5.

## Commit-latency structural split (code cite, not a new leg)

- Warm ring path: `stage_commit_parquet` / `ring_sink_commit` use the warm
  COMMIT_LEASE — chains-sovereign.rs:733-740 records steady-state =
  memcpy into ring window + seal, NO per-call File::create.
- FD-create path: the lease-grant arm at chains-sovereign.rs:326 and
  352-353 (per-process temp dir, one reused name; File::create truncates)
  is the cold FD-create arm. The bench's leg separation IS the comparison;
  no separate FD-create bench leg was fabricated.
- cold-warm delta readable from legs: ring_sink_commit 0.809 µs/tx warm vs
  ring_sink_spill 4.54 ms/256 = 17.7 µs/tx @256 (median) — spill/FD leg
  ~22x warm.

## Receipt-chain integrity

- `cargo test -p cgap-ffi --test lever_courts` — 2 passed, exit 0
  (l4_committed_header_flip_is_exactly_spoliation,
  l4_status_flip_witness_at_lease_edge).
- Tamper battery (crates/cgap-ffi/src/receipt_chain.rs:265-383, `courts`
  module): 5 passed, exit 0 — empty_chain_verifies_as_empty,
  invalid_license_refused_at_append, chain_appends_and_verifies,
  tampered_receipt_or_block_is_typed_refusal,
  lease_exhaustion_is_typed_refusal.

## Zero-alloc on commit legs

Divan printed NO alloc rows for ring_sink_commit or stage_commit_parquet
(divan emits alloc stats only when nonzero): 0 allocs / 0 B as of this run.
Witness pre-P5 was 2 allocs / 36 B — the zero-alloc claim holds improved.

## Non-fabrication note

The 100,000-commit loop does not exist as a bench leg. Witnessed legs are
N=256/4096 batches x 100 divan samples. No 100k leg was invented; per-tx
figures above are divisions of witnessed samples only.

## Compile-freeze incident (disclosed, not my lane's edit)

First bench attempt aborted at compile: crates/cgap-parquet/src/footer.rs:1008
contained injected text `write_i32_field(&pub fn dummy() {}` (unclosed
delimiter, another lane's in-flight corruption). Read-only lane did not
touch it; owning lane repaired within the ~10-min SLA (repaired after 2
1-min polls). Compile-abort is reported as compile-abort, not a failure.

## Verdict

ALIVE. Ring commit path re-witnessed this session: stage_commit 0.755–1.001
µs/tx (< 4 µs USL met, ~4–5x margin), chain_full 7.40–18.2 µs/tx (+17–34%
vs witness, environment-attributed), lease-edge banner reproduced verbatim,
spoliation + tamper courts green, commit legs zero-alloc.
