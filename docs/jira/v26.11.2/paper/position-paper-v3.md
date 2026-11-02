# position-paper-v3.md

Issued 2026-11-02 by lane D10 (Design), /Users/sac/cgap @ main. Supersedes
`docs/jira/v26.11.1/waves/position-paper-v2.md` for the v26.11.2 starDF
campaign. Companion: `claims-appendix.md` (same directory) maps every
quantitative claim in this file to a receipt path:line or a [CITE-TBD]
placeholder. Numeric source of record: `../11-measured-claims.md`; method
of record: `../06-bench-protocol.md`.

Tagging law (non-negotiable, enforced by the audit in section 7 and the
appendix tail): `(receipt: <file>:<line>)` for a number carried by a
receipt on disk; `[TARGET-v26.11.2]` for an aimed-for number with no
measuring receipt; `[CITE-TBD: version+date]` for a public fact cited
from outside the repo that must be version+date pinned before release.
Nothing else is lawful.

## 1. Paradigm shift

cgap is a sovereign `#![no_std]` columnar substrate. The v26.11.2
campaign sharpens v2's thesis into a paradigm claim: at the cgap runtime
tier **the graph is dead** (receipt: ../12-ard-prd-cgap.md:143-157,
Substrate Doctrine). Semantic graphs are compile-time artifacts; the
runtime sees only dictionary-encoded 64-bit SlotId columns, 24 B/row
3-column statement segments, bitmask filters, and radix semi-joins over
contiguous memory. RDF and Arrow are lifecycle phases of one data plane,
not competing runtimes (receipt: ../12-ard-prd-cgap.md:145).

Three-dimension paradigm table (architecture difference only; no
performance ranking is asserted in this table):

| Dimension | Heap/pointer-graph engine | cgap columnar runtime |
|---|---|---|
| Data layout | pointer-chasing object graphs, per-node heap allocation | fixed-width 64-bit SlotId columns, 24 B/row, no offsets, no validity bitmaps (receipt: ../13-arrow-star-schema.md:17-19) |
| Dispatch | virtual-iterator / dynamic-dispatch query pipelines | static instruction schedule, branchless masks, compile-time topology (receipt: ../12-ard-prd-cgap.md:104-110) |
| Verification posture | test suites assert observed behavior | every capability lands with a flip-or-FAIL court, sha256 fail-closed restore, and a receipt naming commands and exits (receipt: ../12-ard-prd-cgap.md:123-126) |

No row of this table is a performance claim. Unit discipline binds the
whole paper: GB/s, ns/frame, us/tx and derived facts/s are never
combined into synthetic multipliers (receipt: ../06-bench-protocol.md,
section 7 unit law; receipt: ../11-measured-claims.md:52-64).


## 2. Patent deconstruction (NOT legal advice)

**Header: this section is an engineering register, explicitly not legal
advice.** Source of record: `../patent/claims.txt` (transcribed from
`../03-ost-design-around-register.md`). Four claim rows; each names a
claim element cgap does NOT practice, and the cgap mechanism that is
different - a design-around, never a paraphrase (receipt:
../03-ost-design-around-register.md:8-13).

| Patent | Claim element NOT practiced | cgap mechanism (distinct) | Status |
|---|---|---|---|
| US 10,817,467 | shared-memory CAS over snapshot state | partition-disjoint snapshot-delta; each partition seals its own delta pages; no shared-memory CAS | DISTINCT-MECHANISM; negative-surface grep evidence `grep -rnE 'CAS|compare_exchange|AtomicU64' crates/cgap-storage/src/ crates/cgap-parquet/src/` exit 1, 0 hits (receipt: ../waves/w2-07-ost-greps.md, via ../03-ost-design-around-register.md:30-43) |
| US 11,216,456 | runtime tuple-binding propagation | compile-time PLNF cardinality + vertical bit-slice masks; bounds fixed before execution | SPLIT: bit-slice half LANDED (crates/cgap-parquet/src/bitslice.rs, 930 lines; receipt: ../03-ost-design-around-register.md:43-48); compile-time PLNF cardinality PENDING |
| US 11,409,698 | runtime subquery continuation frames | stratified semi-naive bit-matrix cascades in caller-leased scratch; no continuation frames | DISTINCT-MECHANISM on negative surface (grep exit 1, 0 continuation/Fiber/resume hits, receipt: ../03-ost-design-around-register.md:56-65); semi-naive cascade implementation PENDING |
| US 11,645,278 | runtime topology switching | branchless register unification + static instruction schedule (R0..=R255 ABI) | LANDED (crates/cgap-bn3/src/unify.rs, register ABI at unify.rs:100,169; receipt: ../03-ost-design-around-register.md:66-77) |

