# Architecture & Product Requirements Document (ARD/PRD): CGAP

Status: CANONICAL product charter (operator-authored, landed 2026-11-02). Supersedes
campaign-doc drift; 00-ard-prd.md carries the supersession pointer. Receipt:
waves/v2-17-ard-landing.md.

## 1. Executive Summary & Product Vision

**CGAP** is a deterministic, bare-metal-capable columnar execution, state-admission, and
cryptographic attestation substrate. It is engineered to serve as an embedded Personal
Data Engine and sovereign state machine for edge silicon, secure enclaves,
microcontrollers, and deterministic high-throughput hosts.

While legacy semantic reasoning engines rely on heap-allocated, pointer-heavy object
graphs, dynamic virtual dispatch, and atomic lock contention, CGAP operates strictly over
flat, dictionary-encoded columnar arrays using branchless register arithmetic and
pre-leased memory buffers. Every state transition is governed by the conservation law:

ΔS_admitted + Φ_lease ≡ R_receipt (mod Σ_PQC)

## 2. Strategic Positioning & Ecosystem Boundary

### 2.1 Market Thesis

The acquisition of legacy in-memory graph engines by consumer silicon vendors proved that
the highest-value market for symbolic reasoning is not cloud-hosted triple stores, but
**on-device context graphs** that eliminate probabilistic AI hallucinations. However,
traditional engines require megabytes of runtime footprint, operating-system heap
allocators, and multi-threaded compare-and-swap synchronization. CGAP captures the
embedded and sovereign computing tier where those engines cannot execute: sub-32 KB Flash
environments, zero-allocation real-time loops, and hardware-attested execution pipelines.

### 2.2 Strict Ecosystem Separation

CGAP operates within a three-tier architectural boundary and never absorbs upstream
experimental bloat:

| Tier | Domain Responsibility | Boundary Law |
| --- | --- | --- |
| **CGAP (Manufacturing & Execution Substrate)** | Columnar storage, two-stage admission gating, branchless join/materialization kernels, lease-bounded transaction commits, and cryptographic receipt chaining. | Runtime is strictly columnar and fixed-width. Semantic graphs exist only as compile-time specifications or pre-interned 64-bit integer columns. |
| **Semantic Research Plane (Upstream)** | Experimental graph semantics, quoted statements, query plan exploration, and complex ontology modeling. | Completely decoupled. Experimental capabilities only enter CGAP after reducing to fixed-width columnar operators or binary ontology packs. |
| **Projection & Interface Compiler (Tooling)** | Declarative generation of command-line surfaces, C-ABI headers, binary layout descriptors, and validation gates. | Compile-time only. Zero runtime footprint inside CGAP execution kernels; verified via byte-identical regeneration proofs. |

## 3. Product Requirements (PRD)

### 3.1 Core Functional Capabilities

* **P1 — Two-Stage Fail-Closed Admission Penstock:** All inbound frames must pass a
  constant-time, branchless header inspection before invoking cryptographic digest
  computation. Invalid magic numbers, unaligned frames, unlicensed binary schemas, or
  malformed flags must be rejected immediately in Stage 1. Only frames admitted with
  explicit escalation indicators trigger Stage 2 full payload hashing.
* **P2 — Fixed-Width Columnar Representation:** All entities and relations are represented
  at runtime as 64-bit fixed-width identifiers derived from deterministic cryptographic
  hashing of their lexical forms. Statements are stored in contiguous 3-column segments
  (24 bytes per row) compatible with zero-copy columnar interchange standards, eliminating
  runtime string interning and pointer chasing.
* **P3 — Branchless Relational & Deductive Materialization:** Rule evaluation and
  relational joins execute as columnar semi-join cascades and scratchpad radix probes over
  contiguous integer arrays. Incremental additions and retractions operate over dual-regime
  columnar delta segments rather than mutable pointer graphs.
* **P4 — Lease-Bounded Sovereign Transaction Commits:** State mutations across the C-ABI
  boundary must execute within pre-allocated, monotonic execution leases using ring-buffered
  persistence. The system must never invoke per-transaction operating-system file creation
  on the hot path, and must deterministically refuse execution the instant a monotonic
  lease ceiling is exhausted or buffer spoliation is detected.
* **P5 — Cryptographic Receipt Chaining & Licensing:** Every admitted state transition
  must emit a tamper-evident, hash-chained receipt bound to a verified sovereign license
  identifier. Unlicensed or tampered state transitions fail closed prior to state mutation.
* **P6 — Binary Ontology Packs (O-ABI):** Domain schemas (such as financial and cloud
  infrastructure ontologies) must compile down to byte-stable, zero-parse binary images
  that map directly into read-only memory without runtime deserialization.

### 3.2 Non-Functional & Target Performance Requirements

