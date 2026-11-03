# 15-ctq-tree

VOC -> CTQ flowdown for the v26.11.2 starDF campaign (charter:
`00-ard-prd.md`). Lane D2. Numeric source of record:
`11-measured-claims.md` - every MEASURED row cites its receipt; rows
without a witnessed measurement are tagged [TARGET-v26.11.2] and never
round up. Method of record: `06-bench-protocol.md`.

## Section 1 - Flowdown structure

Customer segment -> need -> CTQ characteristic -> operational metric
with USL (upper spec limit) / target / LSL (lower spec limit). USL is
the fence the metric must not exceed; LSL the fence it must not fall
below; target the design aim.

## Section 2 - Segment S1: enclave OEMs

Needs: fits the enclave flash budget; no allocator inside the enclave;
sealed evidence survives spoliation; receipts carry BLAKE3 coverage.

| CTQ characteristic | Metric | USL | Target | LSL |
|---|---|---|---|---|
| Binary footprint | Linked ELF size (B) | 131,072 | <115 KiB | riscv32imc flash 32,768 |
| Zero-heap sovereignty | Heap allocs per tx/frame | 0 | 0 | 0 (hard) |
| Spoliation fails closed | Refusal code on tamper | 0x5032_38FF exact | 0x5032_38FF | same (hard) |
| Verifiable receipts | BLAKE3 coverage of sealed blocks | n/a | 100% | 100% (hard) |

Measurement cites: `11-measured-claims.md` #15 (28,679 B pre-link
OBJECT, 87.5% of the 32,768 B budget - MEASURED; subject class named:
object, NOT linked ELF); #16/#18 (0 defined alloc/panic symbols -
MEASURED); `02-pareto-levers.md:172` + `waves/a3-ring-sink.md:50`
(header flip yields EXACTLY 0x5032_38FF both directions - MEASURED
court; const at `crates/cgap-ffi/src/sovereign.rs:83`, corpus anchor
`07-court-corpus.md:79`).

Standing: the linked-ELF 131,072 B ceiling / <115 KiB target row is
[TARGET-v26.11.2] - no linked-ELF size receipt exists this campaign;
only the object-level row is MEASURED and does not round up to ELF.

## Section 3 - Segment S2: real-time

Needs: deterministic per-frame ingress cost; bounded per-tx commit
latency; no hidden indirection on the hot path.

| CTQ characteristic | Metric | USL | Target | LSL |
|---|---|---|---|---|
| Frame ingress cost | ns/frame, full L1 chain | 15 | 10.20 | ceiling-only |
| Commit cost | us/tx, FFI ring path | 4 | 1.05-1.07 | ceiling-only |
| Indirection-free hot path | blr/brk in asm census | 0/0 | 0 blr / 0 brk | 0 (hard) |

Measurement cites: #5/#8 (`waves/w3-l1-ingress.md:63,69-70` - 10.20
fastest / 10.30 median, <15 target MET, MEASURED; quiet host, divan
fastest/median columns — contended divergence registered as
`11-measured-claims.md` row 28, 124.06 ns/frame slowest under
combined bw+cpu);
`02-pareto-levers.md:172` + `waves/a3-ring-sink.md` (`stage_commit_parquet`
269.1 us @256 / 4.368 ms @4096 = 1.05-1.07 us/tx vs 45.8 recorded,
<4 us/tx MET, MEASURED); #14 (`waves/w3-t2-asm-census.md:23-29` - 0 blr /
0 br xN / 0 brk, direct `bl` only, MEASURED census).

## Section 4 - Segment S3: agentic AI runtimes

Needs: deductive throughput high enough to feed agent loops; WAL
integrity across restarts; typed refusals the agent can branch on.

| CTQ characteristic | Metric | USL | Target | LSL |
|---|---|---|---|---|
| Deductive throughput | facts/s/core, closure bench | n/a | >=1M facts/s | stretch >50M |
| Durable replay | WAL restart replay divergence (B) | 0 | byte-identical | 0 (hard) |
| Typed refusals | Refusals carrying typed status | 0 untyped | 100% `CgapStatus` | 100% (hard) |
| DRed maintenance | Mechanism scope | n/a | retraction filter only | filter-vs-oracle |

Measurement cites for the DRed row: `dred.rs:116-127` (`retract` is a
bit-parallel retraction FILTER over static masks - it never follows
`derived_deps` as lineage edges); forensic probe
`adversarial/forensic-lane-05.md` (N=10,000 root-deletion chain leaves
9,999 survivors where 0 is correct; no existing court covers lineage
propagation). The 947M facts/s certifies bitmask tuple-invalidation
speed, not end-to-end DRed lineage unwinding. Errata receipt:
`waves/w6-dred-relabel-errata.md`.

Measurement cites: throughput is [TARGET-v26.11.2] - #26: RDFox never
executed this campaign; cgap closure facts/s unmeasured (MW4 receipt
absent); comparison arm is the published 2-6.1M facts/s envelope
(`06-bench-protocol.md:109-120`, `00-ard-prd.md:59-60`). Per
`11-measured-claims.md` Section 2, ns/frame vs facts/s is a category
error - no row converts one into the other. WAL + typed-refusal rows
cite A05 (WAL framing) and A19 (`CgapStatus` + four sovereign
entrypoints); campaign restart-survival receipts are [TARGET-v26.11.2].

## Section 5 - Segment S4: high-assurance

Needs: every admitted claim is receipted; gates fail closed and are
proven non-vacuous; frozen pins never drift.

| CTQ characteristic | Metric | USL | Target | LSL |
|---|---|---|---|---|
| Receipt coverage | Claims with file:line cite | 0 unreceipted | 100% | 100% (hard) |
| Court non-vacuity | Mutation-court flips | 0 unflipping | every court flips | >=1 (hard) |
| License pin | Attestation URN drift | 0 drift | byte-exact (A17, 80 B) | 0 (hard) |
| Seed non-conflation | Frozen seeds present, distinct | 0 conflations | pair intact | 0 (hard) |

Measurement cites: #19/#20 (`waves/w3-t5-pack-requal.md:20,74-75`,:50 -
14/14 gates green, 14/14 flip battery, 0 vacuous; lock hash
`blake3:cc8c83d3...671986` reproduced, MEASURED); #23/#24/#25 (court
flips: L2 shift corruption exit 101, L5 demote exit 101, L1
escalate-always-true, all sha256-exact restores, MEASURED); #22
(`waves/w3-t7-hygiene.md:13-14,59` - seed spot-check PASS, MEASURED).
License URN pin: `urn:avec:license:p28-sil:v2.1.0-sovereign`; seed pair:
`crates/cgap-parquet/src/bitslice.rs:687-699`.

## Section 6 - CTQ rollup

| Segment | CTQs | MEASURED rows | TARGET rows |
|---|---|---|---|
| S1 enclave OEMs | 4 | 3 | 1 (linked-ELF ceiling) |
| S2 real-time | 3 | 3 | 0 |
| S3 agentic AI | 4 | 1 (DRed filter mechanism scope) | 3 (throughput, WAL, typed refusals) |
| S4 high-assurance | 4 | 4 | 0 |

Layout note (CGAP-PRD-05): CSCB fact frame = 24 B (3x u64 LE); 64 B is
cgap-arrow Quad64 only. audit/cgapprd-03-frame-layout.md.
