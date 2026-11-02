# cgap Position Paper v2 (epistemically grounded)

Issued 2026-11-01 by lane PP-7-finalize, /Users/sac/cgap @ main (HEAD
cfd085a9). Supersedes: nothing — no v1 text exists on disk (see
position-paper-v1-archived.md, an absence marker; the v1 claim inventory
survives only via waves/paper-audit.md rows C1-C17). This issue is the
first epistemically grounded form of the paper: every number carries a
receipt cite or a TARGET tag; every comparative claim is like-with-like
or marked architectural-difference-only; v26.11.2 appears only as
roadmap, never as released.

Tagging law (non-negotiable, enforced in the self-audit section):
`(receipt: <file>)` for a number carried by a receipt on disk;
`TARGET` for an aimed-for number with no measuring receipt; nothing
else is lawful.

## 1. The substrate thesis (factual)

cgap is a sovereign `#![no_std]` columnar substrate: Arrow C Data
Interface kernels, BLAKE3, Parquet persistence, and Git packfile v2 as
zero-external-dep kernel crates behind a `cgap` umbrella facade. On top
of it the v26.11.1 moonshot campaign added a binary ontology layer
(B-ONT scalars, BoSym slabs, BN3 rules, P28-SIL licensing, cgap-gate,
cgap-crypto, cgap-storage) whose property of interest is not a single
headline number but a verification posture: every capability lands with
a court (a test that can flip), a sha256 fail-closed restore, and a
receipt naming its commands and exits. The thesis is narrow: fixed-width
slot arenas + branchless word-parallel kernels + court-receipted
verification give deterministic, allocation-free hot paths that are
auditable at the byte level. Nothing in this paper claims parity with,
let alone superiority to, a deployed RDF triple store on materialization
workloads; section 4 states why such a comparison is not yet lawful.

## 2. Measured-results table (receipt-cited, re-witnessed 2026-11-01)

All rows below were re-run first-hand this session by PP-5
(waves/paper-rewitness.md) unless noted; verdict column is PP-5's.
Host: Apple M3 Max (receipt: waves/paper-audit.md row C7, sysctl).

| # | Claim | Value | Tag / receipt | PP-5 re-witness |
|---|---|---|---|---|
| M1 | bitslice range scan, vertical bit-plane, 4,096 rows | 973.5 ns original; 989.2 ns (PP-2 divan) / 990 ns (PP-2 harness) / 978.7 ns (PP-1 re-run) | (receipt: waves/challenger-sales-report.md; receipt: waves/paper-sip.md) | MATCH within ~1% |
| M2 | scan speedup vs horizontal incumbent | 4.06x (vs 3.957 us); 4.42x in PP-2's run (incumbent 4.374 us, host variance) | (receipt: waves/paper-sip.md) | — (PP-2 first-hand) |
| M3 | two-hop semi-join (2 scans + mask-AND) | 2,073 ns / 4,096 rows; composition is scan-dominated (~97%+); mask-AND alone below 41 ns timer precision | (receipt: waves/paper-sip.md) | — (PP-2 first-hand) |
| M4 | SWAR lineage-bitmask retraction (dred), 65,536 facts | 0.7833-0.7845 ns/lane release; entropy-flat max/min 1.002 (1.006 debug) | (receipt: waves/paper-dred.md) | — (PP-3 first-hand) |
| M5 | branchless stream compaction | 2.738-2.86 ns per 8-lane compact (0/50/100% selectivity); 2.819 ns cited originally | (receipt: waves/challenger-sales-report.md; receipt: waves/paper-audit.md row C2; TARGET reconciliation: waves/paper-dred.md) | PP-1 reproduced exit 0 |
| M6 | bn3 statement_scan_24b, N=65,536 | 82.24 GB/s isolated Divan scan; re-run 81.70-81.88 GB/s | (receipt: waves/bench-bn3-receipt.md:69; receipt: waves/paper-rewitness.md) | MATCH -0.5% |
| M7 | bn3 statement_scan_24b, N=1M | 78.94 GB/s; re-run 78.60-79.00 GB/s | (receipt: waves/bench-bn3-receipt.md:70; receipt: waves/paper-rewitness.md) | MATCH +-0.5% |
| M8 | bn3 unify_join | 6.499 ns/pair (26.62 us / 4,096 pairs, exact) | (receipt: waves/bench-bn3-receipt.md:71; receipt: waves/paper-rewitness.md) | MATCH exact |
| M9 | bn3 builtin dispatch | 3.116 ns/eval (204,200 ns / 65,536) | (receipt: waves/bench-bn3-receipt.md:40-41,73,78; receipt: waves/paper-audit.md row C11) | — |
| M10 | p28 attestation verify | ~460 ns/record (470.9 us/1024; 459.96 ns in PP-5 re-run); FFI direct C-ABI leg: loud [SKIP] (exports not linked, receipted in-band) | (receipt: waves/bench-seal-receipt.md:64-65; receipt: waves/paper-rewitness.md) | MATCH |
| M11 | rle_decode_direct, 0-B | 49.95 us receipted; 46.33-46.41 us re-run (faster side of band); 0 allocs / 0 B max alloc | (receipt: waves/bench-parquet-receipt.md:77; receipt: waves/paper-rewitness.md) | MATCH -7.3% |
| M12 | riscv32imc object-class size (pre-link, --profile audit) | 30,157 B at gate-5; 28,355 B in PP-5 re-run — DRIFT, named non-fatal: blake3::hash 26,047 B vs 27,914 B (-6.7%, compile-output drift; tree moved since receipt), seal_with_mask x3 2,242 B and .rodata 1 B byte-exact, shape identical | (receipt: waves/gate-5-silicon.md:15; receipt: waves/paper-rewitness.md) | DRIFT (named) |
| M13 | linked-ELF subject (authoritative per spec section 4) | flash 798 B / sram 0 B (xip_t1_riscv_elf_size_court exit 0). The gate-5 150 B figure was a different subject: a linked DEMO ELF whose .oabi/.bn3 sections are court demo bytes (0xA7/0xB3), not the kernel — PP-1 row C14 conflation, corrected here | (receipt: waves/gate-5-silicon.md:4,19,70; receipt: waves/paper-rewitness.md; receipt: waves/paper-audit.md row C14) | — |
| M14 | alloc/panic symbol purity, carved riscv objects | 0 alloc / 0 panic symbols, every object | (receipt: waves/gate-5-silicon.md; receipt: waves/paper-rewitness.md) | MATCH |
| M15 | A13 zero-indirection asm census | 0 blr / 0 br xN / 0 brk; exactly 3 direct bl; 1 adrp (238 raw lines) | (receipt: waves/a13-finish-receipt.md:38-43; receipt: waves/paper-rewitness.md) | MATCH line-for-line |
| M16 | nm undefined symbols | 0/0, with flip + sha256-exact restore | (receipt: waves/a01-finish-receipt.md:62,122,135; receipt: waves/paper-audit.md row C15) | — |
| M17 | SIP semi-join asm purity | semi_join_masks 76 / semi_join_two_hop 210 / scan_range_vertical_128 339 / scan_range_vertical_counted 208 asm lines, each 0 blr / 0 br xN / 0 brk; bl sites panic/bzero/alloc/collaborator only, zero inside loop spans | (receipt: waves/paper-sip.md) | — |