Element-vs-mechanism discipline: for each row the table above is the
element side (what the claim requires) versus the mechanism side (what
cgap builds instead); parity claims are independent mechanisms, not
renamed ones. The G7-PATENT gate
(`verify_ost_design_around.py`, receipt: ../00-ard-prd.md:74-77)
mechanically re-verifies 4/4 DISTINCT-MECHANISM verdicts and demotes to
3/4 on a merged/near-duplicate claim input.

**Samsung as citation-class public fact.** The market thesis that the
acquisition of a legacy in-memory graph engine by a consumer silicon
vendor (Samsung) proved on-device context graphs are the highest-value
symbolic-reasoning market (receipt: ../12-ard-prd-cgap.md:23-31) is a
citation-class public fact, not a cgap measurement. It is admitted here
ONLY as context and must be version+date pinned before external
release: [CITE-TBD: acquisition, version+date]. No performance,
capability, or comparative claim is derived from it.


## 3. State of the art (architecture difference ONLY)

Lawful posture: cgap has executed none of the systems below; this
section admits architecture prose only, no performance ranking. Each
system needs a version+date pin before any external release.

- S1. **Apache Arrow nanoarrow** — a flattened C library view of the
  Arrow C Data Interface, oriented at embedded/language-binding
  consumers of the same columnar contract cgap kernels implement.
  Architecture difference: nanoarrow is an interchange/library tier;
  cgap is an execution substrate whose kernels own admission gating,
  deductive closure, and attested persistence. [CITE-TBD: version+date]
- S2. **DuckDB-DuckPGQ** — DuckDB's property-graph query extension over
  DuckDB's own columnar engine. Architecture difference: DuckPGQ layers
  graph query syntax onto a general OLAP engine with a full planner and
  heap-backed hash aggregates; cgap fixes topology at compile/pack time
  and runs branchless register cascades with no runtime planner.
  [CITE-TBD: version+date]
- S3. **Kuzu** — an embedded graph database with native adjacency
  list/CSR-style storage. Architecture difference: Kuzu's storage is
  graph-native (persistent adjacency structures, join via its own
  vectorized engine); cgap's storage is flat dictionary-encoded columns
  with joins as columnar semi-join cascades — graph semantics reduced
  to columnar operators rather than graph-native storage. [CITE-TBD:
  version+date]

What would make comparison lawful (carried from v2, section 4 A4): (a)
cgap materialization throughput measured (still [TARGET-v26.11.2],
receipt: ../11-measured-claims.md:26 row 26); (b) the competitor
executed on the same workload/host with a stated method; (c)
like-for-like units. Until all three, this section admits architecture
prose only. Until the CITE-TBD pins land, this section is UNVERIFIED as
citation apparatus and loud about it.

## 4. Micro-architectures

### 4.1 Radix semi-join (with the corrected budget math)

The v26.11.2 design contract (receipt: ../13-arrow-star-schema.md:44-56)
specifies: Phase 1 scratchpad radix partition into 4,096 bucket heads
with per-j singly-linked chains via next[]; Phase 2 vectorized SWAR
radix probe with h = (y ^ y>>16) & 0x0FFF; Phase 3 branchless
lane_eq_mask equality; Phase 4 contiguous column scatter.

