# V26.11.2 Pareto Levers — Engineering Specs

Charter: CHARTER-CGAP-STARDF-v26.11.2 · repo /Users/sac/cgap · 2026-11-01
Lane: V2-LEVERS · lease: this file + waves/v2-03-levers.md (receipt).

Method: each lever names its v26.11.1 measured receipt coordinate, the
Pareto bottleneck with stage-share evidence, the root cause at file:line
(verified on disk this session), the v26.11.2 fix, and a numeric target.
All targets are TARGETs until a v26.11.2 bench receipt re-measures them
(same method: divan 0.1.21, bench profile, 100 samples, Apple Silicon).

## L1 — Ingress: two-stage digest escalation

- Receipt coordinate: `docs/jira/v26.11.1/waves/bench-chains-ingress.md`
  (P3-16, 2026-11-01, exit 0). Measured: `ingress_chain` 102.7 ms /
  1,048,576 frames = 102.7 ns/frame (10.2 Mframe/s).
- Bottleneck: `stage_digest_fold` 93.38 ms — ~85% of segment cost,
  ~91% of the chain mean. Admission stages are ~15% combined
  (penstock 9.66 ms, A04 project 6.64 ms). ~89 of ~98 ns/frame is
  hashing.
- Root cause: unconditional BLAKE3 payload digest pre-admission for
  every frame; the digest stage pays full-width crypto on the admitted
  subset with no policy gate.
- v26.11.2 fix: two-stage escalation. Fold a cheap 8-byte non-crypto
  prefilter into the dam's `FrameView` (penstock gate first at
  ~5.70 ns/frame dispatch budget; measured penstock admission is
  108.5 Mitem/s ≈ 9.2 ns/item) and escalate to BLAKE3 only on a
  policy flag (first-frame-per-flow or reserved-flag mutation).
- Target: <15 ns/frame chain (vs 102.7 measured; >=6.8x). Digest
  bounded to the suspicious subset; admission stages already at
  108-158 Mitem/s.
- Anti-vacuity mutation: corrupt a slot byte or stage code — the
  per-stage checksum assert (vs untimed preflight fold) must fire;
  escalation must still refuse a spoliated frame.

## L2 — Parquet seal: word-wise SWAR FNV-1a

- Receipt coordinate:
  `docs/jira/v26.11.1/waves/bench-chains-parquet.md` (P3-17, exit 0).
  Measured: `chain_ram` 41.54 ms @ 1,048,576 rows.
- Bottleneck: seal/verify/flush ~35.7 ms — 86% of the chain at 1M
  rows (encode 12%, decode 2%). Each side runs 1 checksum + 4-seed
  `block_digest`: 10 byte-wise FNV-1a passes over ~2 MB per iteration.
- Root cause: `crates/cgap-parquet/src/footer.rs:119-127`
  `fnv1a64_seeded` — `for &b in data { h ^= b; h *= PRIME }`, one
  byte per iteration, called 10x per seal/verify pair.
- v26.11.2 fix: word-wise u64 SWAR FNV in `footer.rs` — process 8
  bytes per step (batched xor + multiply via u64 lanes, byte-exact
  FNV-1a semantics preserved with a carryless bulk body and a byte
  prologue/epilogue for unaligned tails).
- Target: <10 ms @ 1M rows (>=5x on the chain; digest passes are the
  86% segment, codec stages already near-optimal).
- Anti-vacuity mutation: flip one committed window byte before
  `verify_seal` -> `ChecksumMismatch { region: Body }` -> exit 101
  (court witnessed flipping in P3-17; M2 truncation law: decoder is
  span-bounded by the page header, not the row-group span).
