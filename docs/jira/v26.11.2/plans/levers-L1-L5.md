# Plans — Pareto levers L1-L5 (W3 execution specs)

Provenance: 16-lane Plan wave (read-only, on-disk-grounded) + 20-lane Explore wave. Every
claim cited to the source lane's on-disk reads; re-read at use time.

## L1 — two-stage admission escalation (target <15 ns/frame)

Source: crates/cgap-gate/src/penstock.rs (1719 ln; `ingress` :491, fixed row order
magic→OABI→P28, zero-copy FrameView :454); crates/bench/benches/chains-ingress.rs
(chain_frame/stage1-3 :133-184, build_pool preflight :406, stage_digest_fold :591);
lever spec docs/jira/v26.11.2/02-pareto-levers.md:12-28.

1. Extend FrameView: `pub prefilter: u64` + `pub escalate: bool`; `ESCALATE_MASK: u16`
   (reserved-flag bit, row-6 family); `#[inline] fn prefilter8(bytes)` — one unaligned LE
   load + wrap-mul mix. `ingress` signature unchanged; escalate=true iff admitted AND
   (att_flags bit OR first-seen-flow — flag-only first, no state table; gate stays
   side-effect-free by law). Doc-comment row-6 divergence.
2. Bench: stage3 → stage3_escalated(view); chain fold = a04 + st + (escalate ? digest :
   prefilter); preflight records digest_fold (escalated subset) + prefilter_fold; add
   stage_prefilter_fold bench. Keep ingress_chain consts/ItemsCount identical.
3. Court: flip one byte at OFF_ATTESTATION+6 on an admitted frame → ingress must flip
   CGAP_OK → CGAP_ERR_P28_SPOLIATION; assert chain checksum differs AND escalation still
   refuses (no BLAKE3 paid on spoliated frame).
4. ggen surface: ontology individual cgap:IngressPrefilterLever in the cgap-pack graph;
   one new cli verb `gate dam escalate-policy` via crates/cli ontology → generated routing,
   policy logic in custom_handlers.rs (cnv:CustomBehavior).
5. Receipt: waves/l1-ingress-escalation.md (before/after table, mutation flip, alloc column).
Risk: FrameView is Copy w/ FFI twin divergence already recorded — additive fields safe;
build_pool cross-check must move to the new fold formula in the same commit.

## L2 — word-wise SWAR FNV (target <10 ms @1M)

Source: crates/cgap-parquet/src/footer.rs — `fnv1a64_seeded` :119-127 is the sole hotspot
(5x per seal :276/:285, 5x per verify :476/:477/:486); fnv1a64 :114 and block_digest :130
untouched callers; no external callers. Constants frozen: basis 0xcbf2_9ce4_8422_2325, prime
0x0000_0100_0000_01B3 — SWAR changes loop structure only.

Design: byte-exact FNV-1a preserved (all courts pin checksums byte-for-byte). Prologue
byte-wise over len % 8; bulk body per 8-byte LE word: unrolled per-lane
`h ^= b_k; h *= P;` (b_k = (w >> 8k) & 0xFF) — word-wise loading, 1 load/8 bytes, in-register
unroll; NOT the fancy 8-lane-multiply SWAR (error-prone; FNV's per-byte multiply dependency
chain limits it anyway). Epilogue tail bytes. Equity #[test]: SWAR == reference byte-wise
over empty/1..16/4K+1 corpora, pinned to pre-change fixture digests.

Mutations: M-A flip one committed-window byte pre-verify_seal → ChecksumMismatch{Body} → 101
(witnessed P3-17); M-B corrupt one SWAR lane (shift 8*3+1 or drop one h*=P) → equity court
assert fires AND hostile_sealed_block_corpus sweep (:959-966) fails. Both must flip.

Bench: chains-parquet needs no structural change (chain_ram/chain_file measure the chain,
:207/:346; final asserts :250/:358); optional seal_arm isolating write_slab+seal+verify_seal
for direct attribution. Receipt: baseline 41.54 ms@1M, target <10 ms, ≥5x gate per
02-pareto-levers.md:136; cargo test -p cgap-parquet exit 0 (semantics preserved).

## L3 — PLNF semi-join + radix probe (target >20x @1M)

Source: crates/cgap-bn3/src/unify.rs:793-880 — atoms 1.. build cand_rows over ALL rows
(:822-837) then O(cur × cand) bound-slot probe (:859-875); benches/chains-bn3.rs:142-148
join rule; bitslice.rs:348 semi_join_masks, :379 semi_join_two_hop (which heap-allocates two
mask vecs :398-399 — violates noalloc).

1. `pub struct JoinProbeScratch` — caller-leased (mask_s Vec<u64>, cand_groups
   Vec<(u32,u32)>); `unify_batch_into(..., probe: &mut JoinProbeScratch)`; unify_batch wraps
   Default (zero API break).
2. `pub fn emit_join_masks(cols, join_atom, bound_keys, out_masks) -> u64` — point-range
   bit-slice scan semantics (scan_range_vertical, w=56 payload window) + semi_join_masks
   algebra vendored into unify.rs (~30 lines, cite bitslice.rs source line; NO new
   cgap-parquet dep — cgap-bn3 stays zero-external-dep).
3. Radix probe: 2^4=16 buckets by low bits of joined key word; bucket row-index Vec<u32> in
   leased scratch, counts precomputed per corpus; probe loop O(cur + cand + hits). High-water
   marks mirror hw_rows/hw_vars (:596-604).
