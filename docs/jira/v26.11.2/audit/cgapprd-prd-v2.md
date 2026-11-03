# Architecture & Product Requirements Document (ARD/PRD): CGAP — v2 (receipt-true)

Status: CANONICAL product charter, v2 (receipt-true rewrite per
audit/cgapprd-09-prd-rewrite-plan.md; landed 2026-11-02). Supersedes
docs/jira/v26.11.2/12-ard-prd-cgap.md for every section it rewrites;
sections marked STAYS are carried verbatim with receipts attached.
Prior receipt: waves/v2-17-ard-landing.md (ALIVE, its own footer).
00-ard-prd.md carries the campaign supersession pointer.

Falsifier (verbatim, load-bearing): The PRD Verify passes iff every
quantitative claim carries a receipt file:line and zero claims use a
referent that fails an in-tree grep.

Receipt conventions: R1 = audit/cgapprd-01-prd-audit.md (fresh
commands, per-claim verdicts); R2 = audit/cgapprd-02-equation-grounding.md;
R3 = audit/cgapprd-03-frame-layout.md; R8 = audit/cgapprd-08-boundaries.md;
R8b = audit/cgapprd-08b-crossrepo-referents.md. Lanes are
docs/jira/v26.11.2/lanes/*.md.

## 1. Executive Summary & Product Vision

**CGAP** is a deterministic, bare-metal-capable columnar execution, state-admission, and
cryptographic attestation substrate. It is engineered to serve as an embedded Personal
Data Engine and sovereign state machine for edge silicon, secure enclaves,
microcontrollers, and deterministic high-throughput hosts.

While legacy semantic reasoning engines rely on heap-allocated, pointer-heavy object
graphs, dynamic virtual dispatch, and atomic lock contention, CGAP operates strictly over
flat, dictionary-encoded columnar arrays using branchless register arithmetic and
pre-leased memory buffers (receipt: fresh AE4 disassembly invariants exit 0 incl. binary
mutant, R1 fresh commands; lane-g01-scan-branchless.md:3).

### 1.1 State-Transition Law (rewritten per R2)

The Chatman Equation's landed trace is three typed commit trailers with typed
refusals — `lease-phi`, `lease-delta`, `lease-receipt` — and a fail-closed
render; no modular arithmetic exists. `lease-phi` is a caller-supplied 32-byte
digest the render passes through; `lease-delta` is a u64 admitted-delta count;
`lease-receipt` is an 80-byte frame (commit_synth.rs:62,111-156)
(nonce || parent || payload || tag) verified fail-closed
against its domain-separated tag. There is no Σ_PQC term in
the tree: the digest is BLAKE3 (classical), and any
post-quantum seal remains an unbuilt Wave-3 seam.
(R2 exact rewrite; receipts: crates/cgap-git/src/commit_synth.rs:60-62,
111-156,187-189,260-267; zero .rs hits for Σ_PQC per R2 grep 2;
crates/cgap-crypto/src/chatman.rs:9-10,351,368,422)

## 2. Strategic Positioning & Ecosystem Boundary

### 2.1 Market Thesis

The acquisition of legacy in-memory graph engines by consumer silicon vendors proved that
the highest-value market for symbolic reasoning is not cloud-hosted triple stores, but
**on-device context graphs** that eliminate probabilistic AI hallucinations. However,
traditional engines require megabytes of runtime footprint, operating-system heap
allocators, and multi-threaded compare-and-swap synchronization. CGAP captures the
embedded and sovereign computing tier where those engines cannot execute: sub-32 KB Flash
environments, zero-allocation real-time loops, and hardware-attested execution pipelines.
(Carried verbatim per R9 SS2.1 STAYS: historical/qualitative claims, no in-tree referent
asserted.)

### 2.2 Strict Ecosystem Separation

CGAP operates within a three-tier architectural boundary and never absorbs upstream
experimental bloat:

| Tier | Domain Responsibility | Boundary Law |
| --- | --- | --- |
| **CGAP (Manufacturing & Execution Substrate)** | Columnar storage, two-stage admission gating, branchless join/materialization kernels, lease-bounded transaction commits, and cryptographic receipt chaining. | Runtime is strictly columnar and fixed-width. Semantic graphs exist only as compile-time specifications or pre-interned 64-bit integer columns. |
| **Semantic Research Plane (Upstream)** | Experimental graph semantics, quoted statements, query plan exploration, and complex ontology modeling. | Completely decoupled. Experimental capabilities only enter CGAP after reducing to fixed-width columnar operators or binary ontology packs. |
| **Projection & Interface Compiler (Tooling)** | Declarative generation of command-line surfaces, C-ABI headers, binary layout descriptors, and validation gates. | Compile-time only. Zero runtime footprint inside CGAP execution kernels; verified via byte-identical regeneration proofs. |

(Carried verbatim per R9 SS2.2 STAYS; mechanism receipts:
crates/cgap-ffi/src/sovereign.rs:413 cgap_admit_frame admission;
crates/cgap-git ingress_ring + admission_gate, R8 W1 clause 2.)

## 3. Product Requirements (PRD)

### 3.1 Core Functional Capabilities

* **P1 — Two-Stage Fail-Closed Admission Penstock:** All inbound frames must pass a
  constant-time, branchless header inspection before invoking cryptographic digest
  computation. Invalid magic numbers, unaligned frames, unlicensed binary schemas, or
  malformed flags must be rejected immediately in Stage 1. Only frames admitted with
  explicit escalation indicators trigger Stage 2 full payload hashing.
  (STAYS per R9. Receipts: fresh verify_disassembly_invariants.py exit 0 incl. binary
  mutant, R1; lanes/lane-g01-scan-branchless.md:3; lane-g02:118-121.
  QUALIFIER, X-04FIX 2026-11-02: "constant-time" is UNSUPPORTED — no measured claim
  asserts it; BR3 rejected the branch-free dispatch item because typed-refusal branches
  are lawful — `docs/jira/v26.11.2/11-measured-claims.md` row 29. Requirement of
  record: branchless fast path with fail-closed typed refusals, not constant-time.)

* **P2 — Fixed-Width Columnar Representation:** All entities and relations are represented
  at runtime as 64-bit fixed-width identifiers derived from deterministic cryptographic
  hashing of their lexical forms. Statements are stored in contiguous 3-column segments
  (24 bytes per row) compatible with zero-copy columnar interchange standards, eliminating
  runtime string interning and pointer chasing. Clarification (R3 SS5): the landed CSCB
  fact frame is 24 B = 3x u64 LE
  (crates/csct/src/bin/cscb_c1_c2_ingress.rs:45 FRAME_BYTES=24 type-level pin;
  generate_cscb_corpus.py:25,66 "<QQQ"; crates/cgap-git/src/ingress_ring.rs:30;
  crates/cgap-git/src/pack_demux.rs:40-51; workspace-cgap.yaml:23). The only 64-byte
  object is cgap-arrow's Quad64 buffer layout
  (crates/cgap-arrow/src/layout.rs:75 QUAD_SIZE=64) — not the fact frame; no code reads
  bytes 24..64 of any fact (R3 SS5).

* **P3 — Branchless Relational & Deductive Materialization:** Rule evaluation and
  relational joins execute as columnar semi-join cascades and scratchpad radix probes over
  contiguous integer arrays (receipt: crates/cgap-bn3/src/unify.rs:840
  RADIX_BUCKETS=4096, R1 FR-02 VERIFIED; audit/dod-cgap-bn3.md:17). Incremental additions
  and retractions operate over dual-regime columnar delta segments rather than mutable
  pointer graphs. Boundary note (R8 W3): cross-boundary joins stay bounded —
  cgap_radix_join is MISSING at every layer (NO-GO CTQ-04); no C-ABI join surface may be
  inferred from this paragraph.

* **P4 — Lease-Bounded Sovereign Transaction Commits:** State mutations across the C-ABI
  boundary must execute within pre-allocated, monotonic execution leases using ring-buffered
  persistence. The system must never invoke per-transaction operating-system file creation
  on the hot path, and must deterministically refuse execution the instant a monotonic
  lease ceiling is exhausted or buffer spoliation is detected. (STAYS per R9. Receipts:
  crates/cgap-git/src/ingress_ring.rs:146-150,280; commit_synth lease trailers per R2;
  crates/cgap-bn3/tests/posh_alloc_sentry_court.rs:8-24, R1 fresh.)

* **P5 — Cryptographic Receipt Chaining & Licensing:** Every admitted state transition
  must emit a tamper-evident, hash-chained receipt bound to a verified sovereign license
  identifier. Unlicensed or tampered state transitions fail closed prior to state
  mutation. (STAYS per R9. Receipts: cgap-blake3 31/31 incl. chain_mutation_court, R1
  fresh; chain_mutation_court.rs:1; commit_synth.rs:111-156 — 80-byte frame,
  ReceiptTagMismatch fail-closed, R2.)

* **P6 — Binary Ontology Packs (O-ABI):** Domain schemas (such as financial and cloud
  infrastructure ontologies) must compile down to byte-stable, zero-parse binary images
  that map directly into read-only memory without runtime deserialization. (STAYS per R9.
  Receipt: crates/cgap-git/src/pack_demux.rs:40-51 FACT_WINDOW=1,365 pack stride with
  HONEST-NONE doc; corpus pack "<QQQ" 24 B, R3 SS2.)

### 3.2 Non-Functional & Target Performance Requirements (rewritten per R1 honest markings)

| Metric / Capability | Requirement / Target | Verification Discipline |
| --- | --- | --- |
| **Admission Ingress Latency** | ASPIRATIONAL-UNVERIFIED target: less than 15 ns per frame | Target; unmeasured in this charter; nearest measured ingress is the IPC-4 mmap transport court, 4.52-4.54 darwin / 4.574-4.656 linux ns/op, 0 allocs (lanes/lane-ax-06-shm.md:70; R8 W2). |
| **Deductive Materialization Rate** | Measured HONEST FAIL: median 23.35 ns/fact (lanes/lane-posh-02-batch.md:43,111), 2.3x over the 10 ns/fact target; the 10 ns figure is withdrawn. Reported strictly in derived facts/s. | No <2 ms closure gate is attached to this row (deleted; see deletions register). |
| **Sovereign C-ABI Commit Latency** | ASPIRATIONAL-UNVERIFIED target: less than 4 us per committed transaction | Target only until a batch-amortized receipt exists (PRD-4 SS7 claim-format law; no current receipt in R1's table). |
| **Embedded Footprint Ceiling** | SRAM VERIFIED: 36,048 B used, headroom 95,024 B (scripts/verify_cscb_sram.py exit 0, R1 NFR-03). Flash ceiling (<32,768 B) and stack bound remain ASPIRATIONAL-UNVERIFIED (stack bound is self-declared). | Verified on the SRAM gate only; Flash/stack unmeasured. |
| **Determinism** | Byte-identical outputs across identical inputs and seeds | VERIFIED: split PRNG seeds court-backed (corpus seed vs algorithm seed never conflated; mutation courts; repo CLAUDE.md seed law). |
| **Scoped Zero-Allocation** | VERIFIED only WITH the scope fence: 2,662 allocs / 650,294,994 B measured whole-pipeline (lanes/lane-posh-04-sentry.md:37; posh_alloc_sentry_court.rs:8-24) | Whole-pipeline 'zero-allocation' without the scope fence is VACUOUS and must not appear (R1 SS5 weakest links). |

Whole-table note (R1 SS5): zero-alloc claims travel with the scoped fence or are omitted.

Contended-load caveat for any rate claim: seals-rate witness is uncontended-only —
median 4,377,052/4,356,280/4,351,961 receipts/s uncontended
(lanes/lane-b06-cscb-c7.md:24); contended run FAILS (lane-b06-cscb-c7.md:52-54).

## 4. Architectural Requirements (ARD)

### 4.1 Execution & Memory Model

1. **Bare-Metal Purity:** Core execution kernels must compile without standard library
   dependencies, dynamic memory allocation, or runtime panic unwinding branches.
   (STAYS per R9. Receipts: fresh AE4 exit 0, R1; cgap-blake3 Cargo.toml:5 no_std with
   allowlisted RV32 residuals cmov_u64/eytzinger_restore/select_slice, R1 NFR-06 —
   residuals not hidden.)
2. **Zero-Indirection Control Flow:** Hot-path admission, hashing, and bit-slice filtering
   routines must compile to linear instruction streams containing zero indirect jumps,
   zero virtual method table dispatches, and zero software trap instructions. (STAYS;
   receipt: fresh AE4 exit 0 incl. binary mutant, R1.)
3. **SWAR and Bit-Plane Parallelism:** Filtering, footer validation, and candidate pruning
   must use SIMD-Within-A-Register (SWAR) word-wise operations and vertical bit-planes to
   evaluate 64 rows per register cycle. (STAYS per R9: design law, no per-byte claim.)
4. **Zero Heavyweight Query Engine Dependencies:** The architecture forbids pulling in
   external analytical query planners or volcano-style execution engines. All columnar
   scans, projections, and joins are self-contained. (STAYS: design law.)

### 4.2 Concurrency & State Isolation (Incumbent Design-Around)

To maintain strict architectural separation from legacy in-memory graph engines:

* **No Lock-Free Pointer Tries:** CGAP prohibits atomic compare-and-swap (CAS) loops over
  shared pointer trees. Concurrency is achieved via disjoint columnar partition ownership
  and append-only lease segments. (STAYS per R9; consistent with R8 W2 — 0-alloc mmap
  transport, ingress_ring SPSC law, ingress_ring.rs:20-30 module doc.)
* **No Virtual Iterator Pipelines:** Sideways information passing is executed via static
  bitmask propagation across contiguous columns rather than runtime iterator chaining.
  (STAYS: prohibition clause.)
* **No Runtime Topology Switching:** Query and deduction topologies are fixed at compile
  or pack-emission time rather than dynamically re-planned via runtime cost heuristics.
  (STAYS: prohibition clause.)

### 4.3 Interface & Boundary Architecture

* **Single-Source Interface Projection:** All public C-ABI headers, format constants, and
  command-line routing layers are projected from a declarative specification. Manual edits
  to generated interface surfaces are treated as build-breaking checksum violations.
  (STAYS: design law.)
* **Typed Refusal Semantics:** Every boundary function returns an explicit, fixed-width
  status code or typed refusal enumeration. Silent skips, uninitialized reads, and untyped
  error strings are prohibited. (STAYS per R9. Receipts: sovereign.rs:413
  cgap_admit_frame typed refusals; ref_store.rs:48-75 Refusal::tag; GIT-09 admission_gate
  AndonRefusal. Caveat per R8 W3: all cross-boundary Go/POSH claims are bounded by the
  frozen 11-symbol admit/eval/commit/verify C-ABI surface until cgap_ingress_bytes,
  cgap_radix_join, and the cgap_shm_* set land — all NO-GO today. Do NOT infer any
  AshA2A wiring from this section: zero in-tree wiring exists, R8 W0.)

## 5. Verification, Epistemic & Release Governance

1. **Anti-Vacuity Court Discipline:** Every verification gate and test court must prove it
   can fail. Each test suite is paired with a named mutation (flipped bit, corrupted
   header, exhausted lease) executed against an isolated sandbox copy; a gate that passes
   without a witnessed mutation flip is classified as vacuous and fails the release.
   (STAYS per R9. Pattern receipts: chain_mutation_court flip witnessed; POSH-4
   positive-control-first court; e5 warmup honesty pin, edge_closure_courts.rs:422-477,
   PRD-4 SS0.)
2. **Strict Unit Hygiene:** Performance reporting must enforce dimensional compatibility.
   Raw memory scan bandwidth (GB/s), admission latency (ns/frame), transaction commit time
   (µs/tx), and deductive closure throughput (derived facts/s) must never be combined into
   synthetic multipliers. (STAYS: discipline clause.)
3. **Honest Pareto Attribution:** Optimization levers are only ratified when end-to-end
   chain benchmarks move proportionally to the targeted stage. If a stage optimization
   fails to move total chain latency—or if a stage is bounded by irreducible cryptographic
   rounds—the hypothesis is recorded as refuted and engineering shifts to the true
   bottleneck (such as columnar encode/decode or scratchpad radix probing). (STAYS:
   discipline clause.)
4. **Topological Release Integrity:** Every deliverable module in the substrate must
   either publish cleanly in topological dependency order with verified consumer linkage,
   or be explicitly classified as an internal verification/benchmark harness. (STAYS:
   discipline clause.)

## 6. Substrate Doctrine (operator, 2026-11-02)

RDF and Arrow are different lifecycle phases of the same data, not competing runtimes.
The semantic plane (upstream research repo, compile-time) owns meaning: IRIs, ontologies,
SHACL, quoted assertions, rules. At cgap runtime **the graph is dead** — no triples, no
IRIs, no pointer traversal; only dictionary-encoded columns, bitmask filters, and radix
joins. Test for membership: if an operation is a linear scan, dictionary lookup, SWAR
bitmask check, or hash/semi-join over contiguous u64s, it belongs here on pure Arrow.
Every semantic function reduces at this tier: lexeme resolution -> SlotId dictionary keys;
pattern matching -> 24 B 3-column segments (receipt as P2 above); rule joins -> columnar
semi-join cascades; SHACL cardinality -> vectorized filters; retraction -> delta-segment
anti-joins. The upstream semantic plane inherits this substrate as a consumer and never
re-implements columnar memory, prefetching, or SIMD. cgap is independently shippable on
these grounds: typed-refusal admission, SIMD filtering, and lease-bounded commits are the
landed courts (receipts as P1/P2/P4 above); sub-15 ns admission and sub-4 us sovereign
commits are TARGETS, unmeasured in this charter (PRD-4 SS7 honesty law; R9 SS6 rewrite).

## 7. Deletions Register (invented lore removed; no rewrite; R9 DELETED)

1. **'mod Σ_PQC' as conservation-law fact** (prior :19). Deleted outright, replaced by
   the SS1.1 typed-trailer block. Σ_PQC has zero code referent anywhere in the tree
   (R2 grep 2: zero .rs hits); the digest is BLAKE3 classical; any post-quantum seal is
   an unbuilt Wave-3 seam. A16-chatman-pqc.md:28,97-99 is the only honest prior
   statement (flags it UNKNOWN); the other six prose docs restate it as law (R2 list).
2. **Sub-ms checkpointing as current capability.** No paragraph of the prior PRD carried
   it and none is imported here: no subsystem exists in-tree (R1 FR-06 DEFECT-OPEN;
   nearest mark dflss/02-ax-charter-ctq.md:68 NO-GO; dred is anti-join not snapshot).
   TARGET-only until a batch-amortized checkpoint receipt lands (PRD-4 SS7).
3. **'<2 ms closure' as a release gate.** Profile-ambiguous per R1 NFR-02: 1.873 ms is
   DEBUG best-of-3 (lanes/lane-git-12-octopus-merge.md:60); release witness non-repro
   (lanes/lane-b02-cscb-c3.md:32); 36.9 s standalone (lanes/lane-mv-05-sla-reconcile.md:66).
   The gate framing is deleted from SS3.2's materialization row; honest per-profile
   wording replaces it.

## 8. Aspirational Register (ASPIRATIONAL-UNVERIFIED, unblock prerequisites named)

| item | standing | unblock prerequisites |
| --- | --- | --- |
| <15 ns/frame admission | ASPIRATIONAL-UNVERIFIED | a measured admission-latency court; nearest measured datum is the mmap transport court (lane-ax-06-shm.md:70) |
| <4 us C-ABI commit | ASPIRATIONAL-UNVERIFIED | batch-amortized commit-latency receipt (PRD-4 SS7 claim-format law) |
| <32,768 B Flash ceiling + stack bound | ASPIRATIONAL-UNVERIFIED | linked-ELF footprint truth per repo CLAUDE.md (no_std caller crate, riscv32imc); stack bound measurement |
| <2 ms closure (per-profile) | ASPIRATIONAL-UNVERIFIED | release-profile witness that reproduces (lane-b02-cscb-c3.md:32 non-repro) |
| sub-ms checkpointing | ASPIRATIONAL-UNVERIFIED | an in-tree checkpoint subsystem (FR-06 DEFECT-OPEN, R1) + batch-amortized receipt |
| Σ_PQC / PQC seal seam | ASPIRATIONAL-UNVERIFIED | Wave-3 pqc seam in cgap-crypto (chatman.rs:9-10,351,368,422 is the only stub mark); until then digest is BLAKE3 classical |
| AshA2A wiring (work-order projection, authority-at-ring, pack consumption, socket trap) | ASPIRATIONAL-UNVERIFIED | bidirectionally absent today (R8b W2): needs cgap_ingress_bytes NIF ingress membrane, sha256-hex vs BLAKE3 canonicalization decision, Refusal union, differential oracle court (R8b W4) |
| cgap_radix_join cross-boundary | ASPIRATIONAL-UNVERIFIED | the join primitive itself (MISSING at every layer, NO-GO CTQ-04, R8 W3) |

## Footer

{"lane":"cgap-prd-10","files_written":["docs/jira/v26.11.2/audit/cgapprd-prd-v2.md","docs/jira/v26.11.2/lanes/lane-cgap-prd-10-landing.md","docs/jira/v26.11.2/audit/doctrine-ship-plan.md (P9 row appended)"],"commands":[],"courts_run":[],"standing":"ALIVE","divergences":["receipt-lane paths corrected: lane receipts live in v26.11.2/lanes, not v26.11.1 — R9's citations re-pointed at the verified v26.11.2 files"]}
