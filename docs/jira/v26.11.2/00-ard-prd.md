# 00-ard-prd

> SUPERSEDED by `12-ard-prd-cgap.md` (canonical product charter, operator
> landed 2026-11-02). This file remains the v26.11.2 campaign-operational record.

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
- Standing of that sentence (claims registry `11-measured-claims.md`, re-read 2026-11-02):
  the cgap MW4 arm is [TARGET], never a claim — RDFox was NOT executed; the campaign's only
  receipted facts/s figure (addendum row 30) is NOT RDFox-comparable (addendum D2, window
  1,365 vs 1M). Post-measurement lever standing: L1 met on QUIET HOST ONLY (rows 5/8;
  contention divergence registered as row 28 — median 19.03, slowest 124.06 ns/frame),
  L2 NOT MET (row 10), L5 FAILED-BY-MEASUREMENT (row 12), L3/L4 unmeasured (row 27 loud
  skip). §3's baselines/transcription below remain verbatim; status lives here.

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

Post-measurement status (registry rows, `11-measured-claims.md` §1 — verbatim block above
is the charter transcription, not result figures): L1 target met quiet-host-only, 10.20
ns/frame fastest (rows 5/6/8; dispatch's 5.70 ns penstock row NOT reproduced — row 7
records 7.71 ns); L2 target NOT MET (42.7-43.1 ms vs <10 ms, row 10); L5 target
FAILED-BY-MEASUREMENT (3.04 fused vs 2.37 unfused µs/row, row 12); L3/L4 no receipt at
consolidation (row 27 [SKIP]). All figures above remain [TARGET-v26.11.2] class per §2
preamble; measured counterparts cite the registry, never this block.

## §4 The EIGHT RELEASE GATES (G1–G8)

Full spec lives in 01-release-gates.md (COMMAND / RECEIPT / FLIP / BLOCKED per gate).
Gate names + one line each, transcribed from 01-release-gates.md (on-disk definitions):

- G1-PUBLISH: release_check over the 20/23 publishable set (bench/beam/court are
  publish = false; 23/23 members explicit, 20 true / 3 false per
  audit/pub-02-flag-table-rederivation.md and the refreshed table
  01-release-gates.md:325-351); workspace version exactly `26.11.2`; FLIP = strip a
  crate's `description` in a /tmp sandbox, gate must FAIL; sha256 fail-closed restore.
  Count scope note: prior "19/22 published" registry state was UNVERIFIED (guarded table
  01-release-gates.md:28-32) and "19/22" as publishable-set is superseded by PUB-02b
  (audit/pub-02-flag-table-rederivation.md, 2026-11-02); the set is 20/23.
- G2-PARETO: all five Pareto levers measured by the named bench targets with flip
  witnesses per level — L1 < 15 ns/frame, L2 < 10 ms @ 1M rows, L3 > 20x @ 1M,
  L4 < 4 us/tx, L5 < 1.45 us/row; a level met with no flip witness is
  `VACUOUS: L<n> threshold met with no flip court` and exit 3.
  Post-measurement (registry §1): L1 met quiet-host-only (rows 5/8; divergence row 28),
  L2 NOT MET (row 10), L5 FAILED-BY-MEASUREMENT (row 12), L3/L4 unmeasured (row 27) —
  G2 cannot pass today; UNVERIFIED-FOR-PAPER until L3/L4 land receipts.
- G3-MAT-BENCH: like-for-like materialization closure bench — > 50M closure facts/s/core
  [TARGET-v26.11.2 — registry row 26: RDFox NOT executed, cgap arm unmeasured; the
  comparison arm is a published-envelope citation, and rows 5-#10 are CATEGORY-ERROR if
  comparison arm is a published-envelope citation, and rows 5-10 are CATEGORY-ERROR if
  ever quoted against facts/s per registry §2].
  single-core, RDFox 2–6.1M facts/s baseline named with source + machine/method lines
  for both sides; FLIP = mutate the closure kernel to skip one derivation rule.
- G4-DRED: DRED churn court (paper-dred.md is prose, not a bench) — 90% churn workload,
  < 3.0 ns/batch, < 0.15 ns entropy variance across >= 9 batches with per-batch numbers,
  > 15M facts/s; FLIP = skip the entropy recompute, variance gate must flip.
  UNVERIFIED-FOR-PAPER: no register row covers any G4 figure (registry §1 rows 1-32 have
  no DRED entry; paper-dred.md is prose, not a bench — missing referent, never quote).
- G5-SILICON: riscv32imc footprint — flash < 32,768 B, SRAM + stack < 4,096 B
  [TARGET; nearest receipted figure is row 15's 28,679 B — a PRE-LINK audit-profile
  OBJECT, not linked ELF; repo law: footprint truth is linked ELF only],
  0 alloc/panic symbols (symbol_gate.py), QEMU stack-paint high-water recorded;
  FLIP = inject an alloc symbol or panic site in a /tmp copy, gate must FAIL.
- G6-NO-DF: zero `datafusion` matches (comments included) across both repos' manifests,
  locks, and crate sources; FLIP = append `datafusion = "42"` to a /tmp Cargo.toml copy,
  gate must FAIL; self-check corrupts a restore to prove restore-detection is real.
- G7-PATENT: `verify_ost_design_around.py` over patent/claims.txt — 4/4 claims
  DISTINCT-MECHANISM + RDF-11 auditor sign-off; FLIP = feed a merged/near-duplicate
  claim, verdict must drop to 3/4.
- G8-STARDF-F1: purrdf-side census + golden courts — stardf_f1_census.py exit 0 with
  exactly 11 new mapping rows, stardf_golden.py 10/10 byte-match, KD-12/KD-14
  differential courts each with a witnessed flip.