**Corrected math (fixes an error in our own text).** `../13-arrow-star-schema.md:44`
states "4,096 bucket heads = 32 KB L1". That arithmetic is wrong:
4,096 bucket heads x 4 B (u32) = 16,384 B = 16 KiB, not 32 KB. The
correct figure for the bucket-head array alone is **16 KiB**. (If the
intended subject were the full JoinScratchpad { bucket_head: [u32;
4096], next_chain: [u32; 65536] } (receipt: ../13-arrow-star-schema.md:51-52),
that array pair is 16 KiB + 256 KiB = 272 KiB, which does not fit L1 on
any plausible host — the "32 KB L1" phrasing fails on either reading;
16 KiB is the lawful bucket-head figure.) Divergence from the canonical
contract is recorded in the appendix.

**Collision chains, not zero-collision.** The contract's invariant list
claims "no collision degradation" (receipt: ../13-arrow-star-schema.md:40-42),
but the same contract's own mechanism is singly-linked chains through
next[] per bucket (receipt: ../13-arrow-star-schema.md:44-45). The honest
statement: chains are the mechanism; cost is O(N+M+hits) with bucket
load factors determined by the key distribution and the 0x0FFF mask;
degradation is a load-factor story, not eliminated by fiat. The
differential court at N in {1, 64, 4096, 100_003} (receipt:
../plans/levers-L1-L5.md:77) is the check that catches degradation.

### 4.2 56-bit payload-window hash

The bit-slice layer keys scans on a w=56 payload window (receipt:
../plans/levers-L1-L5.md:61; the 56-bit ceiling arm is witnessed by the
A13 hostile-domain court, receipt:
../../v26.11.1/waves/a13-finish-receipt.md:77 — `(2^56-1,1)` ->
`Err(Overflow)`). The radix probe hash h = (y ^ y>>16) & 0x0FFF masks
into 12 bits of bucket index; the payload window supplies the remaining
key discipline: 64-bit slots carry an 8-bit tag plane over a 56-bit
payload window, so bucket hashing never collides on tag-only
differences within a window. Design-spec statement; no measured claim
attached. [TARGET-v26.11.2] for any perf figure in this clause.

### 4.3 SWAR: the honest wrong-bottleneck story

The L2 lever (word-wise u64 SWAR FNV over `footer.rs:119-127`) landed
its kernel, flipped its court (M-B shift corruption `(w>>24)`->`(w>>25)`,
exit 101, sha256-exact restore; receipt:
../waves/w3-l2-swar-fnv.md:38-42), and then **the chain did not move**:
42.58-43.1 ms @1M vs the 41.54 ms baseline, target <10 ms NOT MET, and
the plan's bottleneck attribution was REFUTED — the chain is dominated
by columnar encode/decode, not FNV (receipt:
../waves/w3-l2-swar-fnv.md:26-28,61-63; receipt:
../11-measured-claims.md:30-34 rows 9-11). The paper states this as the
story, not a win: SWAR is a real, court-proven kernel on a stage that
was not the bottleneck. Attribution discipline (12 §5.3) demands the
hypothesis is recorded refuted and engineering shifts to the true
bottleneck (receipt: ../12-ard-prd-cgap.md:131-135).

### 4.4 Eytzinger layout (design-spec only)

Eytzinger-order layout of the sorted composite_id buffer (the
quoted-triple inversion table, receipt: ../13-arrow-star-schema.md:23-26)
is DESIGN-SPEC ONLY. No cgap implementation, no Divan row, no receipt.
The v1-era "1.5-3x" class figures sometimes quoted for cache-friendly
layouts are deleted for this paper: no cgap receipt derives any such
multiple, so none is stated. Any future Eytzinger work lands only with
a measuring receipt; until then the section carries a number-free
design sketch and a loud [TARGET-v26.11.2] posture.

### 4.5 Micro-Parquet (per-clause scoping)

Micro-Parquet work is scoped per-clause, not as a monolith: scope =
write-path clauses of the cgap-parquet kernel (footer seal, RLE decode,
bit_unpack) each as an independently measurable clause with its own
Divan row and receipt; no cross-clause composite number is lawful until
each clause carries one. Current clause-level receipts: rle_decode_direct
49.95 us / 0 B (receipt: ../../v26.11.1/waves/bench-parquet-receipt.md:77);
bit_unpack 6.05B values/s (receipt:
../../v26.11.1/waves/paper-finalize.md:88-90 class, compendium figure
carried via v2 M-notes). Cross-clause composite: [TARGET-v26.11.2].