- Re-attribution 2026-11-02 (P2-PARETO, receipt
  `waves/p2-pareto-reattribution.md`): the 86% "seal/verify/flush"
  attribution above was STALE — W3-L2's SWAR FNV + the interleaved
  `block_digest` already shrank the digest segment, so the per-stage
  Divan legs were re-run: `digest_arm` 6.05 ms @1M (fnv checksum +
  block_digest over the committed block), `flush_arm` 0.04 ms,
  `encode_arm` 2.57 ms/lane, writer-path residual ~10.2 ms (2 lanes
  through `encode_hybrid_into` + framing, ~2x the standalone
  `rle_encode` leg). ONE landed lever: fused `SealDigests` sweep —
  one read pass over the body feeding 6 FNV states (body checksum,
  trailer checksum, 4 block_digest lanes) where `seal()` previously
  made 3 passes. chain_ram 24.02 -> 23.21 ms @1M (-3.4%); seal_arm
  17.22 -> 16.34 ms. <10 ms target NOT met by this lever. Dominant
  stage now: the writer encode path (mmap_writer.rs `encode_hybrid_
  into` literal-loop) — the next honest lever target. Mutation court:
  one-bit xor flip inside `SealDigests::absorb` -> 2 seal courts FAIL
  (FLIPPED), restore sha256-exact
  `3b01b989d32b9cdc6806bdcbfb7a7421b09c631b3d03b2a974fb356d6746ba7a`,
  tests green after restore.

## L3 — BN3 unify: static-footer bit-slice semi-join

- Receipt coordinate:
  `docs/jira/v26.11.1/waves/bench-chains-bn3.md` (P3-18, exit 0).
  Measured: `deduction_chain` 1.470 s @ 1,048,576; `stage_unify_join`
  1.472 s — 81.2% of the chain at 65,536, degenerating to 99.9% @ 1M.
- Bottleneck: the unify probe is a nested-loop join — probe work is
  `sum(cur_len x cand_len)` pairs; it scales O(N^2)-shaped in probe
  pairs while every other stage scales with bytes at line rate
  (scan 12.3 GB/s, project 2 ms).
- Root cause: nested-loop unify probe (`cur` x `cand` per chunk) —
  spec 3.2 v1 strategy, `crates/cgap-bn3/src/unify.rs:11,35-39,274`
  (join probe documented as nested-loop with mask AND, v1 choice).
- v26.11.2 fix: PLNF static-footer bit-slice semi-join keyed on the
  bound (s,o) word pair, with a noalloc scratchpad radix probe
  (`noalloc.rs` scratchpad, the W2-9 deferred arm). Collapses probe
  work from O(cur x cand) to O(cur + cand). The raw-word hash-join
  oracle already inside the P3-18 bench computes the same match count
  in setup time — the shape is demonstrated on the measured corpus.
- Target: >20x @ 1M rows (chain 1.470 s -> <75 ms; unify off the
  Pareto frontier).
- Anti-vacuity mutation: binding-alias comparator swap (doe-4
  SHIM-DOE4 class) — the differential oracle assert (match count ==
  raw-word O(N) hash join, no unify code in the oracle path) must
  flip; M12-7 source_mask popcount assert rides along.

## L4 — Sovereign commit: pre-leased ring sink

- Receipt coordinate:
  `docs/jira/v26.11.1/waves/bench-chains-sovereign.md` (P3-19, exit 0).
  Measured: 45.8 us/tx chain (187.6 ms / 4096 tx).
- Bottleneck: `stage_commit_parquet` 164.4 ms — 87.6% of the chain
  (admit 1.8%, eval 7.7%, verify 2.0%). A real `File::create` +
  write per call.
- Root cause: `crates/cgap-ffi/src/sovereign.rs:706-730` — per-call
  `std::fs::File::create(path)` (O_TRUNC; call site at line 722)
  plus a fresh boxed `FileSink` scratch alloc inside
  `cgap_commit_parquet`.
- v26.11.2 fix: pre-leased ring sink inside the 65,376-lane commit
  lease (P3-19 LEASE_EXHAUSTED boundary finding: lane 65,377 refuses
  typed 0x43470004). The sink is leased once per lane slot at lease
  grant, reused across transactions — no per-call open/alloc; spill
  to a real file only on lease rotation.
- Target: <4 us/tx chain (vs 45.8 measured; >=11x). Commit pays
  memcpy + seal only in the steady state.
- Anti-vacuity mutation: corrupt one committed header byte (refusal
  class) or drop a slab write — per-stage `CgapStatus` asserts and
  the P28 spoliation negative control (one flipped digest byte ->
  exactly CGAP_ERR_P28_SPOLIATION) must fire.

## L5 — Seal chain: fused ASC450 metering

- Receipt coordinate:
  `docs/jira/v26.11.1/waves/bench-chains-seal.md` (P3-19/BENCH-4,
  exit 0). Measured: 2.37 us/row three-way, flat tail (2.378 vs
  2.373 us/row, <=0.3% drift across 1024 -> 65536).
