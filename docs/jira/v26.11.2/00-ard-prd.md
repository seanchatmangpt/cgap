# 00-ard-prd

CHARTER-CGAP-STARDF-v26.11.2 — ARD/PRD for the v26.11.2 campaign. Transcribed to disk so the
overnight loop executes from disk, not operator memory. Governing source: CHARTER-CGAP-STARDF-
v26.11.2 §1–§6. Repo: /Users/sac/cgap (canonical checkout, branch main).

## §1 Document ID + strategic objective

- Document ID: CHARTER-CGAP-STARDF-v26.11.2 (ARD/PRD, this file: 00-ard-prd.md).
- Strategic objective: total convergence — the cgap/stardf substrate reaches feature and
  performance parity with the incumbent RDF/semantic stack, then obsoletes it: every measured
  Pareto bottleneck closes, every prohibition stays enforced, and the like-for-like
  materialization benchmark clears the incumbent's published throughput envelope.
- Success is single-sentence falsifiable: the like-for-like materialization bench (§2, R4)
  beats RDFox's published 2–6.1M facts/s envelope on identical workload shape, and all five
  Pareto levers (§3) hit their targets, with receipts in waves/.

## §2 The five market-wedge requirements

Each wedge closes one measured Pareto bottleneck (targets in §3 are [TARGET-v26.11.2] until
measured by the stated method):

- MW1: Close the five measured Pareto bottlenecks — the L1–L5 levers in §3, each with a
  court, a mutation, and a receipt in docs/jira/v26.11.2/waves/.
- MW2: Strip std from stardf-hash and stardf-datalog — both crates compile #![no_std] with
  alloc-free hot paths where the layout permits; compile-abort of std linkage is the gate.
- MW3: Finish the 22-crate registry sweep — every workspace crate has a registry row,
  standing, and court pointer; no silent-missing cells (coverage-compiler rule).
- MW4: Like-for-like materialization bench vs RDFox 2–6.1M facts/s — identical workload
  shape, published-method comparison, receipt with machine + method; before measurement the
  number is a target, never a claim.
- MW5: Zero-DataFusion law — cgap-datafusion-removal.md is the citation; no DataFusion
  dependency re-enters any workspace member (gate-enforced).

## §3 The FIVE PARETO LEVERS (verbatim)

Verbatim (operator transcription, CHARTER-CGAP-STARDF-v26.11.2 §3):
- L1 chains-ingress: 102.7ns/frame, 85–91% BLAKE3 fold → two-stage admission escalation,
  target <15ns/frame.
- L2 chains-parquet: 41.54ms@1M, 86% byte-wise FNV → word-wise u64 SWAR, target <10ms@1M.
- L3 chains-bn3: 81.2→99.9% nested-loop → PLNF bit-slice semi-join + noalloc radix probe,
  target >20×@1M.
- L4 chains-sovereign: 45.8µs/tx, 87.6% File::create (sovereign.rs:706-724) → pre-leased
  ring sink, target <4µs/tx.
- L5 chains-seal: 2.37µs/row three-way → fused ASC450 into P28 fold, target <1.45µs/row.

## §4 The EIGHT RELEASE GATES (G1–G8)

Full spec lives in 01-release-gates.md. One line each:

- G1: All five Pareto levers (L1–L5) at target with waves/ receipt.
- G2: stardf-hash and stardf-datalog compile #![no_std] (std-linkage gate exits nonzero on std).
- G3: 22-crate registry sweep complete — zero missing cells, every cell pass or
  registered-divergence.
- G4: Like-for-like materialization bench run, method stated, beats RDFox envelope.
- G5: Zero-DataFusion gate green (cgap-datafusion-removal.md holds; no re-entry).
- G6: Every claim in landing docs carries a waves/ file:line receipt or [TARGET-v26.11.2].
- G7: All courts flip — each named anti-vacuity mutation kills its test; a court that
  cannot flip is FAIL; skips are loud ([SKIP] <reason>).
- G8: Workspace green — root cargo build --workspace + cargo test -p cgap-court pass.

## §5 Prohibitions

- Zero DataFusion: citation docs/jira/v26.11.1/cgap-datafusion-removal.md (authoritative
  record of removal); MW5/G5 enforce non-re-entry.
- Zero runtime string interning: strings_admitted counter stays 0; any nonzero is a gate
  failure, not a warning.
- Zero epistemic laundering: every perf/status claim carries a waves/ file:line receipt or
  is explicitly marked [TARGET-v26.11.2]. GB/s is never quoted as a multiple of facts/s —
  throughput units are workload-specific and never converted implicitly.

## §6 Pointer index

- 01-release-gates.md — full G1–G8 gate specs (exit codes, commands, mutation set).
- 02-bench-protocol.md — measurement method for L1–L5 + MW4 like-for-like bench.
- 03-court-corpus.md — court files + anti-vacuity mutations per lever.
- 04-registry-sweep.md — 22-crate registry sweep plan and cell schema.
- 05-overnight-mutation-set.md — mutations the overnight loop applies/fails-closed on.
- OVERNIGHT-LOOP.md — the loop's execution contract (reads this file as entry).
- HANDOFF.md — cross-wave handoff record (receipts, standing, divergences).
- Receipt for this file: waves/v2-01-charter.md.