Notes on M1-M17: the 82.24 GB/s figure (M6) is the ISOLATED-scan number
and must never be cited as end-to-end chain throughput; the isolated vs
chain split is 80.32 GB/s isolated / 12.3 GB/s chain (receipt:
waves/bench-chains-bn3.md:37). Divan-level alloc profiling of the
two-hop semi-join path has NOT been run (UNVERIFIED; receipt:
waves/paper-sip.md "Leaves").

## 3. TARGET-roadmap (unbuilt or unmeasured — honestly)

Everything in this section is a target or an open measurement, NOT a
released capability. Per the tagging law, inventoried numbers from the
v1 claim set that no lane built are carried with the inventor's number
deleted; they are re-admitted only when a receipt measures them.

- T1. Materialization (Datalog fixpoint) throughput of cgap-bn3:
  TARGET, unmeasured. The RDF-1 wave's lanes own this measurement
  (receipt: waves/paper-comparison-audit.md row R3). Until it lands,
  no performance-delta phrasing against any triple store is lawful.
- T2. RDFox performance comparison: TARGET, blocked on T1. Any such
  comparison must be like-with-like (same workload, same units, same
  memory residency) under a normalized harness. Largest currently
  measured in-tree multiples are 4.06x (M2) and 82.2x-vs-charter-target
  (a target multiple, not a competitor multiple) — receipt:
  waves/paper-audit.md section 2 CE2. The v1 "500x-1500x" figure is
  deleted: zero repo matches, no receipt derives it (receipt:
  waves/paper-audit.md row CE2).
- T3. "3.42B triples/s": deleted as an invented claim. No receipt
  derives it; nearest measured rates are 320.9M rule evals/s/core (M9
  family), 153.8M joins/s/core (M8 family), 6.05B values/s bit_unpack
  (compendium), and no combination yields 3.42B (receipt:
  waves/paper-audit.md row C17). Re-admissible only via a measuring
  receipt.
- T4. i-cache miss ~0.00%: deleted as invented. Zero receipt evidence;
  re-admissible only via a perf/cstat measuring receipt (receipt:
  waves/paper-audit.md row C8).
- T5. "v26.11.2" as a version: roadmap only. Zero repo matches; no
  v26.11.2 artifact is released (receipt: waves/paper-audit.md row C6).
