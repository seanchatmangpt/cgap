# ARCHITECTURE

System-topology index for the cgap sovereign columnar/deductive substrate
(23-crate root workspace, v26.11.2 starDF campaign). This file is the
INDEX, not the duplicate: the C4 model (context/container/component/L4
radix semi-join flowcharts) lives in
`docs/jira/v26.11.2/dflss/17-c4-architecture.md`; deep specs live in the
docs named in the pointer index below. Every claim cites file:line or a
receipt; performance figures carry register-row cites; UNVERIFIED never
rounds up.

## 1. Crate dependency DAG (tier-by-tier)

Derived from `cargo metadata --no-deps` run this session (exit 0). Edge
= declared path dependency. 22 root members plus standalone crates
(own `[workspace]`, not root members): `cli`, `csct`, `cgap-fleet-ops`,
`cgap-ocel2`.

Tier 0 - zero-dep kernels (no path deps):

    cgap-blake3   (0 deps)
    cgap-branchless (0 deps)
    cgap-bosym    (0 deps)

Tier 0.5 - kernels whose only path deps are Tier 0:

    cgap-gate     -> branchless
    cgap-crypto   -> blake3
    cgap-bn3      -> blake3
    cgap-bont     -> blake3
    cgap-parquet  -> arrow (arrow -> core/gate/storage; see Tier 1)
    cgap-git      -> blake3, bn3, parquet

Tier 1 - moonshot lane crates:

    cgap-core     -> blake3, gate
    cgap-storage  -> blake3, git
    cgap-arrow    -> core, gate, storage
    cgap-p28      -> blake3, crypto

Tier 2 - membranes:

    cgap-ffi      -> arrow, audit, blake3, bn3, core, gate, p28, parquet
    audit         -> arrow, parquet
    cgap-beam     -> arrow, blake3, git, parquet
    cgap-gpu      -> arrow
    cgap-integration -> audit
    cgap-court    -> audit, blake3, bn3, core, crypto, ffi, gate, p28,
                     parquet, storage
    bench         -> (11 of 12 crates; divan harness)

Tier 3 - standalone host-side crates:

    cli           -> (ggen six-pack generated; own workspace)
    csct          -> blake3 (receipt.rs)
    cgap-fleet-ops -> (standalone; fleet register + SLO surface)
    cgap-ocel2    -> (standalone; receipt-chain OCEL 2.0 projection)

Note: the DAG above is the manifest truth (`cargo metadata`); the
fresh-run boundary-gate E-count table (0 runtime externals across 15
covered crates) lives in
`docs/jira/v26.11.2/dflss/17-c4-architecture.md:11-29` and is NOT
duplicated here.

## 2. Memory-hierarchy dataflow (ingress -> projection)

The hot path is a fail-closed pipeline; every stage either admits or
returns a typed refusal. One 128-byte A04 `CgapFrameHeader` frame
(`docs/jira/v26.11.1/lanes/` A04 spec) flows:

1. **pkt-line / pack ingress.** Packfile V2 streaming demuxer
   `crates/cgap-git/src/pack_demux.rs:1-24` - zero-alloc header-walk
   state machine; typed refusals via `DemuxError`
   (`pack_demux.rs:58`); bodies carried as stored frames of
   `declared_size` with no payload copy (header-walk facet,
   `pack_demux.rs:27`).
2. **Ingress ring staging.** 32,768-byte circular ring
   `crates/cgap-git/src/ingress_ring.rs:1-21` - house-pattern reuse of
   `cgap-core/src/dma_ring.rs` cache-line-padded indices
   (`CachePaddedIndex`), free-running counters, `&mut self` SPSC;
   full-capacity ring (no slot sacrificed); `#![forbid(unsafe_code)]`.
   Typed refusal-quota seam `RefusalQuota`
   (`ingress_ring.rs:114`, default 4,096 sustained refusals/window,
   `ingress_ring.rs:134`) - observational only, trips only after
   explicit `acknowledge` (`ingress_ring.rs:183`) or the sticky
   level-triggered auto-clear (`CLEAN_ADMISSIONS_TO_CLEAR`, :139).
