# 14 DFLSS Charter

Project Sovereign Silicon — CGAP microkernel substrate. DMADV (Define lane D1)
charter for the v26.11.2 starDF campaign (`00-ard-prd.md`, runbook
`OVERNIGHT-LOOP.md`).

## Sponsor

Sean Chatman (portfolio control plane). Executor: cgap lane fleet, one
canonical checkout at `/Users/sac/cgap`, branch main.

## Business Case

CGAP is a sovereign `#![no_std]` columnar/deductive substrate. The commercial
claim — deterministic replay, fixed memory, predictable latency — is only
bankable if the microkernel meets hard physical ceilings, not averages.
DMADV gives the campaign a Define-phase charter: SMART goals, CTQ flowdown,
SIPOC, tollgates, and an FMEA, so every wave receipt lands against a stated
measure instead of a narrative.

## Problem Statement

The substrate's hot paths (ingress, commit, unify, seal) are claimed
deterministic but lack a single governed Define artifact binding each claim
to a CTQ, a measurement method, and a receipt. Without it, footprint and
latency numbers circulate as prose (e.g. the unverified 55.7 KB parquet
figure) instead of receivable facts.

## Goal Statements (SMART)

| # | Goal | Measure | Target | Status |
|---|------|---------|--------|--------|
| G1 | .text ceiling | linker ASSERT | <= 131,072 B | [TARGET-v26.11.2] |
| G1a | Arrow object ceiling | llvm-size -A | 28,679 B / 32,768 | MEASURED |
| G1b | Parquet object baseline | llvm-size -A | 55,711 B pre-link | UNVERIFIED-BY-RECEIPT |
| G2 | Zero heap allocation | nm alloc symbols | 0 alloc, 0 panic (ring/demux ingress path; bn3 fixpoint allocates — scoped OUT per red-09:74-78) | MEASURED (SCOPED) |
| G3 | 0 blr / 0 brk kernels | objdump census | 0 blr, 0 br xN, 0 brk | PARTIAL (1 blr) |
| G4 | Ingress latency | per-frame hot loop | < 15 ns/frame | [TARGET-v26.11.2] |
| G5 | Commit latency | per-tx FFI ring | < 4 us/tx | MET (0.589-0.603) |
| G5a | Escalation law | skip-not-pay replay | hold at 10.20 ns/frame (quiet host; divergence row 28) | RECEIPTED |
| G6 | Throughput | facts per second per core | >= 1,000,000 | [TARGET-v26.11.2] |

Goal receipt notes (paths relative to `docs/jira/v26.11.2/` unless prefixed
`v26.11.1/`):

- G1a: `waves/w3-t3-silicon.md:36` — 87.5% of the 32,768 B budget, PASS.
- G1b: `11-measured-claims.md:38` — no witness receipt yet; do not assert
  the 55.7 KB figure as fact until one lands.
- G2: `v26.11.1/waves/gate-5-silicon.md:26` — cgap-arrow, two passes.
- G3: `waves/p2-asm-0blr-census.md:37` — the one blr sits in
  `unify_to_fixpoint_opt`.
- G5: `waves/a3-ring-sink.md:34-36` — 150.7 us @256 and 2.472 ms @4096.
- G5a: `waves/p2-penstock-escalation-law.md:32` — 10.20 ns/frame, 10.1x
  (quiet host, divan fastest/median, `11-measured-claims.md` rows 5/28;
  contended divergence 124.06 ns/frame slowest, row 28).

## Scope

| In scope | Out of scope |
|----------|--------------|
| Inner kernel crates (arrow, blake3, parquet, core, bn3,
bont, bosym) | cgap-datafusion (removed) |
| riscv32imc compile ceilings (object + ELF) | Host-only benches as authority |
| Ingress, commit, unify, seal hot paths | Non-hot-path storage surfaces |
| Linker ASSERT + nm purity gates | CI-as-truth footprint claims |
| DMADV Define artifacts (CTQ, SIPOC, FMEA) | Publish sequencing (frozen here) |

## VOC to CTQ Flowdown

| VOC | CTQ | Measure | Goal |
|-----|-----|---------|------|
| Bankable deterministic replay | fixed .text, 0 alloc, 0 blr | ASSERT; nm; objdump | G1-G3 |
| Predictable latency under load | ingress < 15 ns/frame | per-frame hot-loop bench | G4 |
| Commit never stalls ingest plane | commit < 4 µs/tx | FFI ring bench | G5 |
| Saturate a core with facts | >= 1M facts/s/core | facts/s/core bench | G6 |

## SIPOC

| Supplier | Input | Process | Output | Customer |
|----------|-------|---------|--------|----------|
| Lane fleet | source + bench subjects | DMADV phases | wave receipts | Integration lane |
| Bench harness | riscv32imc audit builds | size, objdump, timing | bytes, ns, us | Charter CTQs |
| Court corpus | mutations + restores | shim-court flips | fail-closed verdicts | Gate owners |
| Coordinator | lane leases | atomic commits | landed history | Sponsor |

## DMADV Tollgate Roadmap

| Tollgate | Phase | Exit criterion | Standing |
|----------|-------|----------------|----------|
| T1 | Define | charter landed with receipted CTQs | DONE (this file) |
| T2 | Measure | each [TARGET] goal gains a witness receipt | OPEN |
| T3 | Analyze | 1 blr defect closed; census re-run 0/0 | OPEN |
| T4 | Design | linker ASSERT enforcing <= 131,072 B .text lands | OPEN |
| T5 | Verify | full court battery green on linked-ELF subject | OPEN |

## FMEA

| Risk | S | O | D | RPN | Mitigation |
|------|---|---|---|-----|------------|
| Stale mtime gives vacuous pass | 8 | 5 | 3 | 120 | mtime-bump law
(a02-finish-receipt.md:170) |
| 1 blr in fixpoint_opt regresses | 7 | 4 | 3 | 84 | re-run 0blr census per wave |
| Pre-link object asserted as ELF | 6 | 5 | 3 | 90 | subject-class rule
(11-measured-claims.md) |
| Unverified parquet figure in prose | 5 | 4 | 2 | 40 | UNVERIFIED-BY-RECEIPT tag |
| Publish concurrent with lever lanes | 7 | 3 | 2 | 42 | publish law (OVERNIGHT-LOOP.md:146) |
| Compile freeze starves lanes | 6 | 4 | 2 | 48 | 10-min freeze SLA, dual disclosure |

## Action Items

| # | Action | Owner | Status |
|---|--------|-------|--------|
| A1 | Land charter (this file) | lane D1 | DONE |
| A2 | §8.1 tree-recovery: restore wave source | coordinator | DONE — a9827591..88ac2b70 |
| A3 | Witness receipts for G1, G4, G6 targets | T2 lanes | OPEN |
| A4 | Close the 1 blr defect in unify_to_fixpoint_opt | p2 lane | OPEN |
| A5 | Land linker ASSERT for .text ceiling | T4 lane | OPEN |

## Receipt: Commands + Exits

| Command | Exit | Purpose |
|---------|------|---------|
| `awk 'length > 100 {print NR, length}' 14-dflss-charter.md` | 0 | line-length gate (<=100) |
| `cat docs/jira/v26.11.2/14-dflss-charter.md` | 0 | landed-file witness |

Layout note (CGAP-PRD-05): CSCB fact frame = 24 B (3x u64 LE, FRAME_BYTES=24);
the 64 B figure is cgap-arrow Quad64 layout only. audit/cgapprd-03-frame-layout.md.
