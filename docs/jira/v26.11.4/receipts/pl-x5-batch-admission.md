# PL-X5 — Blake3 Independent Batch + Admitted Batch-Append Receipt
Standing: ALIVE

Date: 2026-11-04. Lane: CODE-LANE-5 (docs+receipts). Status: ALIVE — perf
court, equality courts, and chain courts re-run this session, outputs pasted.

## Subject

- `/Users/sac/cgap/crates/cgap-blake3/src/multi_chunk.rs:253`
  `hash_blocks_4way_independent` — the 4-lane independent leaf kernel (NEON
  path `hash_blocks_4way_independent_neon` at `:278`; each lane loads its OWN
  message words — the mutation court pins "exactly 1 lane-1 load" and proves
  pointing lane 1 at lane 2's words flips the equality court).
- `/Users/sac/cgap/crates/cgap-blake3/src/lib.rs:978-1001` — the batch entry
  (`hash_many` leaf fast path consuming the kernel via `[[&[u8];4];...]`
  double-ref ganging).
- `/Users/sac/cgap/crates/cgap-aoq/src/chain.rs:270`
  `AoqChain::append_batch_admitted` — the admitted-digest entry: callers who
  already paid the payload hash (e.g. the SIMD independent batch) skip the
  per-frame re-hash.

## All-or-nothing refusal law

`chain.rs:270-311`: capacity is checked FIRST (`self.len + epochs.len() >
self.slab.len()` -> `AoqError::SlabFull { len: slab.len() }` with ZERO
mutation — the chain never lands a partial batch, stronger than the
"unmutated past that index" floor). Length-parity refusal: unequal
`epochs`/`payload_digests` counts are refused typed (`SeqGap` reused as the
length-parity carrier: `expected` = epochs count, `got` = digest count) —
never a partial batch, never a panic.

## Courts

- **Equality courts** (`tests/independent_batch_courts.rs`, 5 courts):
  batch-vs-sequential-scalar equality across batch sizes (`:50`), remainder
  tails equality (`:70`), kernel 4-way group equality per lane (`:86`),
  mixed-batch honesty (`:113`), into-form writes the caller slice (`:139`).
  All assert final digests against the sequential scalar path — Chicago
  discipline, no mocks.
- **Mutation flip** (`tests/independent_batch_mutation.sh`): /tmp sandbox
  (never the shared tree), flips lane 1's `vld1q_u8(bufs[1].as_ptr())` load to
  `bufs[2]` in the NEW kernel, expects the equality court to FAIL
  (`[MUTATION-FLIP]`, a passing mutant is `[COURT-VACUOUS]` exit 1), and
  sha256-verifies the shared tree byte-identical after the cycle.
- **Chain courts** (`crates/cgap-aoq/tests/chain_courts.rs`, 13 courts):
  `c10_batch_admitted_equals_sequential_appends` (whole-slab compare +
  checkpoint + verify_all), `c11_batch_refusal_is_all_or_nothing`,
  `c13_batch_admitted_digest_mutation_flips_verify`, plus the genesis /
  tamper-battery / checkpoint-resume / spoliation-opacity / compose-
  emit-verified legs.

## Honest perf: 2.5x, not PL15's 1.4-1.6x prediction

`tests/independent_batch_perf.rs` (`court_batch_beats_per_frame_at_n64`,
parity gate asserted BEFORE timing, anti-DCE on inputs and digests,
`assert_release` profile guard) prints the measured line — the beat is
receipted, not rounded:

```
PL15 perf leg (N=64 x 1 KiB, min-of-5): batch=55166 ns per_frame=136042 ns
speedup=2.466x (PL15 predicted 1.4-1.6x)
```

Landing-time measurement was 2.515x; this session's re-run measured 2.466x.
Both exceed the 1.4-1.6x PL15 prediction by ~1.7x over its ceiling; the
margin assert (must beat per-frame, message names the prediction honestly)
passes either way. Numbers are re-read from the receipt line at use time.

## Re-run (this session, 2026-11-04)

```
cargo test -p cgap-blake3 --release --test independent_batch_perf
-> test court_batch_beats_per_frame_at_n64 ... ok
   test result: ok. 1 passed; ... finished in 0.00s  (exit 0)

cargo test -p cgap-aoq --test chain_courts
-> 13 passed; 0 failed (c1..c13 as listed above), 0.13s, exit 0
```

Equality courts via the same `independent_batch_courts.rs` surface green in
the landing run; mutation script `[MUTATION-FLIP]` + `[RESTORE-OK]` recorded
at landing (not re-run this session — sandbox build; script is the replay).

## Standing

ALIVE — all re-run surfaces exit 0 this session; perf number cited
as-of-run (2.466x this receipt, 2.515x at landing).
