# 13 — Arrow schema for triples/quoted statements + semi-join kernel spec

Status: CANONICAL design contract (operator-authored, landed 2026-11-02). Governs A2's L3
semi-join implementation and any quoted-triple work on the cgap substrate. Receipt:
waves/v2-18-schema-landing.md.

## 1. Arrow schema for triples and quoted statements (RDF-star)

All lexemes, IRIs, blank nodes, and quoted statements are interned at ingress into
fixed-width 64-bit identifiers: SlotId = BLAKE3(canonical_bytes)[0..8] as u64. A ground
term gets a SlotId; a quoted statement <<s,p,o>> is recursively content-addressed:
SlotId_quoted = BLAKE3(s || p || o || TAG_TRIPLE)[0..8]. Quoted triples map into the same
64-bit coordinate space, so the statement table stays flat, fixed-width, 100% non-nullable.

### 1.1 statement_record_batch

Schema: slot_s : UInt64 (non-nullable), slot_p : UInt64 (non-nullable), slot_o : UInt64
(non-nullable). 24 B/row; a 64-B cache line holds 8 single-column terms; SWAR evaluates 8
rows per 512-bit register. No offsets, no validity bitmaps (null count = 0).

### 1.2 quoted_triple_batch (the inversion table)

Schema: composite_id : UInt64 (primary key, sorted), sub_s/sub_p/sub_o : UInt64
(non-nullable). Recursion: a subject that is itself a quoted triple holds the inner
composite_id in sub_s. Unnesting is binary search or radix probe over the sorted
composite_id buffer — never pointer dereference or recursive graph walk.

### 1.3 dictionary_batch (cold surface)

Schema: slot_id : UInt64 (unique), term_kind : UInt8 [0=IRI, 1=Literal, 2=BNode,
3=QuotedTriple], lexeme : LargeString. Execution kernels NEVER reference it; ingress
serialization and egress materialization only.

## 2. Semi-join kernel formalization

Datalog closure R(x,z) <- ΔR(x,y), B(y,z): the bottleneck is the semi-join, formalized as
a zero-allocation, scratchpad-bounded, vectorized radix semi-join.

Invariants: (1) O(1) SRAM — scratchpad, hash buckets, emission indices on pre-leased
static leases; (2) deterministic linear time — build O(M), probe O(N), total O(N+M), no
collision degradation; (3) branchless candidate pruning — register equality masks, no
conditional jumps.

Execution: Phase 1 scratchpad radix partition (4,096 bucket heads = 32 KB L1; per-j
singly-linked chain via next[]); Phase 2 vectorized SWAR radix probe (h = (y ^ y>>16) &
0x0FFF); Phase 3 branchless lane_eq_mask equality + direct emission; Phase 4 contiguous
column scatter into the output Arrow buffer. Dedup: vertical bit-plane equality pruning —
BitPlane_seen(x,z)==0 admits to ΔR_{t+1}; fixed point when |ΔR_{t+1}| == 0 (semi-naive
termination).

Reference kernel (#![no_std]): JoinScratchpad { bucket_head: [u32; 4096],
next_chain: [u32; 65536] }, ArrowColumnView<'a> { data: &'a [u64] },
unsafe fn semi_join_radix_probe_kernel(delta_r_x, delta_r_y, b_y, b_z, scratch, out_x,
out_z) -> usize — build phase chains B by (key ^ key>>16) & MASK into L1 bucket heads;
probe phase follows the L1-resident index chain with branchless equality
(emitted += is_match arithmetic advancement). Capacity contract: out buffers >= 4× |ΔR|.

## 3. Comparison envelope (per the unit-hygiene law)

| Dimension | Heap/trie graph engine | cgap pure-Arrow kernel |
| --- | --- | --- |
| Join execution | virtual iterator chain, dynamic dispatch | scratchpad radix probe over linear memory |
| SRAM overhead | heap allocation per tree node | O(1) static scratchpad |
| Branch stress | conditional jumps on pointers | branchless masks, sequential scans |
| Access pattern | non-contiguous pointer traversal | cache-line aligned streaming O(N+M) |
| Embedded | needs OS heap / C++ std | native #![no_std], <= 32 KB Flash |

## Footer

{"lane":"v2-18-schema-landing","files_written":["docs/jira/v26.11.2/13-arrow-star-schema.md"],
"commands":["operator-authored spec landed verbatim"],"courts_run":[],"standing":"ALIVE",
"divergences":[]}
