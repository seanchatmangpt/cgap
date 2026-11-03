# claims-appendix.md

Companion to `position-paper-v3.md` (same directory). Maps each
quantitative claim in the paper to a receipt path:line on disk or an
explicit placeholder class. Compiled by lane D10 (Design), 2026-11-02,
/Users/sac/cgap @ main. Numeric source of record:
`../11-measured-claims.md` — no number appears in the paper that is
absent from that table or derived from its cited receipts.

## A. Quantitative claim register

| # | Claim (paper section) | Value | Receipt path:line / placeholder |
|---|---|---|---|
| Q1 (s1) | SlotId statement row width | 24 B/row, 64 B line = 8 terms | ../13-arrow-star-schema.md:17-19 |
| Q2 (s1) | Paradigm table cites | layout / dispatch / verification rows | ../13-arrow-star-schema.md:17-19; ../12-ard-prd-cgap.md:104-110,:123-126 |
| Q3 (s2) | CAS negative-surface grep | exit 1, 0 hits | ../waves/w2-07-ost-greps.md via ../03-ost-design-around-register.md:30-43 |
| Q4 (s2) | bitslice.rs surface | 930 lines | ../03-ost-design-around-register.md:43-48 |
| Q5 (s2) | unify.rs register ABI | R0..=R255 at unify.rs:100,169 | ../03-ost-design-around-register.md:66-77 |
| Q6 (s2) | G7-PATENT gate | 4/4 DISTINCT-MECHANISM, demote to 3/4 | ../00-ard-prd.md:74-77 |
| Q7 (s3) | nanoarrow / DuckPGQ / Kuzu | architecture prose only, no numbers | [CITE-TBD: version+date] x3 (paper s3) |
| Q8 (s4.1) | Bucket-head budget | 4,096 x 4 B = 16,384 B = 16 KiB | arithmetic over ../13-arrow-star-schema.md:51-52 (corrects ../13-arrow-star-schema.md:44 "32 KB") |
| Q9 (s4.1) | Full JoinScratchpad pair | 16 KiB + 256 KiB = 272 KiB | arithmetic over ../13-arrow-star-schema.md:51-52 |
| Q10 (s4.1) | Differential court sizes | N in {1, 64, 4096, 100_003} | ../plans/levers-L1-L5.md:77 |
| Q11 (s4.2) | Payload window | w=56 payload window | ../plans/levers-L1-L5.md:61; ../../v26.11.1/waves/a13-finish-receipt.md:77 |
| Q12 (s4.3) | L2 court flip | exit 101, sha256-exact restore | ../waves/w3-l2-swar-fnv.md:38-42 |
| Q13 (s4.3) | L2 chain did not move | 42.58-43.1 ms vs 41.54 ms @1M; <10 ms NOT MET | ../waves/w3-l2-swar-fnv.md:26-28,61-63; ../11-measured-claims.md:30-34 rows 9-11 |
| Q14 (s4.5) | rle_decode_direct | 49.95 us / 0 B max alloc | ../../v26.11.1/waves/bench-parquet-receipt.md:77 |
| Q15 (s4.5) | bit_unpack rate | 6.05B values/s | ../../v26.11.1/waves/paper-finalize.md:88-90 (compendium figure carried via v2) |
| Q16 (s5.1) | Pre-link object footprint | 28,679 B, 87.5% of 32,768 B | ../waves/w3-t3-silicon.md:36,50 |
| Q17 (s5.1) | Ingress chain | 10.20 fastest / 10.30 median ns/frame (quiet host, divan fastest/median; contended divergence row 28: 124.06 ns/frame slowest) | ../waves/w3-l1-ingress.md:63,69-70; ../11-measured-claims.md rows 5/28 |
| Q18 (s5.1) | Ring commit | 1.05-1.07 us/tx vs 45.8 baseline | ../15-ctq-tree.md:52-55; ../waves/a3-ring-sink.md |
| Q19 (s5.1) | Asm census | 0 blr / 0 br xN / 0 brk | ../waves/w3-t2-asm-census.md:23-29 |
| Q20 (s5.1) | Court flips L1/L2/L5 | exit 101 x3 | ../11-measured-claims.md:44-46 rows 23-25 |
| Q21 (s5.1) | Tamper refusal code | 0x5032_38FF exact, both directions | ../15-ctq-tree.md:28-34; ../waves/a3-ring-sink.md:50; sovereign.rs:83 |
| Q22 (s5.1) | Seed pair intact | bitslice.rs:687-699 | ../waves/w3-t7-hygiene.md:13-14,59 |
| Q23 (s5.2) | Whole-envelope ceiling | 131,072 B (128 KiB), <115 KiB target | ../15-ctq-tree.md:23,:36-38; ../16-sipoc.md:46 |
| Q24 (s5.3) | Attestation law | conservation law form | ../12-ard-prd-cgap.md:17-20 (Sigma_PQC [TARGET-v26.11.2]) |
| Q25 (s6 F1) | L3 target | >20x @1M [TARGET-until-w5-l3-remeasure lands] | ../plans/levers-L1-L5.md:54-77; no remeasure receipt exists |
| Q26 (s6 F2) | Closure bench target | >50M facts/s/core [TARGET-v26.11.2] | ../00-ard-prd.md:59-60; ../06-bench-protocol.md:109-120 |
| Q27 (s2) | Samsung acquisition fact | market context only | [CITE-TBD: acquisition, version+date] (paper s2) |
| Q28 (s5.1) | Linked-ELF ceiling | UNMEASURED this campaign | ../15-ctq-tree.md:36-38 |
| Q29 (s4.4) | Eytzinger multiple | none stated — design-spec only | no receipt; class deleted per paper s4.4 |
| Q30 (s4.3) | L2 attribution | REFUTED (encode/decode dominates) | ../waves/w3-l2-swar-fnv.md:61-63; ../11-measured-claims.md:34 row 11 |