| Metric / Capability | Requirement / Target | Verification Discipline |
| --- | --- | --- |
| **Admission Ingress Latency** | Less than 15 ns per frame | Measured across both clean bypass traffic and escalated digest-fold traffic. |
| **Deductive Materialization Rate** | Millions of derived facts/sec/core at 1M-row scale | Measured strictly in derived facts/sec (never conflated with raw GB/s scan bandwidth). |
| **Sovereign C-ABI Commit Latency** | Less than 4 µs per committed transaction | Measured end-to-end across the C-ABI boundary using pre-leased ring buffers. |
| **Embedded Footprint Ceiling** | Less than 32,768 bytes Flash; 0 bytes static SRAM heap | Verified on 32-bit embedded RISC-V and WebAssembly targets. |
| **Determinism** | Byte-identical outputs across identical inputs and seeds | Verified via split PRNG seeds (corpus generation strictly separated from algorithm seeding). |

## 4. Architectural Requirements (ARD)

### 4.1 Execution & Memory Model

1. **Bare-Metal Purity:** Core execution kernels must compile without standard library
   dependencies, dynamic memory allocation, or runtime panic unwinding branches.
2. **Zero-Indirection Control Flow:** Hot-path admission, hashing, and bit-slice filtering
   routines must compile to linear instruction streams containing zero indirect jumps,
   zero virtual method table dispatches, and zero software trap instructions.
3. **SWAR and Bit-Plane Parallelism:** Filtering, footer validation, and candidate pruning
   must use SIMD-Within-A-Register (SWAR) word-wise operations and vertical bit-planes to
   evaluate 64 rows per register cycle.
4. **Zero Heavyweight Query Engine Dependencies:** The architecture forbids pulling in
   external analytical query planners or volcano-style execution engines. All columnar
   scans, projections, and joins are self-contained.

### 4.2 Concurrency & State Isolation (Incumbent Design-Around)

To maintain strict architectural separation from legacy in-memory graph engines:

* **No Lock-Free Pointer Tries:** CGAP prohibits atomic compare-and-swap (CAS) loops over
  shared pointer trees. Concurrency is achieved via disjoint columnar partition ownership
  and append-only lease segments.
* **No Virtual Iterator Pipelines:** Sideways information passing is executed via static
  bitmask propagation across contiguous columns rather than runtime iterator chaining.
* **No Runtime Topology Switching:** Query and deduction topologies are fixed at compile
  or pack-emission time rather than dynamically re-planned via runtime cost heuristics.

### 4.3 Interface & Boundary Architecture

* **Single-Source Interface Projection:** All public C-ABI headers, format constants, and
  command-line routing layers are projected from a declarative specification. Manual edits
  to generated interface surfaces are treated as build-breaking checksum violations.
* **Typed Refusal Semantics:** Every boundary function returns an explicit, fixed-width
  status code or typed refusal enumeration. Silent skips, uninitialized reads, and untyped
  error strings are prohibited.

## 5. Verification, Epistemic & Release Governance

1. **Anti-Vacuity Court Discipline:** Every verification gate and test court must prove it
   can fail. Each test suite is paired with a named mutation (flipped bit, corrupted
   header, exhausted lease) executed against an isolated sandbox copy; a gate that passes
   without a witnessed mutation flip is classified as vacuous and fails the release.
2. **Strict Unit Hygiene:** Performance reporting must enforce dimensional compatibility.
   Raw memory scan bandwidth (GB/s), admission latency (ns/frame), transaction commit time
   (µs/tx), and deductive closure throughput (derived facts/s) must never be combined into
   synthetic multipliers.
3. **Honest Pareto Attribution:** Optimization levers are only ratified when end-to-end
   chain benchmarks move proportionally to the targeted stage. If a stage optimization
   fails to move total chain latency—or if a stage is bounded by irreducible cryptographic
   rounds—the hypothesis is recorded as refuted and engineering shifts to the true
   bottleneck (such as columnar encode/decode or scratchpad radix probing).
4. **Topological Release Integrity:** Every deliverable module in the substrate must
   either publish cleanly in topological dependency order with verified consumer linkage,
   or be explicitly classified as an internal verification/benchmark harness.

## Footer


## 6. Substrate Doctrine (operator, 2026-11-02)

RDF and Arrow are different lifecycle phases of the same data, not competing runtimes.
The semantic plane (upstream research repo, compile-time) owns meaning: IRIs, ontologies,
SHACL, quoted assertions, rules. At cgap runtime **the graph is dead** — no triples, no
IRIs, no pointer traversal; only dictionary-encoded columns, bitmask filters, and radix
joins. Test for membership: if an operation is a linear scan, dictionary lookup, SWAR
bitmask check, or hash/semi-join over contiguous u64s, it belongs here on pure Arrow.
Every semantic function reduces at this tier: lexeme resolution -> SlotId dictionary keys;
pattern matching -> 24 B 3-column segments; rule joins -> columnar semi-join cascades;
SHACL cardinality -> vectorized filters; retraction -> delta-segment anti-joins. The
upstream semantic plane inherits this substrate as a consumer and never re-implements
columnar memory, prefetching, or SIMD. cgap is independently shippable on these grounds:
sub-15 ns admission, SIMD filtering, sub-4 us sovereign commits are valuable with zero
graph semantics attached.

{"lane":"v2-17-ard-landing","files_written":["docs/jira/v26.11.2/12-ard-prd-cgap.md"],
"commands":["operator-authored charter landed verbatim"],"courts_run":[],
"standing":"ALIVE","divergences":[]}