- Bottleneck: three-way split, no single 80% stage — ASC450 metering
  ~45.4% (residual method), chatman seal+verify 35.2%, P28 §5
  validate 19.3%.
- Root cause: sequential P28 validate -> ASC450 metering ->
  chatman seal passes; each pass re-walks the attestation span, so
  the metering pass pays a second span traversal plus its own
  SignedProof state machine walk.
- v26.11.2 fix: fuse the ASC450 metering into the P28 §5 span fold —
  meter state advances in the same single ordered pass that
  validates the 80 B attestation span; chatman seal remains a
  distinct tail stage (its crypto roundtrip is irreducible).
- Target: <1.45 us/row (>=1.6x; removes the second span walk, the
  ~45% meter residual collapses into the 19% validate pass).
- Anti-vacuity mutation: bit-flip an upstream receipt (validate must
  refuse), demote a crossing (no proof emission — `proven == N/2`
  assert flips), flip a receipt digest (verify DigestMismatch).
- Re-measure 2026-11-01 (W3-L5, receipt
  `waves/w3-l5-fused.md`): `fused_row` landed in cgap-p28 (anchor
  match + folded record/seal, courts green: 37 lib tests + 2
  lever_courts exit 0; demote-court FLIPPED, restore sha256-exact)
  but the fused bench arm measured 3.04 µs/row @65536 vs 2.37 control —
  the fuse ADDS the §4.4-3 sealed-state mint on licensed rows and the
  one-lineage-per-batch mechanism is refused by the latch law for the
  mixed corpus. <1.45 µs/row target NOT met; lever gate FAILED-BY-
  MEASUREMENT. Re-PARETO: the irreducible cost is keyed-BLAKE3
  roundtrips per row, not redundant traversal.

## Target summary (all TARGETs until re-measured)

