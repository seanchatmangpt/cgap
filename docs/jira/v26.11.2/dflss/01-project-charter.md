# 01 Project Charter (DFLSS DMADV)

DFLSS Define-phase charter for the v26.11.2 starDF campaign, scoped to
the commercial lanes: cgap-core, cgap-bn3, branchless kernels, cgap-git,
cgap-blake3, cgap-storage, csct. Extends the campaign charter
`../14-dflss-charter.md` (which stays canonical for the workspace-wide
G1-G6 goals, SIPOC, tollgates, FMEA) and the VOC/CTQ tree
`../15-ctq-tree.md` (canonical for segments S1-S4). This file binds the
csct/cscb commercial lanes to the same DMADV spine: scope, sponsor,
problem, SMART goals, tollgates. The per-CTQ cells (target / measured /
receipt / status) live in `02-voc-ctq-matrix.md` in this directory ---
that file is the canonical per-cell register for CTQ-01..07; sections 2
and 3 of `../15-ctq-tree.md` remain canonical for the S1-S4 hardware
CTQs they already cover.
## Sponsor and executor

Sean Chatman (portfolio control plane). Executor: cgap lane fleet, one
canonical checkout at `/Users/sac/cgap`, branch main. Lane leases and
git transitions belong to the coordinator; this charter is a
documentation lane (DF-1) and owns only the three files listed in its
receipt `../lanes/lane-dflss-01-charter.md`.

## Problem statement (lane-scope)

The csct commercial surface (RFC-0042 challenger-sales suite + CSCB
courts) carries seven CTQs whose numbers circulated as dispatch prose.
Each figure had to be re-derived from receipts or re-run this session
before it could be admitted into a charter. Where a dispatch figure and
disk disagree, disk wins and the divergence is recorded in the receipt.

## SMART goals (lane scope)

| # | Goal | Measure | Target | Status |
|---|------|---------|--------|--------|
| D1 | Zero heap allocation on ingress | cscb_c4_alloc window | 0 allocs / 0 bytes | VERIFIED |
| D2 | Ingress latency | batch64-amortized p99 | <= 10.0 ns/frame | VERIFIED-CONDITIONAL |
| D3 | Kernel SRAM surface | verify_cscb_sram.py | <= 131,072 B | VERIFIED (36,048 B) |
| D4 | Receipt seal rate | cscb_c7 median vs floor | >= 1,000,000/s | VERIFIED (~3.89M/s) |
| D1 | Zero heap allocation on ingress | cscb_c4 window | 0 allocs / 0 B | VERIFIED |
| D2 | Ingress latency | batch64 p99 | <= 10 ns/frame | VERIFIED-COND |
| D3 | Kernel SRAM surface | sram gate | <= 131,072 B | VERIFIED (36,048 B) |

## Tollgates (lane scope)

| Tollgate | Phase | Exit criterion | Standing |
|----------|-------|----------------|----------|
| T1 | Define | charter + CTQ matrix landed with receipted cells | DONE (these two files) |
| T2 | Measure | every cell in 02-voc-ctq-matrix.md re-derived this session | DONE |
| T3 | Analyze | 25 ms closure aspiration: closed at 60 s DECIDED or re-gated | OPEN |
| T4 | Design | lane-mv-5 receipt lands (or the MV-5 re-scope is re-derived) | OPEN |
| T5 | Verify | cscb_c6 p99 mislabel fixed per FIX-04 sibling audit | OPEN |

## Anti-vacuity law (per goal)

Every goal above is bound to a court that must flip: D1 CSCB_MUTATE=alloc
breach (exit 1 witnessed), D2 mutate-mode scan injection (p99 5 -> 28 ns
in the b01 receipt), D3 sram-bloat mutant 136,068 B, D4 CSCB_MUTATE
mutated-certificate rejection, D6 allowlisted residual census, D7
zeroed-spark mutant (`court_flips_on_zeroed_spark`). A court that cannot
flip is FAIL; skips are loud.