## 5. Systems synthesis

### 5.1 Four-tier budget table (measured / UNMEASURED cells named)

Tiers are the four CTQ customer segments (receipt: ../15-ctq-tree.md
sections 2-5). Every cell names MEASURED (receipt cite) or
UNMEASURED ([TARGET-v26.11.2]).

| Tier (segment) | CTQ | Cell | Status |
|---|---|---|---|
| S1 enclave OEMs | Binary footprint | 28,679 B pre-link OBJECT, 87.5% of the 32,768 B budget | MEASURED (receipt: ../waves/w3-t3-silicon.md:36,50; subject class named: object, NOT linked ELF) |
| S1 enclave OEMs | Linked-ELF ceiling 131,072 B / <115 KiB target | no linked-ELF receipt this campaign | UNMEASURED [TARGET-v26.11.2] (receipt: ../15-ctq-tree.md:36-38) |
| S1 enclave OEMs | 0 alloc/panic symbols | 0/0 defined, arrow + blake3 + parquet | MEASURED (receipt: ../waves/w3-t3-silicon.md:56-62) |
| S2 real-time | Frame ingress ns/frame | 10.20 fastest / 10.30 median, <15 ns target MET | MEASURED (receipt: ../waves/w3-l1-ingress.md:63,69-70) |
| S2 real-time | Commit us/tx, ring path | 1.05-1.07 us/tx vs 45.8 recorded baseline, <4 us/tx MET | MEASURED (receipt: ../15-ctq-tree.md:52-55; ../waves/a3-ring-sink.md) |
| S2 real-time | 0 blr / 0 br xN / 0 brk census | 0/0/0, direct bl only | MEASURED (receipt: ../waves/w3-t2-asm-census.md:23-29) |
| S3 agentic AI | Closure facts/s/core | unmeasured; RDFox never executed; comparison arm = published 2-6.1M facts/s envelope | UNMEASURED [TARGET-v26.11.2] (receipt: ../11-measured-claims.md:26 row 26; ../06-bench-protocol.md:109-120) |
| S3 agentic AI | WAL restart replay | byte-identical replay owed to A05 framing | UNMEASURED [TARGET-v26.11.2] (receipt: ../15-ctq-tree.md:74-76) |
| S3 agentic AI | Typed refusals 100% CgapStatus | A19 surface defined; campaign restart receipts absent | UNMEASURED [TARGET-v26.11.2] (receipt: ../15-ctq-tree.md:74-76) |
| S4 high-assurance | Court non-vacuity | L1/L2/L5 flips exit 101, sha256-exact restores | MEASURED (receipt: ../11-measured-claims.md:44-46 rows 23-25) |
| S4 high-assurance | License URN pin | `urn:avec:license:p28-sil:v2.1.0-sovereign` byte-exact (A17, 80 B attestation) | MEASURED class (receipt: ../15-ctq-tree.md:87-97) |
| S4 high-assurance | Seed non-conflation | SplitMix64 multiplier vs corpus seed pair intact at bitslice.rs:687-699 | MEASURED (receipt: ../waves/w3-t7-hygiene.md:13-14,59) |

### 5.2 The 128 KiB whole-envelope gate as enforcement law

The whole-envelope law: the linked riscv32imc ELF is the WHOLE-envelope
subject, ceiling 131,072 B (= 128 KiB), target <115 KiB (receipt:
../15-ctq-tree.md:23; receipt: ../16-sipoc.md:46). The gate is
enforcement, not aspiration: the object-level 28,679 B figure is a
pre-link OBJECT and never rounds up to ELF (receipt:
../15-ctq-tree.md:36-38). Until a linked-ELF receipt exists, the
whole-envelope cell is UNMEASURED and any "fits in flash" phrasing is
[TARGET-v26.11.2].

### 5.3 Attestation law

