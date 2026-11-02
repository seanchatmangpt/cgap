# PP-1 Paper Audit (court-auditor, read-only both repos)

Date: 2026-11-01. Repos: /Users/sac/cgap (HEAD cfd085a9), /Users/sac/stardf.
Scope: claim-by-claim classification of the architectural position paper against waves/ receipts.

## 1. Paper location

The position paper CANNOT be located as described. Repo-wide grep of both repos
(excluding .git/, target/) for every cited marker returns ZERO matches:
`cgap-sip`, `cgap-dred`, `stardf-inline`, `3.42B`, `500x`, `RDFox`, `i-cache`,
`position paper` (case-insensitive). The nearest subjects are:

- /Users/sac/cgap/docs/CHALLENGER_SALES_REPORT.md (claims doc)
- /Users/sac/cgap/docs/REVENUE_GENERATION_COMPENDIUM.md (claims doc)
- /Users/sac/cgap/docs/jira/v26.11.1/waves/challenger-sales-report.md (its receipt)
- /Users/sac/stardf/docs/parity-vs-ost.md (OST/RDFox capability frame, no numbers)

These documents carry 973.5/2.819 but in a DIFFERENT context than the task
describes: `range_scan_cgap_vertical_bitslice` (vertical bit-slice range scan,
cgap-parquet::bitslice) and `stream_compact_cgap_challenger` (branchless
compaction, cgap-parquet::stream_compact) -- NOT "cgap-sip bit-plane
semi-joins" and NOT "cgap-dred compaction". No module named cgap-sip or
cgap-dred exists (grep: zero matches; no semi_join/bitweav module in crates/).
This is finding F1 (conflation).

## 2. Classification table

| # | Claim | Verdict | Evidence |
|---|---|---|---|
| C1 | 973.5 ns bit-slice range scan, 4,096 rows | MEASURED (reproduced) | waves/challenger-sales-report.md:13; my re-run `cargo bench -p bench --bench challenger_parquet` exit 0 -> 978.7 ns fastest / 989.1 ns median (within variance) |
| C2 | 2.819 ns branchless stream compaction | MEASURED (reproduced) | waves/challenger-sales-report.md:15; my re-run exit 0 -> 2.738-2.86 ns across mask selectivities |
| C3 | cgap-sip bit-plane semi-joins | INVENTED | zero repo matches for cgap-sip/semi_join/bitweav in crates/; no such module |
| C4 | cgap-dred compaction (2.819 ns) | INVENTED (name) / MEASURED (number, different subject) | cgap-dred does not exist; 2.819 ns belongs to stream_compact (C2) |
| C5 | stardf-inline bit-63 structural folding | INVENTED | zero matches for stardf-inline in /Users/sac/stardf; no bit-63 folding code found |
| C6 | v26.11.2 | INVENTED | zero repo matches for v26.11.2 in both repos |
| C7 | Apple M3 Max host | MEASURED / factual | sysctl machdep.cpu.brand_string = "Apple M3 Max"; waves/bench-ingress-receipt.md:33 cites same host |
| C8 | i-cache miss ~0.00% | INVENTED | zero matches for i-cache/icache/0.00% in any receipt; no perf/cstat evidence |
| C9 | 82.24 / 78.94 GB/s bn3 scan | MEASURED | waves/bench-bn3-receipt.md:45,48,69-70 (Divan fastest, 64k / 1M rows) |
| C10 | 6.499 ns/pair unify | MEASURED | waves/bench-bn3-receipt.md:71,77 (26,620 ns / 4,096 pairs = 6.499023) |
| C11 | 3.116 ns/eval builtin dispatch | MEASURED | waves/bench-bn3-receipt.md:40-41,73,78 (204,200 ns / 65,536 = 3.115845; receipt writes 3.115845) |
| C12 | ~460 ns p28 attestation verify | MEASURED | waves/bench-seal-receipt.md:64-65 (470.9 us/1024 and 30.09 ms/65536 -> ~460/~459 ns/record) |
| C13 | 49.95 us 0-B 0-alloc rle decode direct | MEASURED | waves/bench-parquet-receipt.md:77,99 |
| C14 | 30,157 B riscv object + 150 B ELF | MEASURED but CONFLATED | waves/gate-5-silicon.md:15 (30,157 B = pre-link rlib OBJECT, blake3::hash 27,914 B dominant) vs :4,19 (150 B = linked DEMO ELF whose .oabi/.bn3 sections are court demo bytes 0xA7/0xB3, per :70). Compendium lines 341/394 present "30,157 bytes of RISC-V flash" as one figure; the object and the ELF are different subjects and the ELF is not the kernel. |
| C15 | nm 0/0 (0 undefined symbols) | MEASURED | waves/a01-finish-receipt.md:62,122,135 (0/0 with flip + sha256-exact restore) |
| C16 | 0 blr / 0 br xN / 0 brk asm census | MEASURED | waves/a13-finish-receipt.md:39,119,143; a13-wave9-audit-receipt.md:112 |
| C17 | "3.42B triples/s" | INVENTED | zero repo matches; no receipt derives it. Arithmetic check: nearest measured rates are 320.9M rule evals/sec/core (C11), 153.8M joins/sec/core (C10), 6.05B values/sec bit_unpack (compendium:284). No combination yields 3.42B; ~10.7x the measured eval rate. |