3. **Penstock admission (six-row fail-closed taxonomy).**
   `crates/cgap-gate/src/penstock.rs:504-511`: row 1 truncated header
   -> `INVALID_MAGIC`; row 2 magic/version -> `INVALID_MAGIC`; row 3
   digest/length -> `OABI_MISMATCH`; row 4 attestation magic != `P28S`
   -> `P28_SPOLIATION`; row 5 attestation laws -> `P28_SPOLIATION`;
   row 6 reserved flag bits -> ADMITTED with the divergence note at
   `FLAGS_RESERVED_MASK`. The gate mints no codes (`penstock.rs:499`).
4. **BLAKE3 receipt chain.** `crates/cgap-ffi/src/receipt_chain.rs:1-26`:
   link law `R_i = BLAKE3(prev_hash || block_bytes || license_bytes)`,
   genesis prev-hash all-zero; strict per-link position invariant
   (index strictly increasing, zero gaps, `prev_hash == prior hash`)
   enforced in `verify` (court `chain_position_monotonicity_invariant`).
   Any tamper collapses to ONE opaque refusal class
   `ReceiptChainRefusal::Spoliation` (receipt_chain.rs:14-16 header,
   enum :63-70). The PRD equation is PROSE-ONLY (XC-03) - `Sigma_PQC`
   has zero Rust referents (receipt_chain.rs:19-24).
5. **Arrow/Parquet projection.** `cgap-parquet` ring-commit surface
   (`mmap_writer.rs:721` `RingSink`, 65,376-lane ceiling derived from
   the 65,536-byte StaticPageBuffer - bytes vs lanes, data-dependent);
   byte-identical replay is gated
   by `make gate0` (two `ggen sync run` passes, `cmp`) with the frozen
   seed pair at `crates/cgap-parquet/src/bitslice.rs:687-699`
   (SplitMix64 multiplier `0xBF58_476D_1CE4_E5B9` vs corpus seed
   `0xA20C_0FFE_E5A7_E000` - never conflate).
6. **OCEL 2.0 projection with Spoliation verifier.**
   `crates/cgap-ocel2/src/lib.rs:1-40`: `project` runs the REAL
   `RingReceiptChain::verify` over live committed bytes FIRST; any
   tamper -> typed `Ocel2Refusal::Spoliation`, NO tables emitted
   (refusal law, lib.rs "Refusal law" section). Events keyed by chain
   ordinal (deliberately no wall-clock timestamp column); objects =
   witness identities + `block:<hex8>` fallback keeping E2O total.

## 3. Host-side operational contract

The kernel is passive and refuse-closed; orchestration is a HOST
obligation. The crate provides seams, the host provides the loop.

### 3.1 CPU affinity / SPSC sharding

The ingress ring (`ingress_ring.rs:1-21`) is single-producer/
single-consumer by construction (`&mut self` one-handle law). Sharding
producer/consumer pairs across cores and pinning each pair (CPU
affinity) is a host deployment decision: the substrate exposes the
one-writer-per-handle seam and documents the SPSC contract; it does not
affinity-pin. Perf numbers measured under this contract are quiet-host
scoped (register rows 5/8 below). **[TARGET-v26.11.2]** a dedicated
affinity-contract design note is registered, not yet landed. Any
sub-10 ns/frame claim is [TARGET] until measured under
`06-bench-protocol.md` (row 5 scope law).

### 3.2 CONTENTION-SUSPECT quiet-host guard

Emission: `crates/bench/benches/chains-ingress.rs:716-802` (guard
contract :716-719, `CONTENTION_MARK` const :745, emission :802, :854),
copied into `crates/bench/benches/mctq_consolidation.rs:31,65,98`. Consumption law:
`06-bench-protocol.md` section 8 - a run whose stderr carries
`CONTENTION-SUSPECT:` is REJECTED for quiet-host-scoped claims and
voids any claims-file row citing it. Enforcement is PROCEDURAL today;
a machine gate (post-receipt grep, fail-closed) is a TARGET row until
it exists with a receipt. Consumers of register rows 5/6/7/8 must
state the marker check was clean.

### 3.3 Exit-code ABI

