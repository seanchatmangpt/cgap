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

## Target summary (all TARGETs until re-measured)

| lever | measured | target | floor |
|---|---|---|---|
| L1 ingress | 102.7 ns/frame | <15 ns/frame | >=6.8x |
| L2 parquet | 41.54 ms @ 1M | <10 ms @ 1M | >=5x |
| L3 bn3 | 1.470 s @ 1M | <75 ms @ 1M | >20x |
| L4 sovereign | 45.8 us/tx | <4 us/tx | >=11x |
| L5 seal | 2.37 us/row | <1.45 us/row | >=1.6x |

Court plan for every lever: divan bench receipt on the same host +
named-mutation flip in a /tmp sandbox with sha256 fail-closed
restore. A lever whose court cannot flip is FAIL; every skip is
loud. Perf numbers are TARGETs until a v26.11.2 bench receipt
re-measures them with the stated method.