## 3. Category errors (flags for lanes 2-4)

- CE1 Scan-bandwidth vs materialization-throughput: no such comparison exists
  in either repo (zero RDFox matches). If a paper makes it: 82.24 GB/s is a
  sequential byte-scan bandwidth over a 1.57-25.1 MB resident array; RDFox
  2-6.1M facts/sec is materialization throughput (Datalog closure over rule
  firing). Different units, different workload, different memory residency.
  Incommensurable without a normalized harness. Additionally, characterizing
  RDFox internals (vtables / pointer thrash) without running RDFox is
  unverified assertion, not measurement.
- CE2 500x-1500x claim: absent from both repos (zero matches). Cannot be
  grounded by any receipt listed here; largest measured multiple in-tree is
  4.06x (bitslice scan vs horizontal incumbent) and 82.2x-vs-charter-target
  (a target multiple, not a competitor multiple).
- CE3 82.24x-as-speedup: REVENUE_GENERATION_COMPENDIUM.md:35 claims "4.06x to
  82.24x empirical speedups" -- 82.24 is GB/s (a bandwidth), not a multiple.
  Category error inside the repo's own claims doc.
- CE4 C14 conflation (object vs linked ELF vs demo-byte ELF) as above.

## 4. Commands run this audit (all mine)

1. grep marker sweeps both repos (sip/dred/stardf-inline/position paper/3.42B/500x/RDFox/i-cache/v26.11.2) -> 0 relevant matches, exit 1 per grep
2. sysctl -n machdep.cpu.brand_string -> "Apple M3 Max"
3. cargo bench -p bench --bench challenger_parquet -> exit 0 (reproduces C1/C2)
4. receipt re-reads: bench-bn3-receipt.md, bench-seal-receipt.md, bench-parquet-receipt.md, gate-5-silicon.md, a01-finish-receipt.md, a13-finish-receipt.md, challenger-sales-report.md

## 5. Standing

PARTIAL_ALIVE. The measured-claim inventory (C1,C2,C7,C9-C16) is grounded and
C1/C2 were independently reproduced this session. The paper as described in the
task does not exist in either repo; all "known-expected invented" markers
(C3-C6,C8,C17) are confirmed absent with zero matches. Lanes 2-4: correct the
paper against rows C3-C6, C8, C14, C17 and CE1-CE4. Lane 7: C14 is your
correction source (object/ELF conflation + demo-byte ELF).