| lever | measured | target | floor |
|---|---|---|---|
| L1 ingress | 102.7 ns/frame | <15 ns/frame | >=6.8x |
| L1 re-measure 2026-11-01 (W3-L1, receipt `waves/w3-l1-ingress.md`): two-stage escalation landed (FrameView prefilter/escalate, ESCALATE_MASK row-6 family, stage3_escalated + stage_prefilter_fold legs); ingress_chain 10.69 ms @1M = 10.20 ns/frame fastest (median 10.30) — <15 ns/frame MET, 10.1x vs >=6.8x floor; gates 11+3 courts exit 0 both profiles; escalate-always-true mutant FLIPPED, restore sha256-exact. Digest leg vacuous-zero (escalated subset empty on this corpus — receipt-recorded). | | | |
| L1 reclassification 2026-11-01 (P2-PENSTOCK, receipt `waves/w4-p3-escalation-corpus.md`): ESCALATE_MASK (penstock.rs:271, 0x00F0 on header byte 5) is the SAME predicate as stage-1's FLAG_RESERVED_MASK (frame.rs:38, 0xF0) over all 256 flag values — escalated-AND-chain-admitted is structurally impossible, so L1's 10.20 ns/frame (10.1x) is **skip-not-pay**: the digest stage is a prefilter fold ALWAYS on chain-admitted frames, and the escalated BLAKE3 leg is permanently vacuous at chain level (law, not corpus). LeverStatus -> LANDED-skip-not-pay. Repair requires an A04 §6.3 wire-law amendment (split the reserved family; A19 register candidate) — recorded UNBUILT, out of lane ownership. | | | |
| L2 parquet | 41.54 ms @ 1M | <10 ms @ 1M | >=5x |
| L2 re-measure 2026-11-01 (W3-L2, receipt `waves/w3-l2-swar-fnv.md`): SWAR FNV landed + equity green (222 tests exit 0), chain_ram/chain_file @1M 42.7-43.1 ms mean — chain did NOT move; <10 ms target NOT met by this lever (chain 24.02 ms measured this lane; FNV share smaller than planned). | | | |
| L2 re-attribution 2026-11-02 (P2-PARETO, receipt `waves/p2-pareto-reattribution.md`): fused `SealDigests` single-sweep seal (6 FNV states, 1 body pass vs 3) landed byte-exact; chain_ram 24.02 -> 23.21 ms @1M (-3.4%), seal_arm 17.22 -> 16.34 ms; 223 cgap-parquet tests exit 0; mutation FLIPPED (2 seal courts FAIL), sha256-exact restore; <10 ms target NOT met — dominant stage is now the writer encode path (`encode_hybrid_into`, mmap_writer.rs), recorded as the next lever target. | | | |
| L3 bn3 | 1.470 s @ 1M | <75 ms @ 1M | >20x |
| L3 re-measure 2026-11-02 (W5-L3, receipt `waves/w5-l3-remeasure.md`): both join legs measured (divan, /tmp/a1-target, chains-bn3.rs): `stage_unify_join` (nested, pre-L3 baseline leg) 1.674 s fastest / 1.701 s median @1M (8.435 ms @65k), `stage_unify_join_radix` 1.648 s fastest / 1.656 s median @1M (7.661 ms @65k) — **radix:nested @1M = 0.98x fastest (0.97x median)**, ~1.6-2.6% faster. <75 ms @1M NOT met (22x over target); >20x floor NOT delivered. Full-chain implication: the join stage IS the 1.470 s baseline (stage ~ chain, both ~1.5-1.7 s), so the radix semi-join as built does NOT move the chain — L3 lever LANDED-NOT-MET; dominant cost is the per-chunk candidate walk outside the probe (next-lever attribution required). Alloc column: divan emitted no Alloc/Bytes column on any leg = 0 allocs across all samples (divan 0.1.21 omits all-zero columns; corroborated by `l3_noalloc_radix_*` courts). lever_courts 7 exit 0 (34.88 s). Board flip `p:measuredNs` is a coordinator followup (generated ontology, not hand-edited). | | | |
| L4 sovereign | 45.8 us/tx | <4 us/tx | >=11x |
| L4 re-measure 2026-11-01 (W3-L4, receipt `waves/a3-ring-sink.md`, witness re-run A3 2026-11-01): RingSink landed (mmap_writer.rs, BlockSink on `&mut RingSink`, spill-on-rotation, over-budget/failed-drain typed refusal) + FFI commit leg swapped to the thread-local `COMMIT_LEASE` (spill path resolved at grant; signature unchanged; 65,376/65,377 -> 0x43470004 probe verbatim). Measured (divan, /tmp/w3-l4-target): `stage_commit_parquet` (the FFI ring path) 269.1 us @256 / 4.368 ms @4096 = **1.05-1.07 us/tx** vs 45.8 recorded — <4 us/tx MET, >=43x. `ring_sink_commit` steady leg 262.1 us @256 = 1.02 us/tx; `ring_sink_spill` 6.46 ms @256 (fat slabs, forced rotation). Alloc column: 2 allocs/36 B per tx REMAIN (FooterSealer `spans.to_vec()` + `build_meta_block` Vec, footer.rs:187/326 — pre-existing, outside lane ownership; NOT 0). Courts: lever_courts 2 exit 0 (header flip -> exactly 0x5032_38FF both ways; 70k-lane LEASE refusal); commit_positive byte-identity court adjusted to rotation semantics (concatenated owner-API blocks, per-lane-slot determinism), disclosed. | | | |
| L4 re-witness 2026-11-01 (A3, `waves/a3-ring-sink.md`): independent re-run, /tmp/a3-target. `stage_commit_parquet` 150.7 us @256 = **0.589 us/tx** / 2.472 ms @4096 = **0.603 us/tx** — <4 us/tx MET, >=76x vs 45.8. LEASE probe verbatim: 65,376 accepted, 65,377 -> 0x43470004. Alloc column NOT 0: 2 allocs/36 B per tx remain (footer.rs:187/326, pre-existing). lever_courts 2/2 exit 0; full cgap-ffi + cgap-parquet suites exit 0. | | | |
| L5 seal | 2.37 us/row | <1.45 us/row | >=1.6x |
| L5 re-measure 2026-11-01 (W3-L5, receipt `waves/w3-l5-fused.md`): fused_row landed + courts green (37+2 tests exit 0, demote-court FLIPPED/restore-exact), but fused arm 3.04 µs/row @65536 vs 2.37 control — REGRESSION; <1.45 µs/row target NOT met. | | | |

Court plan for every lever: divan bench receipt on the same host +
named-mutation flip in a /tmp sandbox with sha256 fail-closed
restore. A lever whose court cannot flip is FAIL; every skip is
loud. Perf numbers are TARGETs until a v26.11.2 bench receipt
re-measures them with the stated method.

