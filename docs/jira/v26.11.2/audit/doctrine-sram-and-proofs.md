# doctrine-sram-and-proofs

DOCTRINE-2 audit (2026-11-02). HYPOTHESIS UNDER AUDIT — every number
re-measured this session; claims ranked by witness strength.

## Pillar 3 — SRAM envelope (fresh run, exit 0 both modes)

- python3 scripts/verify_cscb_sram.py -> exit 0; measured 36,048 B
  (.text+.rodata+.data+.bss, riscv32imc linked ELF) <= 131,072 B,
  headroom 95,024 B. Toolchain rustc 1.101.0-nightly.
- --mutate=sram-bloat -> exit 0 flip: mutant 136,068 B > bound
  ("MUTATE COURT ... overflow flips (anti-vacuity holds)").
- Stack disclosure: max leaf frame 2048 B from 9 sp-adjusting
  prologues — script self-labels UNVERIFIED as worst-case chain bound.
- DO NOT CONFLATE: 39,184 B is the empirical baseline, REPORT-ONLY
  (lane-b04-cscb-c5.md:36-37,70). "39,184 -> 36,048 improvement, PASS"
  is a category error; the gate is the measured 36,048 alone.

## Zero-alloc witness inventory (instrumented > static)

TIER 1 — instrumented, positive controls witnessed:
1. soc_full_chip_court (CountingAlloc; lane-git-16).
2. cscb_c4_alloc: 0 allocs/0 bytes over 1M frames; MUTATE=alloc -> exit 1
   "ALLOC BUDGET BREACH allocs=1 bytes=8" (lane-b03:15-20).
3. C03 e5 edge court: AE7, positive control witnessed
   site="c03-positive-control" (lane-c03:71-75).
4. G07 conformance seal: crate-single counting allocator + positive
   control (lane-g07:15-19,45-46).
5. GIT-07 egress zero_alloc_serve_court: snapshot_owned delta>0 control
   (lane-git-07:50-54,63).
6. BONUS: cgap-blake3 tests/zero_alloc.rs — CountingAlloc, n==0 after
   real 64 MiB hash (:25,:55,:118).

TIER 2 — static only (design intent, NOT proof):
- object_store (git-05): banned-token sweep, self-labeled "source-scan,
  not allocator count" (lane-git-05:29,42).
- delta_resolver (git-04): source-scan; "no tracking-alloc instrument"
  (lane-git-04:39-40).
- mcp_driver (git-14): WEAKEST — receipt contains ZERO "alloc"
  occurrences (grep verified); no instrument, no scan.

## Pillar 4 — proof chains

Methodology (EXP-04 + FIX-04): HONEST, 2 weak points. FIX-04 APPLIED in
cscb_c7_receipt_sealing.rs:133-140 — labels now min/median/max with the
no-p99 disclosure. RESIDUAL: cscb_c1_c2_ingress.rs:190-194 still labels
max as p99 (lane-fix04:33-38) — same rename not applied there.
Fresh re-run this session (release, exit 0): min 3.57M / median 3.95M /
max 4.37M receipts/s, runs=7; 1M-receipt chain validity PASS; gate
>=1M/s PASS. B06's "4.35M" was its run's median; honest claim: "median
~3.95-4.35M/s, machine-dependent; gate is the 1M floor". Pipeline scope
(frame+hash+verify), not a bare hash loop.

Tamper-refusal court of record: cscb_c7 verify_against_chain — nonces
strict, parent==prev digest (genesis zero), digest re-derivation
(:73-96); bitflip court + vacuity check (bitflip MUST change digest)
(:174-202). IN-CRATE GAP CONFIRMED STILL OPEN: cgap-blake3's own tests
carry NO bitflip-rejection or chain-mutation test (structural parity
only); the tamper courts live only in the external csct bin.

## Per-claim table

| claim | witness | strength |
|---|---|---|
| SRAM 36,048 B gate PASS | fresh script run | STRONG |
| sram-bloat flip 136,068 B | same script | STRONG |
| "39,184 -> 36,048 improvement" | cross-method, report-only | NARRATIVE-ONLY |
| leaf frame 2048 B worst case | self-disclosed UNVERIFIED | WEAK |
| 4.35M receipts/s | run median; today 3.95M | MEDIUM (gate 1M) |
| "p99" statistic | renamed in c7; ingress residual | MEDIUM |
| chain parent==prev enforced | c7 external court | STRONG but EXTERNAL |
| bitflip tamper-refusal | c7 external court | STRONG but EXTERNAL |
| zero-alloc c4/e5/egress/G07/soc/blake3 | instrumented + breaches | STRONG |
| zero-alloc object_store/delta_resolver | source-scan, self-disclosed | WEAK |
| zero-alloc mcp_driver | no witness at all | WEAKEST |

## 3 weakest links

1. Tamper-refusal/chain-mutation proof lives ONLY in the external csct
   bin — a cgap-blake3-only rebuild would lose the bitflip court.
2. mcp_driver zero-alloc has no witness of any class.
3. "p99" mislabel persists in cscb_c1_c2_ingress.rs:190-194.

## See Also

audit/dod-cgap-blake3.md · lanes/lane-fix04-cscb-p99.md ·
lanes/lane-b04-cscb-c5.md · lanes/lane-b06-cscb-c7.md
