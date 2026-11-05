# AB-19 — Slab Stress (STRESS lane, v26.11.3)
Standing: UNVERIFIED

- **Subject**: `crates/cgap-arrow/src/slab.rs` + `crates/cgap-arrow/src/{bitmask,blake3}.rs` (read-only; zero lines changed in src/).
- **Lane file**: `crates/cgap-arrow/tests/slab_stress.rs` (new, 5 courts).
- **Standing**: see Verification Ladder below; src/ untouched, so the suite cannot regress by construction of the diff — but full-suite gate run as well.

## Surface adaptation (contract -> real lifecycle)

The slab is a fixed-capacity record slab (`64 + CAP*64` bytes, inline
storage) with lifecycle push -> seal -> decode and reset-to-`new()` as the
only release primitive. There is no per-row `free()`, no handle table, and
no fragmentation in the allocator sense. Deliverables mapped onto the real
surface:

| Contract item | Surface mapping |
|---|---|
| capacity stress: fill -> refusal -> frees -> refill, 1k cycles | fill 64 -> `SlabFull` (state byte-frozen) -> reset (`Slab64::new()`) -> refill, x1000, seal+decode identity per cycle |
| fragmentation: 17 orderings x 100, free-count == alloc-count | 17 deterministic LCG interleavings of {push, too-long push, seal, reset, push+seal double-op} x 100 iterations; success census == `len()` at every step (nothing lost) and byte image stable run-over-run |
| double-free / use-after-free of handles -> typed refusal | sealed-state refusal battery: double `seal`, push-after-seal, 1k repetition, decode-reacquired object, tampered windows -> exact `SlabError` variants, slab byte-frozen |
| alloc census: refusal paths allocate 0 | thread-local counting global allocator; per-site delta == 0 (SlabFull, push-after-seal, double seal, too-long, BadLength) + 10k-refusal soak delta == 0; live-Vec positive control |
| throughput: alloc/free pairs/sec, min-of-3 | fill+reset cycles/sec over Slab64, min-of-3, conservative 100k/s floor |

## Courts (all Chicago: real slab, real allocator, real byte images)

1. `capacity_stress_1k_fill_refuse_reset_refill_cycles` — 1,000 cycles:
   exact `SlabFull` refusal with the slab byte-identical after refusal,
   reset+refill clean, sealed image decodes with structural identity every
   cycle. Mutation: weaken the `len >= CAP` gate in `push` -> the
   exact-variant + state-frozen asserts flip.
2. `fragmentation_17_orderings_x100_image_and_census_stable` — 17 orderings
   x 100 iterations; census drift or byte-image drift on any iteration
   flips the court; per-ordering refusal-presence assert (each ordering
   observed at least one refusal). Mutation: advance `len` on a refused op
   -> census assert flips.
3. `sealed_state_refuses_every_mutation_typed_and_frozen` — double seal,
   push-after-seal, 1k repetition, decode-reacquired object, tampered
   windows -> exact variants; slab equal-to-frozen after the battery.
   Mutation: delete the `is_sealed` early returns -> variant asserts flip.
4. `alloc_census_refusal_paths_allocate_zero` — per-site delta == 0,
   10k-refusal soak delta == 0, positive control (live Vec moves the
   counter by exactly 1). Non-vacuity: the positive control — a counter
   that never moves cannot pass. Mutation: any alloc introduced on a
   refusal path flips the site delta assert.
5. `throughput_push_reset_pairs_min_of3` — min-of-3, floor 100k/s.
   Mutation n/a (measurement leg; floor assert catches gross regression).

## Verification ladder (commands + exits run this session, exact subject)

- `cargo test -p cgap-arrow --test slab_stress` — exit 0, `5 passed; 0 failed` (first compile run).
- Same command (gate 1, `--nocapture`): exit 0, 5 passed, throughput
  MIN-OF-3 = **12,204,201 pushes/sec** (runs: 18.78M / 12.20M / 20.64M).
- Same command (gate 2): exit 0, `5 passed; 0 failed` (2.91s). GATE x2 MET.
- `cargo test -p cgap-arrow` (full suite, isolated binary re-run pass): exit 0
  (`EXIT=0`, /tmp/suite_ab19.log) — 15/15 `test result: ok` across 12 test
  binaries: 145 lib + 3 bitmask_swar + 5 ipc_body + 17 ipc_courts + 4
  ipc_stress (54.3s) + 36 mutation_kills + 1 property + 5 **slab_stress
  (new, 2.78s)** + 4 stream_writer_stress + 7 validation-adjacent. Suite
  UNREGRESSED.
- Note: one earlier full-suite pass showed a one-off failure in the AB-10
  `stream_writer_stress` binary; re-run in isolation 4/4 exit 0 (0.80s) and
  green again in the final full-suite pass — parallel-suite flake in a
  foreign lane's UDS/lsof court, not this lane's diff (this lane's diff is
  one new standalone test file; src/ untouched).

## Falsifiers

- Capacity gate: weaken `push`'s `len >= CAP` gate -> capacity court flips
  at the CAP+1-th push.
- Fragmentation gate: advance `len` on refused op -> census flips.
- Sealed-state gate: delete the `is_sealed` early return -> variant +
  freeze asserts flip.
- Alloc-census gate: any allocation on a refusal path -> site delta assert
  flips; dead counter -> positive control flips.
- Throughput leg: gross throughput collapse -> floor assert.

## Notes

- Lane restarted once (coordinator 429 kill after slab.rs read); resumed
  with the coordinator's confirmed adaptation.
- No git commands run (lane law).