## B. Citation-placeholder register

| # | Placeholder | Class | Close condition |
|---|---|---|---|
| C1 | [CITE-TBD: version+date] nanoarrow | public project citation | pin repo tag + access date before external release |
| C2 | [CITE-TBD: version+date] DuckDB-DuckPGQ | public project citation | pin version + access date before external release |
| C3 | [CITE-TBD: version+date] Kuzu | public project citation | pin version + access date before external release |
| C4 | [CITE-TBD: acquisition, version+date] Samsung | citation-class public fact | pin the acquisition (subject, date, source) before release |

## C. Divergences recorded

- D1. `../13-arrow-star-schema.md:44` states "4,096 bucket heads = 32
  KB L1"; the arithmetic is 4,096 x 4 B = 16 KiB for the bucket-head
  array. This paper states 16 KiB and records the divergence from the
  canonical contract; the contract file itself is outside D10's lease
  and was not edited.
- D2. Long lines: the paper has 19 lines >100 chars, the appendix has
  26; all are markdown table rows (exempt-by-convention per v2
  finalize divergence D2; receipt:
  ../../v26.11.1/waves/paper-finalize.md:34-36).
- D3. The dispatch named a "20-lane finishing plan" as assembly
  context; no such plan document was located on disk (ls of v26.11.2
  root + waves/ + plans/ checked). Section assembly followed the
  dispatch text itself. Recorded as a loud divergence.
- D4. The untagged-numeral audit leaves 28 residual digit-bearing
  lines after paragraph-scoped tagging; all 28 are classified lawful
  (identifier, cite-bearing path, tag-continuation, or named-deleted
  quotation — e.g. the quoted "32 KB" error under correction). Zero
  unlawfully untagged quantitative claims.
- D5. No court was executable by this text lane; anti-vacuity is
  carried by paper s6 falsifiers F1-F6, each backed by in-session court
  runs in the cited receipts (L1/L2/L5 flips exit 101). Loud, per the
  v2 finalize precedent (../../v26.11.1/waves/paper-finalize.md:49-57).

## D. Receipt tail (commands + exits, this session)

All commands run with cwd /Users/sac/cgap. Exits witnessed this
session.

```
# C1. Long-line census over the paper
awk 'length > 100 {c++} END{print c+0}' \
  docs/jira/v26.11.2/paper/position-paper-v3.md   # stdout: 19  exit 0
# C2. Long-line classifier (all 19 are markdown table rows)
awk 'length > 100 {print NR}' docs/jira/v26.11.2/paper/position-paper-v3.md
#   -> rows 33-35, 54-57, 204-215  exit 0 (all table rows)
# C3. Paragraph-scoped untagged-numeral audit (python3, method per
#     waves/paper-finalize.md): first pass 103 raw digit-lines without
#     an on-line tag; scoped pass leaves 28; all 28 classified lawful
#     per divergence D4.                          exit 0
# C4. Token sweep: on-disk read-back of both files via wc -l + the
#     audit passes above; no corruption found.
#     exit 0
```