Gate/court exit codes: 0 pass / 1 gate fail / 2 subject missing /
3 loud skip (repo CLAUDE.md "Gate exit codes"; loud-[SKIP] class at
`07-court-corpus.md:13` and `09-overnight-mutation-set.md:19`). A skip
is loud, never a fake pass;
a mutation-free green gate run is VACUOUS (exit nonzero). A court that
cannot flip is FAIL. Skips surface as exit 3, never silent green.

### 3.4 Status registers and typed refusals

`crates/cgap-ffi/src/sovereign.rs:71-90`: `CGAP_OK` 0x0000_0000 (:71),
`CGAP_ERR_INVALID_MAGIC` 0x4347_0001 (:73),
`CGAP_ERR_LEASE_EXHAUSTED` 0x4347_0004 (:80),
`CGAP_ERR_CONCURRENT_WRITER_DENIED` 0x4347_0006 (:87),
`CGAP_ERR_P28_SPOLIATION` 0x5032_38FF (:90). The kernel exposes state;
the host decides (alerting is host-side per
`redteam/fix-14-ops-requirements.md` R-A6.2) - poll-based detection
via `cgap_stats_snapshot` (R-A1.2/R-A1.3, stats.rs landed per
`lanes/lane-rc05-stats-block.md`; ring-site integration is
DOCUMENTED-NOT-WIRED, fix-14 A1 honest residual) and the fleet SLO
surface `crates/cgap-fleet-ops/src/slo.rs` (host poller +
`AlertPredicate`, post-RC-13, fix-14 A6).

## 4. Measured-performance register pointers

Never restate numbers here; cite the register:
`docs/jira/v26.11.2/11-measured-claims.md`. Load-bearing rows for this
topology: row 5 (L1 full chain 10.20 fastest / 10.30 median ns/frame,
QUIET HOST ONLY, receipt `waves/w3-l1-ingress.md:63,69-70`); row 8
(<15 ns/frame target MET on quiet host only); row 28 (registered
contention divergence: 124.06 ns/frame slowest under combined load,
`redteam/red-10-timing-jitter-exploit.md:43-66`); row 10 (L2 <10 ms
target NOT MET); row 29 ("constant-time" ingress = UNSUPPORTED - PRD
prose only, branches are typed refusals per BR3).

## 5. Deep-doc pointer index (this file stays the index)

- C4 model (context/container/component/L4 flowcharts, honest-notes
  register): `docs/jira/v26.11.2/dflss/17-c4-architecture.md`
- Bench method + units law + contention consumption:
  `docs/jira/v26.11.2/06-bench-protocol.md`
- Measured-claims register: `docs/jira/v26.11.2/11-measured-claims.md`
- Deployed-ops requirements (A1/A2/A3/A6 + honest residuals):
  `docs/jira/v26.11.2/redteam/fix-14-ops-requirements.md`
- Boundary gate (E=0 fail-closed dep audit):
  `scripts/boundary_gate.py`; zero-string gate:
  `scripts/zero_string_gate.py` (`--self-check` = non-vacuity)
- Frame/header layout: A04 `CgapFrameHeader` (128 B);
  attestation: A17 `P28LicenseAttestation` (80 B); WAL framing: A05;
  BN3 layout: A11; SlotId: A06; sovereign entrypoints: A19 -
  `docs/jira/v26.11.1/lanes/`
- Publish law: `docs/jira/v26.11.2/01-release-gates.md:26-32`
  (delisting statement; guarded per-crate table at :327)

## Divergences (dispatch vs tree)

- Dispatch cited "OPT-05" (zero payload copies) and "OPT-06"
  (affinity design note) as lever IDs. Grep over `docs/` + `crates/`
  this session finds NO on-disk referent for OPT-05 or OPT-06 (only
  `OPT-03` exists: `bench-validate/opt-03-channel-curve.md`). Doc
  yields to the tree: zero-copy claims are cited to
  `pack_demux.rs:20-24` directly; the affinity contract is carried as
  a [TARGET] with that provenance stated.
- Dispatch's "dflss/17" path is
  `docs/jira/v26.11.2/dflss/17-c4-architecture.md` (the orphan
  `docs/jira/v26.11.2/17-c4-architecture.md` redirects per DF-2,
  :3-6).
- Dispatch framing "six-row taxonomy penstock.rs:505-513" verified
  exactly (rows table at :505-513); row 6 is ADMITTED-with-divergence,
  not a refusal - stated above.