- T6. stardf-inline bit-63 structural folding: TARGET-IN-FLIGHT at
  issue time. Lane PP-4's receipt (waves/inline-folding.md) was absent
  at the PP-7 poll deadline (~25 min, 30 s intervals; five of six input
  receipts present). No number is stated for it. This section upgrades
  to measured text only when inline-folding.md lands.
- T7. cgap-dred Divan harness: the 2.819 ns figure remains TARGET for
  the dred surface pending a divan harness wired to dred (receipt:
  waves/paper-dred.md "Standing"); the measured dred numbers are M4.
- T8. End-to-end chain throughput as a headline: the chain number is
  12.3 GB/s (receipt: waves/bench-chains-bn3.md:37), not 82.24 GB/s;
  any headline must cite the chain figure for chain workloads.

## 4. Corrected RDFox section (architectural difference ONLY)

Lawful status: cgap has NOT executed RDFox, and cgap's own
materialization throughput is unmeasured (T1). Therefore this paper
makes NO performance claim against RDFox — no ratio, no multiple, no
"speedup". The only lawful statements are architectural differences,
and the RDFox side is characterized from RDFox's public materials, not
from assertion about its internals.

- A1. Storage shape: cgap-bn3 uses a fixed-width 24-byte cache-aligned
  statement slot arena (scanned at M6/M7 rates). RDFox, per its public
  materials, uses dictionary encoding with compressed integer indexes
  over a columnar layout. Difference, not ranking.
- A2. Unit discipline: cgap's receipted 82.24 GB/s (M6) is sequential
  byte-scan bandwidth over a resident array. RDFox's public figure of
  2-6.1M facts/s is materialization throughput (Datalog closure over
  rule firing). Different units, different workload, different memory
  residency — incommensurable without the T1/T2 normalized harness
  (receipt: waves/paper-audit.md section 3 CE1). Scanning bandwidth is
  not rule-materialization throughput.
- A3. Corrections carried from PP-6 (receipt:
  waves/paper-comparison-audit.md): the "Standard graph engines
  (<100 MB/s)" baseline and its "82x" ratio are deleted (unnamed
  baseline; also internally conflated with a 1.0 GB/s wire-rate margin
  in the same row, R4); "Replaces slow RDF/SPARQL/OWL/Prolog engines"
  is restated as an architectural difference — no RDF/SPARQL/OWL/Prolog
  engine was executed in any cgap receipt (R5). RDFox-internal
  characterization as naive pointer tries was withdrawn: contradicted
  by RDFox's documented dictionary-encoded compressed-integer design
  (R2).
- A4. What would make a performance comparison lawful: (a) cgap
  materialization throughput measured (T1); (b) RDFox executed on the
  same workload/host with a stated method; (c) like-for-like units.
  Until all three, this section admits architecture prose only.

## 5. Falsifiers

Each claim class names the observation that would kill it.

- F-M1..M5 (bitslice / compaction / dred kernels): any differential
  court flip — e.g. the PP-2 mutant dropping the `lo_lt` bound over-
  admits 827 rows (exit 101, receipt: waves/paper-sip.md); the PP-3
  alt-support mutant fails all three courts (exit 101, receipt:
  waves/paper-dred.md). A restored-pass-after-flip cycle that cannot
  flip marks the court vacuous (FAIL).
- F-M6..M9 (bn3 rates): a Divan re-run on the pinned host deviating
  beyond host-noise band (PP-5 used ~-0.5%-to-1% bands) without a
  named compile-output or tree change; or an asm census finding a
  branch/indirection inside a loop span.
- F-M12/M13 (silicon sizes): a re-run whose per-symbol shape (not just
  total) drifts — PP-5's drift was non-fatal precisely because shape
  held byte-exact while one symbol moved -6.7%.
- F-T1/T2 (comparison posture): a receipt measuring cgap materialization
  throughput INSTANTLY converts section 4 from architecture-only to
  comparison-eligible; conversely, any performance ratio vs RDFox
  appearing in a cgap doc before T1 lands falsifies this paper's
  discipline claim.
- F-T6 (inline folding): landing waves/inline-folding.md with measured
  numbers converts T6 from TARGET-IN-FLIGHT to measured; its continued
  absence past the next wave falsifies any "in flight" phrasing.
- F-whole-paper (self-audit falsifier): a single number in this file
  lacking a `(receipt: ...)` or TARGET tag falsifies the tagging law.
  The audit command and its exit are recorded in waves/paper-finalize.md.

## 6. Self-audit

grep-based audit for untagged numerals: the receipt
(waves/paper-finalize.md) records the exact command and exit; the
required outcome is zero untagged numbers. Sections 3 and 4 were
additionally swept for banned phrasing (released-reading v26.11.2,
performance deltas vs RDFox, unsourced baselines) — none present.