4. noalloc mirror (noalloc.rs): fixed-capacity radix tables, NaError::MatchCapacity loud
   refusal (:48-68 pattern); ALLOC arm remains semantics authority (:8-10); differential
   court extends to radix path.

Mutations: M-L3-1 drop the radix probe → stage_unify_join ns @1M must move ≥10x (target
>20x; if <10x the join wasn't the 81.2% — re-PARETO before landing); M-L3-2 alias comparator
(register NUMBER not bound VALUE, unify.rs:560-563 doe-4 class) → oracle-count asserts
(chains-bn3.rs:286-305) flip. Differential court: radix path match-for-match identical to
nested-loop over CourtRng corpus at N ∈ {1, 64, 4096, 100_003}.

Bench: add stage_unify_join_radix (consts [65_536, 1_048_576]), same corpus/asserts
(:409-418) + probe_masks popcount assert. SEED LAW: bench CANONICAL_SEED =
0xBF58_476D_1CE4_E5B9 (:33); 0xA20C_0FFE_E5A7_E000 is the bitslice corpus-spread XOR constant
(:687-704), NOT a seed — never port into bench code.

## L4 — pre-leased ring sink (target <4 µs/tx from 45.8)

Source: crates/cgap-ffi/src/sovereign.rs:706-730 — per-call Vec lane copy (:~709), boxed
FileSink scratch vec![0u8; sealed.len()] (:724), std::fs::File::create O_TRUNC + write_all +
flush per call (:722; mmap_writer.rs:674-700). That is the 87.6%.

1. `RingSink { ring: Box<[u8; 65_536]>, head, wraps, spill: Option<std::fs::File> }`
   implementing BlockSink (mmap_writer.rs §7.3 seam — same crate as BlockSink; window returns
   &mut ring[head..head+len]; commit advances head/wraps). Lease-granted per lane slot;
   steady-state per-tx = memcpy + writer.seal. StaticPageBuffer::<65_536> still gates slab
   capacity — P3-19 boundary at lane 65,377 / status 0x43470004 preserved byte-for-byte.
2. Spill on rotation: drains ring → pre-opened handle (opened once at lease grant; never
   per-call File::create). IO failure keeps CGAP_ERR_LEASE_EXHAUSTED mapping + RECORDED
   MAPPING DIVERGENCE receipt note. New ring refusal: spill exceeding ring budget →
   CGAP_ERR_LEASE_EXHAUSTED (never silent truncation). FFI signature unchanged — path
   becomes rotation spill target resolved at lease grant. No new exported symbols → no ABI
   re-litigation.
3. Mutations: A flip one committed sealed-block header byte in the ring → verify leg returns
   exactly CGAP_ERR_P28_SPOLIATION; B skip one slab write → per-stage CgapStatus assert in
   chain_corpus flips (chains-sovereign.rs:269-276 pattern). #[cfg(test)] courts in
   sovereign.rs.
4. Bench: keep lease_boundary_probe bisect verbatim; add ring_sink_commit (steady, no spill)
   + ring_sink_spill (forced rotation) legs; steady-state leg must show zero heap allocs in
   Divan alloc column. Target recorded ≤4 µs/tx, not CI-asserted.
5. Receipt: waves/ PARETO (µs/tx, alloc column, LEASE_EXHAUSTED probe, flip evidence);
   update 02-pareto-levers.md L4 row.

## L5 — fused ASC450-into-P28 fold (target <1.45 µs/row from 2.37)

Source: benches/chains-seal.rs:218-275 per-row redundancies: fill_splitmix 32 B lease id
(:231-232), Lease::new + LeaseEpochSealer::new per row (:233-237), Asc450Meter::new re-runs
validate_intrinsic (asc450.rs:699) + fresh blake3 of the 80 B wire (:223 already proved),
two lease.current_tick() calls under Mutex.

Design: `fused_row<S: ProofSealer>(meter, rec, amount, crossing, tick) -> Result<FusedOutcome,
Asc450Error>` in crates/cgap-p28/src/asc450.rs (kernel stays zero-dep). One meter lineage per
batch via Asc450Meter::recover from previous SealedMeterState (:724; §4.4-3 path — recovery
verifies fail-closed so chain law holds); §5-validated attestation_digest fed through
recover's ceiling/digest match (:735-739 doubles as §5↔§4 anchor check); counter accumulation
(plan_event/commit_event :789-794) folded in the same loop body as receipt from_root
chaining; current_tick() hoisted (meter never reads time, §4.3). FusedOutcome =
{attestation_digest, proof: Option<SignedProof>, sealed: SealedMeterState}.

Mutation: demote one crossing row's amount to ≤ ceiling → proven drops below N/2,
assert_eq!(proven, N/2) fires, DeltaS.admissions falls back to attestation anchor → prev_root
and folded checksum both diverge — verdict differs, not just a counter.

Bench: governance_chain_fused_per_row (consts [1024, 65536]) reusing corpus/final-state
asserts; keep unfused arm as control for PARETO split. Frozen pins untouched:
urn:avec:license:p28-sil:v2.1.0-sovereign, LICENSE_MAGIC, flag bits, 80 B layout,
attestation offset table (attestation.rs frozen-layout tests :752/:781 tripwire);
SAFE_HARBOR_CEILING_MAX_CENTS semantics unchanged. Unit: fused output byte-equals unfused
per-row path on mixed corpus. Verify <1.45 µs/row at N=65536 before landing; UNVERIFIED
until run.
