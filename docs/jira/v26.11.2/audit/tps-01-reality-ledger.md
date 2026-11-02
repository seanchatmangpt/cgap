# tps-01-reality-ledger

TPS-1 re-derivation of the TPS manual's §1 reality register from disk
(2026-11-02, read-only lane). NAMING CAVEAT: no file named "TPS" or
"manual" exists under docs/ (repo-wide grep) — rows re-derived from
the named commands + receipts. Nearest on-disk registers:
11-measured-claims.md §1 + audit/audit-ledger.md.

## §1 Corrected reality register

| station | disk state (fresh) | measured | class |
|---|---|---|---|
| cscb SRAM budget | verify_cscb_sram.py exit 0 | 36,048 B; gate 131,072; headroom 95,024 | CONFIRMED |
| SRAM anti-vacuity | --mutate=sram-bloat exit 0 | 136,068 B > bound — flips | CONFIRMED |
| disassembly invariants | verify_disassembly_invariants.py exit 0 | AE4 CLEAN: census + linked witness + M-AE4-1 + M-G03-E + self-check | CONFIRMED |
| audit-README blocker 8 (G20 exit 2) | fresh standalone run exit 0 | emit-asm build now green | STALE (repaired) |
| cgo floor darwin | receipt-cited (harness arm not cheap) | 85 ns/call (lane-ipc-03:44); 73 ns fence (lane-ax-05:68) | RECEIPT-CITED |
| cgo floor container | receipt-cited (colima down; starting = host state change) | 154 ns/call (lane-ax-05:68,82) | RECEIPT-CITED |
| mmap zero-copy darwin | receipt-cited | 4.52-4.54 ns/op (lane-ax-06:70) | RECEIPT-CITED |
| mmap zero-copy container | receipt-cited | 4.574-4.656 ns/op (lane-ax-06:70) | RECEIPT-CITED |
| penstock admission ns/frame | cargo bench chains-ingress stage_penstock_admission exit 0, 100 samples @1M | fastest 7.35 ns/frame; median 8.13; receipted 7.71 reproduces within drift | RE-DERIVED; manual 5.70 FAILED |
| ingress chain ns/frame | same fresh run | fastest 10.70; median 11.80; receipted 10.20/10.30 — drift, <15 target MET | RE-DERIVED; 10.20 DRIFTED |
| <15 ns/frame target | same | 10.70 < 15 — MET on fresh numbers | CONFIRMED |
| 10.1x vs P3-16 baseline | fresh 10.70 vs 102.7 | 9.6x fresh (10.1x receipted) | DRIFTED |

## §2 TPS-manual figures that FAILED re-derivation

1. "5.70 ns/frame penstock admission" (also REVENUE_GENERATION_
   COMPENDIUM.md:59,237,288) — not reproduced: receipted 7.71
   (w3-l1-ingress.md:64,71-72, already recorded NOT-reproduced);
   fresh 7.35 fastest. Must not be quoted as measured.
2. "10.20 ns/frame ingress (10.30 median)" — receipted figure stands
   (w3-l1-ingress.md) but fresh = 10.70/11.80. Drift, not
   refutation; target met; 10.1x -> 9.6x fresh.
3. audit/README.md blocker 8 — STALE (fresh standalone exit 0, full
   AE4 court CLEAN).

## §3 Receipt citations (re-read at use time)

lane-ipc-03:44,50 (85 ns floor); lane-ax-05:68,77,82 (154 ns
container; darwin single-admit 182.7-185.6 cited-not-rerun);
lane-ax-06:70-72 (mmap + pread deltas, 0 allocs both);
w3-l1-ingress.md:63-72,83. Container arms not re-run (colima down;
harness arms not cheap) — loud skips recorded.

## See Also

audit/audit-ledger.md · audit/tps-08-shipgate-plan.md ·
waves/w3-l1-ingress.md