Exit-code contract (all gates): 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.
Skips are loud ([SKIP] <reason>), never silent.

## §5 Prohibitions

- Zero DataFusion: citation docs/jira/v26.11.1/cgap-datafusion-removal.md (authoritative
  record of removal); MW5/G5 enforce non-re-entry.
- Zero runtime string interning: strings_admitted counter stays 0; any nonzero is a gate
  failure, not a warning.
- Zero epistemic laundering: every perf/status claim carries a waves/ file:line receipt or
  is explicitly marked [TARGET-v26.11.2]. GB/s is never quoted as a multiple of facts/s —
  throughput units are workload-specific and never converted implicitly.

## §6 Pointer index

On disk now (v26.11.2/):

- 01-release-gates.md — full G1–G8 gate specs (COMMAND/RECEIPT/FLIP/BLOCKED per gate).
- 02-pareto-levers.md — L1–L5 lever detail (baseline, method, court per lever).
- 03-ost-design-around-register.md — OST design-around register (patent lane).
- 04-stardf-closure.md — stardf closure subject + court record.
- 05-wave-plan.md — wave plan for the campaign (lane dispatch map).
- OVERNIGHT-LOOP.md — the loop's execution contract (reads this file as entry).
- HANDOFF.md — cross-wave handoff record (receipts, standing, divergences).

Landing set (sibling lanes in flight; reference as the landing set, do not block on
them):

- 06-bench-protocol.md — measurement method for L1–L5 + MW4 like-for-like bench.
- 07-court-corpus.md — court files + anti-vacuity mutations per lever.
- 08-registry-sweep.md — 22-crate registry sweep plan and cell schema.
- 09-overnight-mutation-set.md — mutations the overnight loop applies/fails-closed on.

### research/ subsection (g01–g20, one-line hook each)

- g01-ggen-architecture.md — workspace member inventory read from Cargo.toml:52-100.
- g02-manifest-schema.md — two manifest schemas, not one (which schema where).
- g03-marketplace-inventory.md — marketplace headline facts (packs, identity, counts).
- g04-receipt-system.md — three receipt mechanisms coexist by design, never merged.
- g05-shacl-validation.md — SHACL validation layers for generated artifacts.
- g06-sparql-rdf-kernel.md — SPARQL/RDF kernel architecture survey.
- g07-bench-conventions.md — bench framework landscape + conventions of record.
- g08-court-mutation-patterns.md — engine-level exit-code contract + mutation patterns.
- g09-nostd-patterns.md — where #![no_std] lives and the patterns around it.
- g10-ci-publish-mechanics.md — key CI/publish mechanics finding for cgap v26.11.2.
- g11-template-engine.md — template engine layout (generator-of-record internals).
- g12-lockfile-replay.md — ggen.lock shape and byte-identical replay mechanics.
- g13-doc-taxonomy.md — governing doc standard (house style, taxonomy).
- g14-cli-structure.md — two distinct CLI systems and which one cgap consumes.
- g15-governance-skills.md — authority structure of governance skills.
- g16-pack-authoring.md — canonical pack layout (packs/cgap-pack/).
- g17-ontology-assets.md — cgap's own O-ABI packs as doc subjects.
- g18-testing-discipline.md — test organization + Chicago court conventions.
- g19-marketplace-governance.md — marketplace registry identity + schema.
- g20-perf-claims-inventory.md — perf-claims inventory headline (claim-vs-receipt map).

Receipt for this file: waves/v2-01-charter.md (charter); this reconciliation:
waves/w1-07-prd-repoint.md.

## §7 Runtime model doctrine (operator, 2026-11-01)

Two-layer split, binding on all receipts: **RDF is the compile-time authoring/validation
surface** (cgap-pack ontology, cnv individuals, SPARQL gates/witnesses — ~50 KB TTL, not an
engine). **The runtime is dictionary-encoded Arrow columns**: Bn3Statement is a
three-column table of u64 SlotIds (SlotId = BLAKE3(lexeme)[0..8] — a dictionary-encoding
rule, not an RDF store); joins are columnar semi-joins; materialization is
scan-column → hash-join → emit-rows. No graph-engine pretense: MW4 materialization is
framed and measured as Datalog-on-Arrow derived-facts/s, which is what makes the number
comparable rather than category-erroneous. The 500x-1500x scan-vs-materialization framing
is banned (unit category error); cgap never quotes it.

## §8 Project boundary (operator, 2026-11-01)

**stardf (separate repo, OUT OF SCOPE here)** is the RDF research substrate: binary
ontologies, quoted triples, semantic planning, Datalog reasoning, no_std convergence —
the 5-capability experimentation home. **cgap (this repo)** is the shipped columnar/
attestation substrate it feeds: kernels, receipt economy, gates, CLI, publish wave; its
RDF surface is thin by design (SlotId interning, bn3 statement arrays, pack/gate
authoring graph). The crates/stardf-datalog member is cgap-side Arrow-shaped
materialization code with a borrowed name — rename owed at integration; it carries no
stardf-project charter. All v26.11.2 docs are re-based on this split; stardf-referencing
sections are OUT-OF-SCOPE annotations, not open work.
Frame layout note (CGAP-PRD-05, 2026-11-02): the CSCB fact frame is 24 B =
3x u64 LE (FRAME_BYTES=24; ring bound 32,768/24 = 1,365 facts). The 64 B
figure exists only as cgap-arrow's Quad64 quad-buffer layout (layout.rs:36),
not a fact-frame size — audit/cgapprd-03-frame-layout.md.