Conservation law of record: `Delta S_admitted + Phi_lease = R_receipt
(mod Sigma_PQC)` (receipt: ../12-ard-prd-cgap.md:17-20). The law binds
admitted state deltas, execution leases, and receipts under a
post-quantum attestation regime. Sigma_PQC (the PQC aggregate) is
[TARGET-v26.11.2]: no PQC algorithm measurement exists this campaign.
Fail-closed posture is measured: tamper refusal emits EXACTLY
0x5032_38FF in both directions of a header-flip court (receipt:
../15-ctq-tree.md:28-34; receipt: ../waves/a3-ring-sink.md:50; const at
crates/cgap-ffi/src/sovereign.rs:83).

## 6. Gaps and falsifiers

- F1 (L3 >20x): the PLNF semi-join + radix probe target >20x @1M is
  [TARGET-until-w5-l3-remeasure lands] — the L3 lever measurement is
  not yet re-measured by the 06-bench-protocol method on the repaired
  unify.rs surface. Falsifier: if the w5-l3 remeasure lands below 10x,
  the join was not the 81.2% chain share and the lever re-PARETOs
  (receipt: ../plans/levers-L1-L5.md:75-76).
- F2 (closure bench): >50M closure facts/s/core is [TARGET-v26.11.2]
  (receipt: ../00-ard-prd.md:59-60). Falsifier: MW4 receipt measuring
  below the RDFox 2-6.1M facts/s envelope like-for-like kills the
  parity ambition in section 1.
- F3 (whole envelope): falsified the moment a linked-ELF receipt
  exceeds 131,072 B, or an object figure is ever quoted as an ELF
  figure. A quoted-ELF misuse of the 28,679 B object number falsifies
  this paper's discipline claim directly.
- F4 (patent register): falsified if any lane restates a patented
  mechanism as a design-around — typed REFUSED (receipt:
  ../03-ost-design-around-register.md:8-13; gate G7-PATENT, receipt:
  ../00-ard-prd.md:74-77).
- F5 (citation apparatus): falsified if any [CITE-TBD] pin ships
  unpinned, or an invented version/date appears. Zero invented sources
  is the discipline claim; one invented cite falsifies the paper.
- F6 (whole paper): a single number in this file lacking a
  `(receipt: ...)` / [TARGET-v26.11.2] / [CITE-TBD] tag falsifies the
  tagging law. Audit command + exit recorded in the appendix tail.

## 7. Self-audit

- Line-length audit: `awk 'length > 100'` over this file; the only
  expected >100-char lines are markdown table rows (exempt-by-convention
  per the v2 finalize receipt, receipt:
  ../../v26.11.1/waves/paper-finalize.md:34-36 divergence D2). Actual
  counts recorded in the appendix tail.
- Untagged-numeral audit: grep-based sweep for digit-bearing lines
  without an on-line tag; method per the v2 finalize receipt
  (../../v26.11.1/waves/paper-finalize.md:33-44): each digit line must
  be (a) tag-bearing, (b) an identifier (patent numbers, file paths,
  register names like R0..=R255), or (c) a named-deleted quotation.
  Command + exit recorded in the appendix tail.
- Banned-phrasing sweep: no released-reading v26.11.2 claims (v26.11.2
  is roadmap), no performance delta vs any unexecuted competitor
  system, no unsourced baselines. Result recorded in the appendix tail.
- Anti-vacuity posture: this is a text lane; no executable court exists
  for prose. The falsifiers in section 6 are the non-vacuity story,
  each backed by the cited lanes' in-session court runs (L1/L2/L5 flips
  exit 101, receipt: ../11-measured-claims.md:44-46). The self-audit
  falsifier F6 is checked by the appendix-tail command; an untagged
  number on any line of this file falsifies the paper.


---

## ERRATA (2026-11-02, TC-08 / cull-08)

Section 5.3, lines 231-233: the conservation law of record is quoted
as "... (mod Sigma_PQC)". `mod Sigma_PQC` is INVENTED LORE, zero code
referent (cgapprd-02-equation-grounding.md:47-57); landed trace is
three typed commit trailers with a fail-closed render, BLAKE3
classical, PQC seal an unbuilt Wave-3 seam (:70-81). The paper's own
"[TARGET-v26.11.2]" qualifier on Sigma_PQC at :233-234 was already
honest; the "(mod Sigma_PQC)" equation framing at :231 is NOT and is
superseded by this errata. Lines retained unedited.
